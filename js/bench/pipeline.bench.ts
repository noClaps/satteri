/**
 * End-to-end JS pipeline benchmarks.
 *
 * Covers the full stack from the JS side: parse → HAST binary → HTML.
 * Requires the native Rust module to be built:
 *   cargo build --release -p tryckeri-napi
 *
 * Run with: pnpm bench
 */

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { bench, describe } from 'vitest';
import {
  parseToBuffer,
  parseToHastBuffer,
  parseToHtml,
  mdastBufferToHastBuffer,
  hastBufferToHtmlStr,
  compileMdx,
  compileMdxFromBuffer,
} from '../src/parse.ts';
import { ArenaReader } from '../src/arena-reader.ts';
import { createProcessor } from '../src/processor.ts';
import headingIds from '../src/plugins/heading-ids.ts';
import collectHeadings from '../src/plugins/collect-headings.ts';
import lintHeadingDepth from '../src/plugins/lint-heading-depth.ts';
import flattenHeadings from '../src/plugins/flatten-headings.ts';
import { fallbackLangNoIO } from '../src/plugins/fallback-lang.ts';
import { rehypeTasklistEnhancer } from '../src/plugins/rehype-tasklist-enhancer.ts';
import { HastArenaReader } from '../src/hast-reader.ts';
import { visitHastArena } from '../src/hast-visitor.ts';
import { DataMap } from '../src/data-map.ts';

const __dirname = dirname(fileURLToPath(import.meta.url));

const MARKDOWN = readFileSync(
  join(__dirname, '../../crates/bench/fixtures/markdown.md'),
  'utf8',
);

const MDX = `import {Chart} from './chart.js'

# Hello, world

Some *emphasis* and **strong** content.

<Chart values={[1, 2, 3]} />

> A blockquote with a [link](https://example.com).

- item one
- item two
- item three
`;

const TASKLIST_MD = `# Project checklist

- [x] Set up repository
- [ ] Write documentation
- [x] Add tests
- [ ] Deploy to production
- [x] Review pull request

Some text with a [link](https://example.com) here.

## Sub-tasks

- [x] Task A
- [ ] Task B
- [x] Task C
`;

// Pre-computed buffers so intermediate benchmarks measure only their step.
const mdastBuf = parseToBuffer(MARKDOWN);
const hastBuf = mdastBufferToHastBuffer(mdastBuf);

// ---------------------------------------------------------------------------
// Parse benchmarks
// ---------------------------------------------------------------------------

describe('parse', () => {
  bench('parseToBuffer — Markdown → MDAST binary', () => {
    parseToBuffer(MARKDOWN);
  });

  bench('parseToHastBuffer — Markdown → HAST binary (combined Rust path)', () => {
    parseToHastBuffer(MARKDOWN);
  });
});

// ---------------------------------------------------------------------------
// HAST / HTML benchmarks
// ---------------------------------------------------------------------------

describe('hast', () => {
  bench('mdastBufferToHastBuffer — MDAST binary → HAST binary', () => {
    mdastBufferToHastBuffer(mdastBuf);
  });

  bench('hastBufferToHtmlStr — HAST binary → HTML string', () => {
    hastBufferToHtmlStr(hastBuf);
  });

  bench('full pipeline — parseToBuffer → mdastBufferToHastBuffer → hastBufferToHtmlStr', () => {
    const buf = parseToBuffer(MARKDOWN);
    const hast = mdastBufferToHastBuffer(buf);
    hastBufferToHtmlStr(hast);
  });
});

// ---------------------------------------------------------------------------
// MDAST reader benchmark (JS-only, no native call)
// ---------------------------------------------------------------------------

describe('arena-reader', () => {
  bench('ArenaReader — walk all nodes from pre-parsed buffer', () => {
    const reader = new ArenaReader(mdastBuf);
    for (let i = 0; i < reader.nodeCount; i++) {
      reader.getNode(i);
    }
  });
});

// ---------------------------------------------------------------------------
// Plugin pipeline benchmarks
// ---------------------------------------------------------------------------

// Processors are created once and reused across bench iterations — same as
// production usage where a processor is set up once per build.
const processorHeadingIds = createProcessor({ plugins: [headingIds] });
const processorAllPlugins = createProcessor({
  plugins: [headingIds, collectHeadings, lintHeadingDepth(), flattenHeadings()],
});

const processorFallbackLang = createProcessor({ plugins: [fallbackLangNoIO()] });
const processorFallbackPlusHeadings = createProcessor({
  plugins: [headingIds, fallbackLangNoIO()],
});
const processorAll = createProcessor({
  plugins: [headingIds, collectHeadings, lintHeadingDepth(), flattenHeadings(), fallbackLangNoIO()],
});

describe('plugins', () => {
  bench('no plugins — parseToHtml (pure Rust, single NAPI call)', () => {
    parseToHtml(MARKDOWN);
  });

  bench('headingIds — parseToBuffer + processBuffer', () => {
    const buf = parseToBuffer(MARKDOWN);
    processorHeadingIds.processBuffer(buf);
  });

  bench('fallbackLang (no I/O) — parseToBuffer + processBuffer', () => {
    const buf = parseToBuffer(MARKDOWN);
    processorFallbackLang.processBuffer(buf);
  });

  bench('headingIds + fallbackLang — parseToBuffer + processBuffer', () => {
    const buf = parseToBuffer(MARKDOWN);
    processorFallbackPlusHeadings.processBuffer(buf);
  });

  bench('all 5 plugins — parseToBuffer + processBuffer', () => {
    const buf = parseToBuffer(MARKDOWN);
    processorAll.processBuffer(buf);
  });

  bench('all 5 plugins + HAST → HTML — full e2e', () => {
    const buf = parseToBuffer(MARKDOWN);
    const result = processorAll.processBuffer(buf);
    const hast = mdastBufferToHastBuffer(result.buffer as Uint8Array);
    hastBufferToHtmlStr(hast);
  });

  bench('plugin overhead only — all 5 plugins on pre-parsed buffer', () => {
    processorAll.processBuffer(mdastBuf);
  });
});

// ---------------------------------------------------------------------------
// HAST (rehype) plugin benchmarks
// ---------------------------------------------------------------------------

const tasklistHastBuf = (() => {
  const buf = parseToBuffer(TASKLIST_MD);
  return mdastBufferToHastBuffer(buf);
})();

const tasklistPlugin = rehypeTasklistEnhancer();

describe('rehype plugins', () => {
  bench('no rehype plugin — HAST buffer → HTML', () => {
    hastBufferToHtmlStr(tasklistHastBuf);
  });

  bench('rehypeTasklistEnhancer — visit HAST buffer', () => {
    const reader = new HastArenaReader(tasklistHastBuf);
    const dm = new DataMap();
    visitHastArena(reader, tasklistPlugin, dm);
  });

  bench('full pipeline: parse → HAST → rehype plugin → HTML', () => {
    const mdastBuf = parseToBuffer(TASKLIST_MD);
    const hBuf = mdastBufferToHastBuffer(mdastBuf);
    const reader = new HastArenaReader(hBuf);
    const dm = new DataMap();
    visitHastArena(reader, tasklistPlugin, dm);
    hastBufferToHtmlStr(hBuf);
  });

  bench('full pipeline: parse → MDAST plugins → HAST → rehype plugin → HTML', () => {
    const mdastBuf = parseToBuffer(TASKLIST_MD);
    processorHeadingIds.processBuffer(mdastBuf);
    const hBuf = mdastBufferToHastBuffer(mdastBuf);
    const reader = new HastArenaReader(hBuf);
    const dm = new DataMap();
    visitHastArena(reader, tasklistPlugin, dm);
    hastBufferToHtmlStr(hBuf);
  });
});

// ---------------------------------------------------------------------------
// MDX benchmarks
// ---------------------------------------------------------------------------

// MDX compilation requires the native module to be rebuilt after adding compileMdx.
// Run: cargo build --release -p tryckeri-napi
try {
  compileMdx('# test'); // probe — throws if not yet built
  // Pre-parse MDX with MDX constructs enabled so compileMdxFromBuffer has a valid buffer.
  // (parseToBuffer uses default ParseOptions which don't enable MDX constructs, so we
  // use compileMdx itself as the parse step for that bench.)
  const mdxBuf = parseToBuffer(MDX); // MDAST binary (no MDX constructs — intentional: measures the buffer path perf)
  describe('mdx', () => {
    bench('compileMdx — MDX source → JavaScript (parse + compile)', () => {
      compileMdx(MDX);
    });
    bench('compileMdxFromBuffer — pre-parsed MDAST binary → JavaScript', () => {
      compileMdxFromBuffer(mdxBuf);
    });
  });
} catch {
  // compileMdx not available in the current native binary; skip.
}
