import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import { buildHelloWorldBuffer } from './fixtures.ts';
import { createProcessor } from '../src/processor.ts';
import { definePlugin } from '../src/plugin.ts';
import headingIds from '../src/plugins/heading-ids.ts';
import lintHeadingDepth from '../src/plugins/lint-heading-depth.ts';
import flattenHeadings from '../src/plugins/flatten-headings.ts';
import collectHeadings from '../src/plugins/collect-headings.ts';

describe('createProcessor', () => {
  it('createProcessor([]) works, processBuffer returns same buffer', () => {
    const buf = buildHelloWorldBuffer();
    const processor = createProcessor({ plugins: [] });
    const result = processor.processBuffer(buf);
    assert.strictEqual(result.buffer, buf);
    assert.equal(result.mutationCount, 0);
    assert.deepEqual(result.diagnostics, []);
  });

  it('heading-ids plugin: heading node data.id is set to "hello"', () => {
    const buf = buildHelloWorldBuffer();
    const processor = createProcessor({ plugins: [headingIds] });
    const result = processor.processBuffer(buf);
    const data = result.dataMap.get(1);
    assert.ok(data, 'dataMap should have entry for node 1');
    assert.equal(data.id, 'hello');
  });

  it('heading-ids plugin: id is a slug (lowercase, no special chars)', () => {
    const buf = buildHelloWorldBuffer();
    const processor = createProcessor({ plugins: [headingIds] });
    const result = processor.processBuffer(buf);
    const data = result.dataMap.get(1);
    assert.ok(data?.id, 'id should be set');
    assert.match(String(data!.id), /^[a-z0-9-]+$/);
  });

  it('heading-ids plugin: hProperties.id matches id', () => {
    const buf = buildHelloWorldBuffer();
    const processor = createProcessor({ plugins: [headingIds] });
    const result = processor.processBuffer(buf);
    const data = result.dataMap.get(1);
    assert.deepEqual(data!.hProperties, { id: data!.id });
  });

  it('lint-heading-depth: no diagnostics when heading is within limit', () => {
    const buf = buildHelloWorldBuffer();
    const processor = createProcessor({ plugins: [lintHeadingDepth({ maxDepth: 1 })] });
    const result = processor.processBuffer(buf);
    assert.equal(result.diagnostics.length, 0);
  });

  it('lint-heading-depth with maxDepth=0: reports a diagnostic for h1 heading', () => {
    const buf = buildHelloWorldBuffer();
    const processor = createProcessor({ plugins: [lintHeadingDepth({ maxDepth: 0 })] });
    const result = processor.processBuffer(buf);
    assert.equal(result.diagnostics.length, 1);
    assert.equal(result.diagnostics[0].severity, 'warning');
    assert.match(result.diagnostics[0].message, /depth 1 exceeds maximum of 0/);
  });

  it('flatten-headings: records a Replace mutation when heading exceeds max', () => {
    const buf = buildHelloWorldBuffer();
    const processor = createProcessor({ plugins: [flattenHeadings({ maxDepth: 0 })] });
    const result = processor.processBuffer(buf);
    assert.equal(result.mutationCount, 1);
  });

  it('flatten-headings: no mutation when heading is within max', () => {
    const buf = buildHelloWorldBuffer();
    const processor = createProcessor({ plugins: [flattenHeadings({ maxDepth: 2 })] });
    const result = processor.processBuffer(buf);
    assert.equal(result.mutationCount, 0);
  });

  it('collect-headings: after processing, getHeadings() returns 1 heading with depth=1', () => {
    const buf = buildHelloWorldBuffer();
    let capturedInstance: ReturnType<typeof collectHeadings.createOnce> | null = null;
    const wrappedCollect = definePlugin({
      meta: { name: 'collect-headings-wrapper' },
      createOnce(ctx) {
        capturedInstance = collectHeadings.createOnce(ctx);
        return capturedInstance;
      },
    });
    const processor = createProcessor({ plugins: [wrappedCollect] });
    processor.processBuffer(buf);
    assert.ok(capturedInstance, 'instance should be captured');
    const headings = (capturedInstance as { getHeadings(): { depth: number }[] }).getHeadings();
    assert.equal(headings.length, 1);
    assert.equal(headings[0].depth, 1);
  });

  it('multiple plugins run in order: heading-ids runs first, then counter sees results', () => {
    const buf = buildHelloWorldBuffer();
    let headingCallCount = 0;
    const counterPlugin = definePlugin({
      meta: { name: 'counter' },
      createOnce() {
        return {
          heading(_node: unknown) {
            headingCallCount++;
          },
        };
      },
    });
    const processor = createProcessor({ plugins: [headingIds, counterPlugin] });
    processor.processBuffer(buf);
    assert.equal(headingCallCount, 1);
  });

  it('createOnce is called once per processor, not once per processBuffer call', () => {
    const buf = buildHelloWorldBuffer();
    let createOnceCallCount = 0;
    const countingPlugin = definePlugin({
      meta: { name: 'counter' },
      createOnce(_ctx) {
        createOnceCallCount++;
        return {};
      },
    });
    const processor = createProcessor({ plugins: [countingPlugin] });
    processor.processBuffer(buf);
    processor.processBuffer(buf);
    assert.equal(createOnceCallCount, 1);
  });

  it('processBufferToTree returns a tree object with type === "root"', () => {
    const buf = buildHelloWorldBuffer();
    const processor = createProcessor({ plugins: [] });
    const result = processor.processBufferToTree(buf);
    assert.ok(result.tree, 'tree should exist');
    assert.equal(result.tree.type, 'root');
  });

  it('processBufferToTree tree has children', () => {
    const buf = buildHelloWorldBuffer();
    const processor = createProcessor({ plugins: [] });
    const result = processor.processBufferToTree(buf);
    assert.ok(Array.isArray(result.tree.children));
    assert.ok(result.tree.children!.length > 0);
  });

  it('getDiagnostics returns array (empty when no processor-level reports)', () => {
    const processor = createProcessor({ plugins: [] });
    assert.deepEqual(processor.getDiagnostics(), []);
  });

  it('createProcessor throws for invalid plugin (missing meta.name)', () => {
    assert.throws(
      () => createProcessor({ plugins: [{ meta: {} as { name: string }, createOnce() { return {}; } }] }),
      /Invalid plugin/,
    );
  });
});
