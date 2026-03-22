import { materializeNode, TYPE_NAMES } from './materializer.ts';
import type { MdastNode } from './types.ts';
import type { ArenaReader } from './arena-reader.ts';
import type { DataMap } from './data-map.ts';

export const MutationType = {
  Replace: 'replace',
  Remove: 'remove',
  InsertBefore: 'insertBefore',
  InsertAfter: 'insertAfter',
  Wrap: 'wrap',
  PrependChild: 'prependChild',
  AppendChild: 'appendChild',
  SetProperty: 'setProperty',
} as const;

export type MutationTypeValue = (typeof MutationType)[keyof typeof MutationType];

export interface Mutation {
  type: MutationTypeValue;
  nodeId: number;
  newNode?: MdastNode;
  key?: string;
  value?: unknown;
}

export interface Diagnostic {
  message: string;
  nodeId?: number;
  position?: MdastNode['position'];
  severity: 'error' | 'warning' | 'info';
}

const VISITOR_KEYS = new Set([
  'root', 'paragraph', 'heading', 'thematicBreak', 'blockquote', 'list', 'listItem',
  'html', 'code', 'definition', 'text', 'emphasis', 'strong', 'inlineCode', 'break',
  'link', 'image', 'linkReference', 'imageReference', 'footnoteDefinition',
  'footnoteReference', 'table', 'tableRow', 'tableCell', 'delete',
  'yaml', 'toml', 'math', 'inlineMath',
  'mdxJsxFlowElement', 'mdxJsxTextElement', 'mdxFlowExpression',
  'mdxTextExpression', 'mdxjsEsm',
]);

export class VisitorContext {
  readonly #mutations: Mutation[] = [];
  readonly #diagnostics: Diagnostic[] = [];
  readonly #reader: ArenaReader;
  readonly #dataMap: DataMap;
  readonly #rootId: number = 0;

  constructor(reader: ArenaReader, dataMap: DataMap) {
    this.#reader = reader;
    this.#dataMap = dataMap;
  }

  removeNode(node: MdastNode): void {
    this.#mutations.push({ type: MutationType.Remove, nodeId: node._nodeId });
  }

  insertBefore(node: MdastNode, newNode: MdastNode): void {
    this.#mutations.push({ type: MutationType.InsertBefore, nodeId: node._nodeId, newNode });
  }

  insertAfter(node: MdastNode, newNode: MdastNode): void {
    this.#mutations.push({ type: MutationType.InsertAfter, nodeId: node._nodeId, newNode });
  }

  wrapNode(node: MdastNode, parentNode: MdastNode): void {
    this.#mutations.push({ type: MutationType.Wrap, nodeId: node._nodeId, newNode: parentNode });
  }

  prependChild(node: MdastNode, childNode: MdastNode): void {
    this.#mutations.push({ type: MutationType.PrependChild, nodeId: node._nodeId, newNode: childNode });
  }

  appendChild(node: MdastNode, childNode: MdastNode): void {
    this.#mutations.push({ type: MutationType.AppendChild, nodeId: node._nodeId, newNode: childNode });
  }

  replaceNode(node: MdastNode, newNode: MdastNode): void {
    this.#mutations.push({ type: MutationType.Replace, nodeId: node._nodeId, newNode });
  }

  setProperty(node: MdastNode, key: string, value: unknown): void {
    this.#mutations.push({ type: MutationType.SetProperty, nodeId: node._nodeId, key, value });
  }

  report({ message, node, severity = 'error' }: { message: string; node?: MdastNode; severity?: 'error' | 'warning' | 'info' }): void {
    this.#diagnostics.push({
      message,
      nodeId: node?._nodeId,
      position: node?.position,
      severity,
    });
  }

  get root(): MdastNode {
    return materializeNode(this.#reader, this.#rootId, this.#dataMap);
  }

  get source(): string {
    return this.#reader.getSource();
  }

  getMutations(): Mutation[] { return this.#mutations; }
  getDiagnostics(): Diagnostic[] { return this.#diagnostics; }
}

export interface PluginInstance {
  before?(context: VisitorContext): void;
  after?(context: VisitorContext): void;
  transformRoot?(root: MdastNode, context: VisitorContext): MdastNode | undefined | null;
  [nodeTypeName: string]: unknown;
}

export interface VisitResult {
  mutations: Mutation[];
  diagnostics: Diagnostic[];
  hasMutations: boolean;
}

/**
 * Walk the arena and dispatch to plugin visitor functions.
 */
export function visitArena(reader: ArenaReader, plugin: PluginInstance, dataMap: DataMap): VisitResult {
  const context = new VisitorContext(reader, dataMap);

  plugin.before?.(context);

  const mutations: Mutation[] = [];

  if (typeof plugin.transformRoot === 'function') {
    // Full materialization path
    const root = materializeNode(reader, 0, dataMap);
    const result = plugin.transformRoot(root, context);
    if (result !== undefined && result !== null) {
      mutations.push({ type: MutationType.Replace, nodeId: 0, newNode: result });
    }
  } else {
    // Fast path: walk raw bytes, only materialize subscribed node types

    // Build reverse map: numeric type → visitor function
    const TYPE_TO_VISITOR = new Map<number, (node: MdastNode, context: VisitorContext) => MdastNode | undefined | null>();
    for (const [name, fn] of Object.entries(plugin)) {
      if (VISITOR_KEYS.has(name) && typeof fn === 'function') {
        for (const [num, typeName] of Object.entries(TYPE_NAMES)) {
          if (typeName === name) {
            TYPE_TO_VISITOR.set(Number(num), fn as (node: MdastNode, context: VisitorContext) => MdastNode | undefined | null);
            break;
          }
        }
      }
    }

    // Walk raw buffer — only type-check each node, materialize only on subscription match
    const stack: number[] = [0];
    while (stack.length > 0) {
      const nodeId = stack.pop()!;
      const nodeType = reader.getNodeType(nodeId);

      const visitor = TYPE_TO_VISITOR.get(nodeType);
      if (visitor) {
        const node = materializeNode(reader, nodeId, dataMap);
        const result = visitor.call(plugin, node, context);
        if (result !== undefined && result !== null) {
          mutations.push({ type: MutationType.Replace, nodeId, newNode: result });
        }
      }

      const childIds = reader.getChildIds(nodeId);
      for (let i = childIds.length - 1; i >= 0; i--) {
        stack.push(childIds[i]);
      }
    }
  }

  plugin.after?.(context);

  const allMutations = [...mutations, ...context.getMutations()];
  const diagnostics = context.getDiagnostics();

  return {
    mutations: allMutations,
    diagnostics,
    hasMutations: allMutations.length > 0,
  };
}
