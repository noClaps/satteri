/**
 * Parse all .mdx files from the docs repo through the Rust pipeline via NAPI.
 *
 * Usage: node scripts/parse-docs.mjs
 */

import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const native = require('../tryckeri_napi.linux-x64-gnu.node');

const DOCS_DIR = '/home/erika/Projects/docs';

function collectMdxFiles(dir, out = []) {
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    try {
      const stat = statSync(full);
      if (stat.isDirectory()) {
        collectMdxFiles(full, out);
      } else if (entry.endsWith('.mdx')) {
        out.push(full);
      }
    } catch {
      // skip
    }
  }
  return out;
}

const files = collectMdxFiles(DOCS_DIR).sort();
console.log(`Found ${files.length} .mdx files\n`);

let success = 0;
let failures = [];
let totalBytes = 0;

const start = performance.now();

for (const path of files) {
  const source = readFileSync(path, 'utf8');
  totalBytes += Buffer.byteLength(source);

  try {
    // Full pipeline: parse MDX → HAST → HTML
    const html = native.parseMdxToHtml(source);
    if (typeof html !== 'string') throw new Error('not a string');
    success++;
  } catch (e) {
    failures.push({ path: relative(DOCS_DIR, path), error: e.message });
  }
}

const elapsed = performance.now() - start;
const mb = totalBytes / (1024 * 1024);

console.log('Results:');
console.log(`  Success: ${success}/${files.length}`);
console.log(`  Failed:  ${failures.length}`);
console.log(`  Total:   ${mb.toFixed(2)} MB in ${elapsed.toFixed(0)}ms`);
console.log(`  Speed:   ${(mb / (elapsed / 1000)).toFixed(1)} MB/s`);

if (failures.length > 0) {
  console.log('\nFailures:');
  for (const f of failures.slice(0, 20)) {
    console.log(`  ${f.path} — ${f.error}`);
  }
  if (failures.length > 20) {
    console.log(`  ... and ${failures.length - 20} more`);
  }
}
