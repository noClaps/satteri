import { test } from 'node:test';
import assert from 'node:assert/strict';
import { ArenaReader, NodeType, NodeTypeName } from '../src/arena-reader.ts';
import { buildHelloWorldBuffer, buildTestBuffer } from './fixtures.ts';

test('NodeType constants', () => {
  assert.equal(NodeType.Root, 0);
  assert.equal(NodeType.Heading, 2);
  assert.equal(NodeType.Text, 10);
  assert.equal(NodeType.Yaml, 25);
  assert.equal(NodeType.Toml, 26);
  assert.equal(NodeType.Math, 27);
  assert.equal(NodeType.InlineMath, 28);
  assert.equal(NodeTypeName[0], 'Root');
  assert.equal(NodeTypeName[2], 'Heading');
  assert.equal(NodeTypeName[10], 'Text');
  assert.equal(NodeTypeName[25], 'Yaml');
});

test('ArenaReader rejects invalid magic', () => {
  const buf = new ArrayBuffer(44);
  assert.throws(() => new ArenaReader(buf), /bad magic/);
});

test('ArenaReader rejects wrong version', () => {
  const buf = new ArrayBuffer(44);
  const view = new DataView(buf);
  view.setUint32(0, 0x5241444d, true); // correct "MDAR" magic
  view.setUint32(4, 99, true);
  assert.throws(() => new ArenaReader(buf), /version/);
});

test('ArenaReader reads node count', () => {
  const buf = buildHelloWorldBuffer();
  const reader = new ArenaReader(buf);
  assert.equal(reader.nodeCount, 5);
});

test('ArenaReader reads root node', () => {
  const buf = buildHelloWorldBuffer();
  const reader = new ArenaReader(buf);
  const root = reader.getNode(0);
  assert.equal(root.type, NodeType.Root);
  assert.equal(root.typeName, 'Root');
  assert.equal(root.childrenCount, 2);
  assert.equal(root.position.start.line, 1);
  assert.equal(root.position.start.column, 1);
});

test('ArenaReader reads heading node', () => {
  const buf = buildHelloWorldBuffer();
  const reader = new ArenaReader(buf);
  const heading = reader.getNode(1);
  assert.equal(heading.type, NodeType.Heading);
  assert.equal(heading.childrenCount, 1);
  assert.equal(reader.getHeadingDepth(1), 1);
});

test('ArenaReader reads text values', () => {
  const buf = buildHelloWorldBuffer();
  const reader = new ArenaReader(buf);
  assert.equal(reader.getTextValue(3), 'Hello');
  assert.equal(reader.getTextValue(4), 'World');
});

test('ArenaReader getNodeType fast path', () => {
  const buf = buildHelloWorldBuffer();
  const reader = new ArenaReader(buf);
  assert.equal(reader.getNodeType(0), NodeType.Root);
  assert.equal(reader.getNodeType(1), NodeType.Heading);
  assert.equal(reader.getNodeType(2), NodeType.Paragraph);
  assert.equal(reader.getNodeType(3), NodeType.Text);
});

test('ArenaReader getChildIds', () => {
  const buf = buildHelloWorldBuffer();
  const reader = new ArenaReader(buf);
  assert.deepEqual(reader.getChildIds(0), [1, 2]);
  assert.deepEqual(reader.getChildIds(1), [3]);
  assert.deepEqual(reader.getChildIds(2), [4]);
  assert.deepEqual(reader.getChildIds(3), []);
});

test('ArenaReader.walk visits all nodes', () => {
  const buf = buildHelloWorldBuffer();
  const reader = new ArenaReader(buf);
  const visited: { nodeId: number; nodeType: number }[] = [];
  reader.walk((nodeId, nodeType) => { visited.push({ nodeId, nodeType }); });
  assert.equal(visited.length, 5);
  assert.equal(visited[0].nodeId, 0);
});

test('ArenaReader.walk skip children', () => {
  const buf = buildHelloWorldBuffer();
  const reader = new ArenaReader(buf);
  const visited: number[] = [];
  reader.walk((nodeId, nodeType) => {
    visited.push(nodeId);
    if (nodeType === NodeType.Heading) return false;
  });
  assert.ok(visited.includes(0));
  assert.ok(visited.includes(1));
  assert.ok(!visited.includes(3)); // Text "Hello" skipped
  assert.ok(visited.includes(2));
  assert.ok(visited.includes(4));
});

test('ArenaReader getSource', () => {
  const buf = buildHelloWorldBuffer();
  const reader = new ArenaReader(buf);
  assert.equal(reader.getSource(), '# Hello\n\nWorld');
});

test('ArenaReader accepts Uint8Array', () => {
  const buf = buildHelloWorldBuffer();
  const u8 = new Uint8Array(buf);
  const reader = new ArenaReader(u8);
  assert.equal(reader.nodeCount, 5);
  assert.equal(reader.getTextValue(3), 'Hello');
});

test('ArenaReader getTypeData returns empty for nodes without data', () => {
  const buf = buildHelloWorldBuffer();
  const reader = new ArenaReader(buf);
  assert.equal(reader.getTypeData(2).length, 0); // Paragraph has no data
});

test('ArenaReader out of range node throws', () => {
  const buf = buildHelloWorldBuffer();
  const reader = new ArenaReader(buf);
  assert.throws(() => reader.getNode(99), /out of range/);
});

test('getListData reads correct layout: start(u32)@0, ordered(bool)@4, spread(bool)@5', () => {
  // Build a minimal list buffer to verify layout offsets
  // ListData #[repr(C)]: start(0..4), ordered(4), spread(5), _pad(6..8)
  const typeData = new Uint8Array(8);
  const view = new DataView(typeData.buffer);
  view.setUint32(0, 42, true); // start = 42
  typeData[4] = 1;              // ordered = true
  typeData[5] = 1;              // spread = true

  const buf = buildTestBuffer({
    source: '',
    nodes: [
      { id: 0, type: 0, childrenStart: 0, childrenCount: 1, dataOffset: 0, dataLen: 0 },
      { id: 1, type: 5, childrenStart: 0, childrenCount: 0, dataOffset: 0, dataLen: 8 },
    ],
    children: [1],
    typeData,
  });
  const reader = new ArenaReader(buf);
  const d = reader.getListData(1);
  assert.equal(d.start, 42);
  assert.equal(d.ordered, true);
  assert.equal(d.spread, true);
});
