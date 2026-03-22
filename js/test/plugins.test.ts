import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import { buildHelloWorldBuffer, buildTestBuffer } from './fixtures.ts';
import { createProcessor } from '../src/processor.ts';
import { definePlugin } from '../src/plugin.ts';
import headingIds from '../src/plugins/heading-ids.ts';
import lintHeadingDepth from '../src/plugins/lint-heading-depth.ts';
import flattenHeadings from '../src/plugins/flatten-headings.ts';
import collectHeadings from '../src/plugins/collect-headings.ts';

function buildHeadingBuffer(depth: number, text: string): ArrayBuffer {
  const enc = new TextEncoder();
  const textBytes = enc.encode(text);
  const sourceStr = '#'.repeat(depth) + ' ' + text;
  const textSourceStart = depth + 1;

  const typeData = new Uint8Array(9);
  typeData[0] = depth;
  new DataView(typeData.buffer).setUint32(1, textSourceStart, true);
  new DataView(typeData.buffer).setUint32(5, textBytes.length, true);

  return buildTestBuffer({
    source: sourceStr,
    nodes: [
      { id: 0, type: 0, parent: 0, childrenStart: 0, childrenCount: 1, dataOffset: 0, dataLen: 0 },
      { id: 1, type: 2, parent: 0, childrenStart: 1, childrenCount: 1, dataOffset: 0, dataLen: 1 },
      { id: 2, type: 10, parent: 1, childrenStart: 0, childrenCount: 0, dataOffset: 1, dataLen: 8 },
    ],
    children: [1, 2],
    typeData,
  });
}

describe('built-in plugins integration', () => {
  describe('heading-ids', () => {
    it('slugifies "Hello" → "hello"', () => {
      const buf = buildHelloWorldBuffer();
      const processor = createProcessor({ plugins: [headingIds] });
      const result = processor.processBuffer(buf);
      const data = result.dataMap.get(1);
      assert.equal(data?.id, 'hello');
    });

    it('slugifies "Hello World" → "hello-world"', () => {
      const buf = buildHeadingBuffer(1, 'Hello World');
      const processor = createProcessor({ plugins: [headingIds] });
      const result = processor.processBuffer(buf);
      const data = result.dataMap.get(1);
      assert.equal(data?.id, 'hello-world');
    });

    it('slugifies text with special chars correctly', () => {
      const buf = buildHeadingBuffer(1, 'Hello, World!');
      const processor = createProcessor({ plugins: [headingIds] });
      const result = processor.processBuffer(buf);
      const data = result.dataMap.get(1);
      assert.match(String(data?.id), /^[a-z0-9-]+$/);
      assert.equal(data?.id, 'hello-world');
    });

    it('sets hProperties.id to the same slug', () => {
      const buf = buildHelloWorldBuffer();
      const processor = createProcessor({ plugins: [headingIds] });
      const result = processor.processBuffer(buf);
      const data = result.dataMap.get(1);
      assert.deepEqual(data?.hProperties, { id: data?.id });
    });

    it('produces no mutations (data change, no structural replace)', () => {
      const buf = buildHelloWorldBuffer();
      const processor = createProcessor({ plugins: [headingIds] });
      const result = processor.processBuffer(buf);
      assert.equal(result.mutationCount, 0);
    });
  });

  describe('lintHeadingDepth', () => {
    it('no diagnostic when h1 is within maxDepth=1', () => {
      const buf = buildHelloWorldBuffer();
      const processor = createProcessor({ plugins: [lintHeadingDepth({ maxDepth: 1 })] });
      assert.equal(processor.processBuffer(buf).diagnostics.length, 0);
    });

    it('no diagnostic when h1 is within maxDepth=3 (default)', () => {
      const buf = buildHelloWorldBuffer();
      const processor = createProcessor({ plugins: [lintHeadingDepth()] });
      assert.equal(processor.processBuffer(buf).diagnostics.length, 0);
    });

    it('reports 1 diagnostic with severity "warning" when maxDepth=0 on h1', () => {
      const buf = buildHelloWorldBuffer();
      const processor = createProcessor({ plugins: [lintHeadingDepth({ maxDepth: 0 })] });
      const result = processor.processBuffer(buf);
      assert.equal(result.diagnostics.length, 1);
      assert.equal(result.diagnostics[0].severity, 'warning');
    });

    it('diagnostic message mentions depth and maxDepth', () => {
      const buf = buildHelloWorldBuffer();
      const processor = createProcessor({ plugins: [lintHeadingDepth({ maxDepth: 0 })] });
      const result = processor.processBuffer(buf);
      assert.match(result.diagnostics[0].message, /depth 1/);
      assert.match(result.diagnostics[0].message, /maximum of 0/);
    });

    it('diagnostic has nodeId for the heading', () => {
      const buf = buildHelloWorldBuffer();
      const processor = createProcessor({ plugins: [lintHeadingDepth({ maxDepth: 0 })] });
      const result = processor.processBuffer(buf);
      assert.equal(result.diagnostics[0].nodeId, 1);
    });
  });

  describe('flattenHeadings', () => {
    it('no mutation when h1 is within maxDepth=2', () => {
      const buf = buildHelloWorldBuffer();
      assert.equal(createProcessor({ plugins: [flattenHeadings({ maxDepth: 2 })] }).processBuffer(buf).mutationCount, 0);
    });

    it('no mutation when h1 is within maxDepth=1 (default=3)', () => {
      const buf = buildHelloWorldBuffer();
      assert.equal(createProcessor({ plugins: [flattenHeadings()] }).processBuffer(buf).mutationCount, 0);
    });

    it('records 1 Replace mutation when h1 exceeds maxDepth=0', () => {
      const buf = buildHelloWorldBuffer();
      assert.equal(createProcessor({ plugins: [flattenHeadings({ maxDepth: 0 })] }).processBuffer(buf).mutationCount, 1);
    });

    it('produces no diagnostics (only mutations)', () => {
      const buf = buildHelloWorldBuffer();
      assert.equal(createProcessor({ plugins: [flattenHeadings({ maxDepth: 0 })] }).processBuffer(buf).diagnostics.length, 0);
    });
  });

  describe('collectHeadings', () => {
    it('collects the correct heading count from buildHelloWorldBuffer', () => {
      const buf = buildHelloWorldBuffer();
      let capturedInstance: ReturnType<typeof collectHeadings.createOnce> | null = null;
      const wrapped = definePlugin({
        meta: { name: 'collect-headings-test' },
        createOnce(ctx) {
          capturedInstance = collectHeadings.createOnce(ctx);
          return capturedInstance;
        },
      });
      createProcessor({ plugins: [wrapped] }).processBuffer(buf);
      assert.equal((capturedInstance as { getHeadings(): unknown[] } | null)?.getHeadings().length, 1);
    });

    it('collected heading has depth=1', () => {
      const buf = buildHelloWorldBuffer();
      let capturedInstance: ReturnType<typeof collectHeadings.createOnce> | null = null;
      const wrapped = definePlugin({
        meta: { name: 'collect-headings-depth-test' },
        createOnce(ctx) {
          capturedInstance = collectHeadings.createOnce(ctx);
          return capturedInstance;
        },
      });
      createProcessor({ plugins: [wrapped] }).processBuffer(buf);
      const headings = (capturedInstance as { getHeadings(): { depth: number }[] } | null)?.getHeadings();
      assert.equal(headings?.[0].depth, 1);
    });

    it('accumulates headings across two processBuffer calls', () => {
      const buf = buildHelloWorldBuffer();
      let capturedInstance: ReturnType<typeof collectHeadings.createOnce> | null = null;
      const wrapped = definePlugin({
        meta: { name: 'collect-headings-multi-test' },
        createOnce(ctx) {
          capturedInstance = collectHeadings.createOnce(ctx);
          return capturedInstance;
        },
      });
      const processor = createProcessor({ plugins: [wrapped] });
      processor.processBuffer(buf);
      processor.processBuffer(buf);
      assert.equal((capturedInstance as { getHeadings(): unknown[] } | null)?.getHeadings().length, 2);
    });
  });
});
