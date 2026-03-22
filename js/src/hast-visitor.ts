import { materializeHastNode, type HastNode } from './hast-materializer.ts';
import { HastArenaReader, HAST_ROOT, HAST_ELEMENT, HAST_TEXT, HAST_COMMENT, HAST_RAW } from './hast-reader.ts';
import type { DataMap } from './data-map.ts';

export interface Mutation {
  type: 'replace' | 'remove' | 'setProperty';
  nodeId: number;
  newNode?: HastNode;
  key?: string;
  value?: unknown;
}

export interface Diagnostic {
  message: string;
  nodeId?: number;
  severity: 'error' | 'warning' | 'info';
}

export interface HastVisitorContext {
  removeNode(node: HastNode): void;
  replaceNode(node: HastNode, newNode: HastNode): void;
  setProperty(node: HastNode, key: string, value: unknown): void;
  report(opts: { message: string; node?: HastNode; severity?: 'error' | 'warning' | 'info' }): void;
  getMutations(): Mutation[];
  getDiagnostics(): Diagnostic[];
}

class HastVisitorContextImpl implements HastVisitorContext {
  readonly #mutations: Mutation[] = [];
  readonly #diagnostics: Diagnostic[] = [];

  removeNode(node: HastNode): void {
    this.#mutations.push({ type: 'remove', nodeId: node._nodeId });
  }

  replaceNode(node: HastNode, newNode: HastNode): void {
    this.#mutations.push({ type: 'replace', nodeId: node._nodeId, newNode });
  }

  setProperty(node: HastNode, key: string, value: unknown): void {
    this.#mutations.push({ type: 'setProperty', nodeId: node._nodeId, key, value });
  }

  report({ message, node, severity = 'error' }: { message: string; node?: HastNode; severity?: 'error' | 'warning' | 'info' }): void {
    this.#diagnostics.push({ message, nodeId: node?._nodeId, severity });
  }

  getMutations(): Mutation[] { return this.#mutations; }
  getDiagnostics(): Diagnostic[] { return this.#diagnostics; }
}

export interface HastVisitorInstance {
  before?(ctx: HastVisitorContext): void;
  after?(ctx: HastVisitorContext): void;
  transformRoot?(root: HastNode, ctx: HastVisitorContext): HastNode | void;
  element?(node: HastNode, ctx: HastVisitorContext): HastNode | void;
  text?(node: HastNode, ctx: HastVisitorContext): HastNode | void;
  comment?(node: HastNode, ctx: HastVisitorContext): HastNode | void;
  raw?(node: HastNode, ctx: HastVisitorContext): HastNode | void;
  doctype?(node: HastNode, ctx: HastVisitorContext): HastNode | void;
}

export interface VisitResult {
  mutations: Mutation[];
  diagnostics: Diagnostic[];
  hasMutations: boolean;
}

// Map from node_type number to visitor method name
const TYPE_TO_METHOD: Record<number, keyof HastVisitorInstance> = {
  [HAST_ROOT]: 'transformRoot',
  [HAST_ELEMENT]: 'element',
  [HAST_TEXT]: 'text',
  [HAST_COMMENT]: 'comment',
  [HAST_RAW]: 'raw',
};

/**
 * Walk a HAST binary arena and dispatch to visitor methods.
 */
export function visitHastArena(
  reader: HastArenaReader,
  plugin: HastVisitorInstance,
  dataMap: DataMap
): VisitResult {
  const ctx = new HastVisitorContextImpl();
  const mutations: Mutation[] = [];

  plugin.before?.(ctx);

  if (typeof plugin.transformRoot === 'function') {
    // Full materialization path via transformRoot
    const root = materializeHastNode(reader, 0, dataMap);
    const result = plugin.transformRoot(root, ctx);
    if (result != null) {
      mutations.push({ type: 'replace', nodeId: 0, newNode: result });
    }
  } else {
    // Fast path: walk raw bytes, only materialize on subscription match
    const nodeCount = reader.nodeCount;
    const stack: number[] = [0];

    while (stack.length > 0) {
      const nodeId = stack.pop()!;
      const nodeType = reader.getNodeType(nodeId);
      const methodName = TYPE_TO_METHOD[nodeType];

      if (methodName && methodName !== 'transformRoot') {
        const fn = plugin[methodName] as ((node: HastNode, ctx: HastVisitorContext) => HastNode | void) | undefined;
        if (typeof fn === 'function') {
          const node = materializeHastNode(reader, nodeId, dataMap);
          const result = fn.call(plugin, node, ctx);
          if (result != null) {
            mutations.push({ type: 'replace', nodeId, newNode: result });
          }
        }
      }

      const childIds = reader.getChildIds(nodeId);
      for (let i = childIds.length - 1; i >= 0; i--) {
        stack.push(childIds[i]);
      }
    }
  }

  plugin.after?.(ctx);

  const allMutations = [...mutations, ...ctx.getMutations()];
  const diagnostics = ctx.getDiagnostics();

  return {
    mutations: allMutations,
    diagnostics,
    hasMutations: allMutations.length > 0,
  };
}
