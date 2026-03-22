import { test } from 'node:test';
import assert from 'node:assert/strict';
import { ArenaReader } from '../src/arena-reader.ts';
import { DataMap } from '../src/data-map.ts';
import { visitArena, MutationType } from '../src/visitor.ts';
import { buildHelloWorldBuffer } from './fixtures.ts';
import type { MdastNode } from '../src/types.ts';

function setup() {
  const buf = buildHelloWorldBuffer();
  const reader = new ArenaReader(buf);
  const dataMap = new DataMap();
  return { reader, dataMap };
}

test('visitor with no subscriptions produces no mutations, no diagnostics', () => {
  const { reader, dataMap } = setup();
  const result = visitArena(reader, {}, dataMap);
  assert.equal(result.mutations.length, 0);
  assert.equal(result.diagnostics.length, 0);
  assert.equal(result.hasMutations, false);
});

test('visiting heading nodes — callback fires once for the test doc', () => {
  const { reader, dataMap } = setup();
  let callCount = 0;
  visitArena(reader, { heading(_node: MdastNode) { callCount++; } }, dataMap);
  assert.equal(callCount, 1);
});

test('visitor callback receives correct MDAST node (type="heading", depth=1)', () => {
  const { reader, dataMap } = setup();
  let capturedNode: MdastNode | null = null;
  visitArena(reader, { heading(node: MdastNode) { capturedNode = node; } }, dataMap);
  assert.ok(capturedNode !== null);
  assert.equal(capturedNode!.type, 'heading');
  assert.equal(capturedNode!.depth, 1);
});

test('return value from visitor creates a Replace mutation', () => {
  const { reader, dataMap } = setup();
  const newNode = { type: 'paragraph', children: [] } as unknown as MdastNode;
  const result = visitArena(reader, {
    heading(_node: MdastNode) { return newNode; },
  }, dataMap);
  assert.equal(result.mutations.length, 1);
  assert.equal(result.mutations[0].type, MutationType.Replace);
  assert.equal(result.mutations[0].newNode, newNode);
});

test('context.removeNode creates a Remove mutation', () => {
  const { reader, dataMap } = setup();
  const result = visitArena(reader, {
    heading(node, context) { context.removeNode(node); },
  }, dataMap);
  assert.equal(result.mutations.length, 1);
  assert.equal(result.mutations[0].type, MutationType.Remove);
  assert.equal(result.mutations[0].nodeId, 1);
});

test('context.report creates a diagnostic entry', () => {
  const { reader, dataMap } = setup();
  const result = visitArena(reader, {
    heading(node, context) {
      context.report({ message: 'test diagnostic', node, severity: 'warning' });
    },
  }, dataMap);
  assert.equal(result.diagnostics.length, 1);
  assert.equal(result.diagnostics[0].message, 'test diagnostic');
  assert.equal(result.diagnostics[0].severity, 'warning');
  assert.equal(result.diagnostics[0].nodeId, 1);
});

test('plugin.before is called before traversal', () => {
  const { reader, dataMap } = setup();
  const order: string[] = [];
  visitArena(reader, {
    before(_context) { order.push('before'); },
    heading(_node: MdastNode) { order.push('heading'); },
  }, dataMap);
  assert.equal(order[0], 'before');
  assert.equal(order[1], 'heading');
});

test('plugin.after is called after traversal', () => {
  const { reader, dataMap } = setup();
  const order: string[] = [];
  visitArena(reader, {
    heading(_node: MdastNode) { order.push('heading'); },
    after(_context) { order.push('after'); },
  }, dataMap);
  assert.equal(order[0], 'heading');
  assert.equal(order[1], 'after');
});

test('transformRoot gets the full materialized root', () => {
  const { reader, dataMap } = setup();
  let capturedRoot: MdastNode | null = null;
  visitArena(reader, {
    transformRoot(root, _context) { capturedRoot = root; },
  }, dataMap);
  assert.ok(capturedRoot !== null);
  assert.equal(capturedRoot!.type, 'root');
  assert.equal(capturedRoot!._nodeId, 0);
});

test('multiple subscribed types — all fire', () => {
  const { reader, dataMap } = setup();
  const fired: string[] = [];
  visitArena(reader, {
    heading(_node: MdastNode) { fired.push('heading'); },
    text(_node: MdastNode) { fired.push('text'); },
    paragraph(_node: MdastNode) { fired.push('paragraph'); },
  }, dataMap);
  assert.ok(fired.includes('heading'));
  assert.ok(fired.includes('paragraph'));
  assert.equal(fired.filter(x => x === 'text').length, 2);
});

test('non-subscribed types are not materialized via getNode', () => {
  const { reader, dataMap } = setup();
  let getNodeCalls = 0;
  const proxyReader = new Proxy(reader, {
    get(target, prop) {
      if (prop === 'getNode') {
        return function (...args: Parameters<typeof target.getNode>) {
          getNodeCalls++;
          return target.getNode(...args);
        };
      }
      const val = (target as unknown as Record<string | symbol, unknown>)[prop];
      return typeof val === 'function' ? val.bind(target) : val;
    },
  });
  visitArena(proxyReader as ArenaReader, { heading(_node: MdastNode) {} }, dataMap);
  assert.equal(getNodeCalls, 1, `Expected getNode called 1 time, got ${getNodeCalls}`);
});

test('context.source returns the source text', () => {
  const { reader, dataMap } = setup();
  let capturedSource: string | null = null;
  visitArena(reader, { before(context) { capturedSource = context.source; } }, dataMap);
  assert.equal(capturedSource, '# Hello\n\nWorld');
});

test('hasMutations is false when no mutations, true when there are mutations', () => {
  const { reader, dataMap } = setup();

  const noMutResult = visitArena(reader, { heading(_node: MdastNode) {} }, dataMap);
  assert.equal(noMutResult.hasMutations, false);

  const mutResult = visitArena(reader, {
    heading(node, context) { context.removeNode(node); },
  }, dataMap);
  assert.equal(mutResult.hasMutations, true);
});
