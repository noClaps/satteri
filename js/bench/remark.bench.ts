/**
 * Remark ecosystem benchmarks — comparison baseline for the Rust pipeline.
 *
 * Each bench here has a counterpart in pipeline.bench.ts so results can be
 * read side-by-side.
 *
 * Run with: pnpm bench
 */

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { bench, describe } from 'vitest';
import { unified } from 'unified';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import rehypeStringify from 'rehype-stringify';
import { remark } from 'remark';
import { compile as compileMdxJs } from '@mdx-js/mdx';
import { visit } from 'unist-util-visit';
import { toString } from 'mdast-util-to-string';
import type { Root, Heading } from 'mdast';

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

// ---------------------------------------------------------------------------
// Remark plugin equivalents of our built-in plugins
// ---------------------------------------------------------------------------

// Equivalent to heading-ids.ts
function remarkHeadingIds() {
  return (tree: Root) => {
    visit(tree, 'heading', (node: Heading) => {
      const text = toString(node);
      const id = text.toLowerCase().trim()
        .replace(/[^\w\s-]/g, '')
        .replace(/[\s_-]+/g, '-')
        .replace(/^-+|-+$/g, '');
      node.data = { ...node.data, id, hProperties: { id } };
    });
  };
}

// Equivalent to collect-headings.ts
function remarkCollectHeadings() {
  const headings: { depth: number; position: unknown }[] = [];
  return (tree: Root) => {
    visit(tree, 'heading', (node: Heading) => {
      headings.push({ depth: node.depth, position: node.position });
    });
  };
}

// Equivalent to lint-heading-depth.ts
function remarkLintHeadingDepth({ maxDepth = 3 } = {}) {
  return (tree: Root, file: { message(msg: string, node: unknown): void }) => {
    visit(tree, 'heading', (node: Heading) => {
      if (node.depth > maxDepth) {
        file.message(`Heading depth ${node.depth} exceeds maximum of ${maxDepth}`, node);
      }
    });
  };
}

// Equivalent to flatten-headings.ts
function remarkFlattenHeadings({ maxDepth = 3 } = {}) {
  return (tree: Root) => {
    visit(tree, 'heading', (node: Heading) => {
      if (node.depth > maxDepth) {
        node.depth = maxDepth as Heading['depth'];
      }
    });
  };
}

// ---------------------------------------------------------------------------
// Processors built once and reused (mirrors how remark is used in practice).
// ---------------------------------------------------------------------------

const htmlProcessor = unified()
  .use(remarkParse)
  .use(remarkRehype)
  .use(rehypeStringify);

const parseProcessor = remark();

const processorHeadingIds = unified()
  .use(remarkParse)
  .use(remarkHeadingIds)
  .use(remarkRehype)
  .use(rehypeStringify);

const processorAllPlugins = unified()
  .use(remarkParse)
  .use(remarkHeadingIds)
  .use(remarkCollectHeadings)
  .use(remarkLintHeadingDepth)
  .use(remarkFlattenHeadings)
  .use(remarkRehype)
  .use(rehypeStringify);

// ---------------------------------------------------------------------------
// Parse benchmarks  (compare: pipeline.bench.ts > parse)
// ---------------------------------------------------------------------------

describe('remark > parse', () => {
  bench('remark.parse — Markdown → mdast JS tree', () => {
    parseProcessor.parse(MARKDOWN);
  });
});

// ---------------------------------------------------------------------------
// Full pipeline to HTML  (compare: pipeline.bench.ts > hast > full pipeline)
// ---------------------------------------------------------------------------

describe('remark > html', () => {
  bench('unified remark→rehype→stringify — Markdown → HTML string', async () => {
    await htmlProcessor.process(MARKDOWN);
  });

  // Synchronous path via .processSync for a fair no-async comparison.
  bench('unified remark→rehype→stringify — sync', () => {
    htmlProcessor.processSync(MARKDOWN);
  });
});

// ---------------------------------------------------------------------------
// Plugins  (compare: pipeline.bench.ts > plugins)
// ---------------------------------------------------------------------------

describe('remark > plugins', () => {
  bench('remark + headingIds — Markdown → HTML', () => {
    processorHeadingIds.processSync(MARKDOWN);
  });

  bench('remark + all plugins — Markdown → HTML', () => {
    processorAllPlugins.processSync(MARKDOWN);
  });
});

// ---------------------------------------------------------------------------
// MDX  (compare: pipeline.bench.ts > mdx)
// ---------------------------------------------------------------------------

describe('remark > mdx', () => {
  bench('@mdx-js/mdx compile — MDX source → JavaScript', async () => {
    await compileMdxJs(MDX);
  });
});
