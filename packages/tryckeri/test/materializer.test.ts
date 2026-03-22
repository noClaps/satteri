import { test, expect } from "vitest";
import { MdastReader } from "../src/mdast-reader.js";
import { DataMap } from "../src/data-map.js";
import { materializeTree } from "../src/materializer.js";
import { buildHelloWorldBuffer } from "./fixtures.js";

function setup() {
  const buf = buildHelloWorldBuffer();
  const reader = new MdastReader(buf);
  const dataMap = new DataMap();
  return { reader, dataMap };
}

test('materializeTree returns a root node with type === "root"', () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);
  expect(root.type).toBe("root");
});

test("root node children is a lazy getter initially", () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);
  const desc = Object.getOwnPropertyDescriptor(root, "children");
  expect(typeof desc?.get === "function").toBe(true);
  expect("value" in (desc ?? {})).toBe(false);
});

test("accessing root.children returns 2 children (heading, paragraph)", () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);
  const children = root.children!;
  expect(children.length).toBe(2);
  expect(children[0]!.type).toBe("heading");
  expect(children[1]!.type).toBe("paragraph");
});

test("heading has depth === 1", () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);
  const heading = root.children![0]!;
  expect(heading.depth).toBe(1);
});

test('text child of heading has value === "Hello"', () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);
  const heading = root.children![0]!;
  const textNode = heading.children![0]!;
  expect(textNode.type).toBe("text");
  expect(textNode.value).toBe("Hello");
});

test('text child of paragraph has value === "World"', () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);
  const para = root.children![1]!;
  const textNode = para.children![0]!;
  expect(textNode.type).toBe("text");
  expect(textNode.value).toBe("World");
});

test("position data is correct: root.position.start.line === 1", () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);
  expect(root.position.start.line).toBe(1);
});

test("_nodeId is non-enumerable", () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);
  expect(Object.keys(root)).not.toContain("_nodeId");
  expect(root._nodeId).toBe(0);
});

test("data getter returns null when no data is set", () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);
  expect(root.data).toBeNull();
});

test("setting node.data stores in dataMap", () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);
  root.data = { id: "hello" };
  expect(dataMap.has(0)).toBe(true);
});

test("reading node.data after setting returns the value", () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);
  root.data = { id: "hello" };
  expect(root.data).toEqual({ id: "hello" });
});

test("children are lazily evaluated (getter replaced by plain array after access)", () => {
  const { reader, dataMap } = setup();
  const root = materializeTree(reader, dataMap);

  const beforeDesc = Object.getOwnPropertyDescriptor(root, "children");
  expect(typeof beforeDesc?.get === "function").toBe(true);

  const children = root.children;
  expect(Array.isArray(children)).toBe(true);

  const afterDesc = Object.getOwnPropertyDescriptor(root, "children");
  expect("get" in (afterDesc ?? {})).toBe(false);
  expect("value" in (afterDesc ?? {})).toBe(true);
});
