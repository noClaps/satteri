import { test } from 'node:test';
import assert from 'node:assert/strict';
import { runPluginsOnBuffer } from '../src/pipeline.ts';
import { DataMap } from '../src/data-map.ts';
import { buildHelloWorldBuffer } from './fixtures.ts';
import type { MdastNode } from '../src/types.ts';

function makePlugin(instance: Record<string, unknown>, name = 'test-plugin') {
  return { instance, name };
}

test('structuralMutationCount is 0 for a data-only plugin (heading-ids)', () => {
  const buffer = buildHelloWorldBuffer();

  const headingIdsPlugin = {
    heading(node: MdastNode) {
      node.data = { id: node.children?.[0]?.value ?? 'heading' };
    },
  };

  const result = runPluginsOnBuffer(buffer, [makePlugin(headingIdsPlugin)]);

  assert.equal(result.structuralMutationCount, 0, 'no structural mutations for data-only plugin');
  assert.ok(result.mutationCount >= 0, 'mutationCount is non-negative');
});

test('structuralMutationCount is 1 for a plugin that returns a replacement node', () => {
  const buffer = buildHelloWorldBuffer();

  const replacePlugin = {
    heading(node: MdastNode) {
      return { type: 'paragraph', children: node.children } as unknown as MdastNode;
    },
  };

  const result = runPluginsOnBuffer(buffer, [makePlugin(replacePlugin)]);

  assert.equal(result.structuralMutationCount, 1, 'one structural mutation (replace heading)');
});

test('mutationCount equals total mutations across all plugins', () => {
  const buffer = buildHelloWorldBuffer();

  const plugin1 = {
    heading(node: MdastNode) {
      return { type: 'paragraph', children: node.children } as unknown as MdastNode;
    },
  };

  const plugin2 = {
    heading(node: MdastNode, ctx: { removeNode(n: MdastNode): void }) {
      ctx.removeNode(node);
    },
  };

  const result = runPluginsOnBuffer(buffer, [makePlugin(plugin1, 'p1'), makePlugin(plugin2, 'p2')]);

  assert.equal(result.mutationCount, 2, 'two total mutations across two plugins');
  assert.equal(result.structuralMutationCount, 2, 'both are structural');
});

test('same buffer reference returned when no structural mutations', () => {
  const buffer = buildHelloWorldBuffer();

  const noopPlugin = {};

  const result = runPluginsOnBuffer(buffer, [makePlugin(noopPlugin)]);

  assert.strictEqual(result.buffer, buffer, 'buffer reference should be unchanged');
  assert.equal(result.structuralMutationCount, 0);
});

test('DataMap entries are visible across plugin passes (plugin 1 sets, plugin 2 reads)', () => {
  const buffer = buildHelloWorldBuffer();
  let seenIdInPlugin2: string | null = null;

  const plugin1 = {
    heading(node: MdastNode) {
      node.data = { id: 'my-heading' };
    },
  };

  const plugin2 = {
    heading(node: MdastNode) {
      seenIdInPlugin2 = (node.data as { id?: string } | null)?.id ?? null;
    },
  };

  const result = runPluginsOnBuffer(buffer, [makePlugin(plugin1, 'p1'), makePlugin(plugin2, 'p2')]);

  assert.ok(result.dataMap instanceof DataMap, 'result.dataMap is a DataMap');
  const nodeData = result.dataMap.get(1);
  assert.ok(nodeData !== null, 'node 1 should have data in DataMap');
  assert.ok('id' in nodeData!, 'id key should be present in node data');
  assert.equal(seenIdInPlugin2, 'my-heading', 'plugin 2 should read data set by plugin 1');
});

test('filename option is available in plugin fileContext', () => {
  const buffer = buildHelloWorldBuffer();
  let capturedFilename: string | null = null;

  const plugin = {
    before(fileContext: { filename: string }) {
      capturedFilename = fileContext.filename;
    },
  };

  runPluginsOnBuffer(buffer, [makePlugin(plugin)], { filename: 'my-doc.md' });

  assert.equal(capturedFilename, 'my-doc.md', 'filename should be passed to plugin context');
});

test('empty plugin list returns original buffer and zero mutations', () => {
  const buffer = buildHelloWorldBuffer();
  const result = runPluginsOnBuffer(buffer, []);

  assert.strictEqual(result.buffer, buffer, 'original buffer returned');
  assert.equal(result.mutationCount, 0);
  assert.equal(result.structuralMutationCount, 0);
  assert.equal(result.diagnostics.length, 0);
});

test('provided dataMap is used and returned in result', () => {
  const buffer = buildHelloWorldBuffer();
  const customDataMap = new DataMap();
  customDataMap.set(99, { 'pre-existing': 'yes' });

  const result = runPluginsOnBuffer(buffer, [], { dataMap: customDataMap });

  assert.strictEqual(result.dataMap, customDataMap, 'same dataMap instance');
  const nodeData = result.dataMap.get(99);
  assert.equal(nodeData?.['pre-existing'], 'yes', 'pre-existing data preserved');
});

test('diagnostics are collected from all plugins', () => {
  const buffer = buildHelloWorldBuffer();

  const plugin1 = {
    before(_fileCtx: unknown, ctx: { report(d: { message: string; severity: string }): void }) {
      ctx.report({ message: 'warning from plugin 1', severity: 'warning' });
    },
  };
  const plugin2 = {
    before(_fileCtx: unknown, ctx: { report(d: { message: string; severity: string }): void }) {
      ctx.report({ message: 'error from plugin 2', severity: 'error' });
    },
  };

  const result = runPluginsOnBuffer(buffer, [makePlugin(plugin1, 'p1'), makePlugin(plugin2, 'p2')]);

  assert.equal(result.diagnostics.length, 2, 'should have 2 diagnostics total');
  assert.ok(result.diagnostics.some(d => d.message === 'warning from plugin 1'));
  assert.ok(result.diagnostics.some(d => d.message === 'error from plugin 2'));
});
