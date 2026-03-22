import { test } from 'node:test';
import assert from 'node:assert/strict';
import { DataMap } from '../src/data-map.ts';

test('get returns null for unknown keys', () => {
  const dm = new DataMap();
  assert.equal(dm.get(0), null);
  assert.equal(dm.get(999), null);
});

test('set then get returns the value', () => {
  const dm = new DataMap();
  dm.set(1, { foo: 'bar' });
  assert.deepEqual(dm.get(1), { foo: 'bar' });
});

test('merge merges objects', () => {
  const dm = new DataMap();
  dm.set(1, { a: 1 });
  dm.merge(1, { b: 2 });
  assert.deepEqual(dm.get(1), { a: 1, b: 2 });
});

test('merge creates new entry when key does not exist', () => {
  const dm = new DataMap();
  dm.merge(5, { x: 42 });
  assert.deepEqual(dm.get(5), { x: 42 });
});

test('delete removes key', () => {
  const dm = new DataMap();
  dm.set(2, { v: 'hello' });
  dm.delete(2);
  assert.equal(dm.get(2), null);
});

test('clear empties map', () => {
  const dm = new DataMap();
  dm.set(1, { a: 'a' });
  dm.set(2, { b: 'b' });
  dm.clear();
  assert.equal(dm.size, 0);
  assert.equal(dm.get(1), null);
});

test('size property', () => {
  const dm = new DataMap();
  assert.equal(dm.size, 0);
  dm.set(1, { x: 1 });
  assert.equal(dm.size, 1);
  dm.set(2, { y: 2 });
  assert.equal(dm.size, 2);
  dm.delete(1);
  assert.equal(dm.size, 1);
});
