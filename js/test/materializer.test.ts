import { test } from 'node:test';
import assert from 'node:assert/strict';
import { ArenaReader } from '../src/arena-reader.ts';
import { DataMap } from '../src/data-map.ts';
import { materializeNode, materializeTree } from '../src/materializer.ts';
import { buildHelloWorldBuffer } from './fixtures.ts';

function setup() {
  const buf = buildHelloWorldBuffer();
  const reader = new ArenaReader(buf);
  const dataMap = new DataMap();
  return { reader, dataMap };
}

test('materializeTree returns a root node with type === "root"', () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);
  assert.equal(root.type, 'root');
});

test('root node children is a lazy getter initially', () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);
  const desc = Object.getOwnPropertyDescriptor(root, 'children');
  assert.ok(typeof desc?.get === 'function', 'children should be a lazy getter before access');
  assert.ok(!('value' in (desc ?? {})), 'children should not have a value before access');
});

test('accessing root.children returns 2 children (heading, paragraph)', () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);
  const children = root.children!;
  assert.equal(children.length, 2);
  assert.equal(children[0].type, 'heading');
  assert.equal(children[1].type, 'paragraph');
});

test('heading has depth === 1', () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);
  const heading = root.children![0];
  assert.equal(heading.depth, 1);
});

test('text child of heading has value === "Hello"', () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);
  const heading = root.children![0];
  const textNode = heading.children![0];
  assert.equal(textNode.type, 'text');
  assert.equal(textNode.value, 'Hello');
});

test('text child of paragraph has value === "World"', () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);
  const para = root.children![1];
  const textNode = para.children![0];
  assert.equal(textNode.type, 'text');
  assert.equal(textNode.value, 'World');
});

test('position data is correct: root.position.start.line === 1', () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);
  assert.equal(root.position.start.line, 1);
});

test('_nodeId is non-enumerable', () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);
  assert.ok(!Object.keys(root).includes('_nodeId'), '_nodeId should not appear in Object.keys');
  assert.equal(root._nodeId, 0);
});

test('data getter returns null when no data is set', () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);
  assert.equal(root.data, null);
});

test('setting node.data stores in dataMap', () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);
  root.data = { id: 'hello' };
  assert.ok(dataMap.has(0));
});

test('reading node.data after setting returns the value', () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);
  root.data = { id: 'hello' };
  assert.deepEqual(root.data, { id: 'hello' });
});

test('children are lazily evaluated (getter replaced by plain array after access)', () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);

  const beforeDesc = Object.getOwnPropertyDescriptor(root, 'children');
  assert.ok(typeof beforeDesc?.get === 'function', 'should be a getter before access');

  const children = root.children;
  assert.ok(Array.isArray(children));

  const afterDesc = Object.getOwnPropertyDescriptor(root, 'children');
  assert.ok(!('get' in (afterDesc ?? {})), 'should not be a getter after access');
  assert.ok('value' in (afterDesc ?? {}), 'should have a plain value after access');
});
