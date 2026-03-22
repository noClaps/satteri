import { visitMdast } from "./visitor.js";
import { DataMap } from "./data-map.js";
import { MdastReader } from "./mdast-reader.js";
import { materializeNode } from "./materializer.js";
import type { PluginDefinition } from "./plugin.js";
import type { MdastNode } from "./types.js";
import type { Mutation, Diagnostic } from "./visitor.js";

export class ProcessorContext {
  readonly #diagnostics: Diagnostic[] = [];

  report(diagnostic: Diagnostic): void {
    this.#diagnostics.push(diagnostic);
  }

  getDiagnostics(): Diagnostic[] {
    return [...this.#diagnostics];
  }
}

export interface FileContext {
  source: string;
  filename: string;
  get root(): MdastNode;
}

export interface RunResult {
  buffer: ArrayBuffer | Uint8Array;
  dataMap: DataMap;
  diagnostics: Diagnostic[];
  mutationCount: number;
  structuralMutationCount: number;
}

/**
 * Process one arena buffer through an ordered list of initialized plugin instances.
 */
export function runPluginsOnBuffer(
  buffer: ArrayBuffer | Uint8Array,
  pluginInstances: { instance: ReturnType<PluginDefinition["createOnce"]>; name: string }[],
  { filename = "<unknown>", dataMap }: { filename?: string; dataMap?: DataMap } = {},
): RunResult {
  const dm = dataMap ?? new DataMap();
  const allDiagnostics: Diagnostic[] = [];
  let totalMutations = 0;
  let structuralMutations = 0;
  let currentBuffer = buffer;

  for (const { instance, name: _name } of pluginInstances) {
    const reader = new MdastReader(currentBuffer);

    const fileContext: FileContext = {
      source: reader.getSource(),
      filename,
      get root() {
        return materializeNode(reader, 0, dm);
      },
    };

    const wrappedPlugin = wrapInstance(instance, fileContext);
    const result = visitMdast(reader, wrappedPlugin, dm);
    allDiagnostics.push(...result.diagnostics);
    totalMutations += result.mutations.length;

    // Structural mutations require arena rebuild (Phase 8+).
    // SetProperty mutations are data-only — already applied via DataMap.
    const structural = result.mutations.filter((m: Mutation) => m.type !== "setProperty");
    structuralMutations += structural.length;

    if (structural.length > 0) {
      currentBuffer = applyMutationsJS(reader, currentBuffer, structural, dm);
    }
  }

  return {
    buffer: currentBuffer,
    dataMap: dm,
    diagnostics: allDiagnostics,
    mutationCount: totalMutations,
    structuralMutationCount: structuralMutations,
  };
}

function wrapInstance(
  instance: ReturnType<PluginDefinition["createOnce"]>,
  fileContext: FileContext,
): ReturnType<PluginDefinition["createOnce"]> {
  const wrapped: Record<string, unknown> = {};

  for (const [key, val] of Object.entries(instance as Record<string, unknown>)) {
    if (key !== "before" && key !== "after" && key !== "transformRoot") {
      if (typeof val === "function") {
        wrapped[key] = val;
      }
    }
  }

  const inst = instance as Record<string, unknown>;
  if (typeof inst.before === "function") {
    wrapped.before = (visitorContext: unknown) =>
      (inst.before as (fc: FileContext, vc: unknown) => void)(fileContext, visitorContext);
  }
  if (typeof inst.after === "function") {
    wrapped.after = (visitorContext: unknown) =>
      (inst.after as (fc: FileContext, vc: unknown) => void)(fileContext, visitorContext);
  }
  if (typeof inst.transformRoot === "function") {
    wrapped.transformRoot = (root: MdastNode, visitorContext: unknown) =>
      (inst.transformRoot as (r: MdastNode, fc: FileContext, vc: unknown) => MdastNode | undefined)(
        root,
        fileContext,
        visitorContext,
      );
  }

  return wrapped as ReturnType<PluginDefinition["createOnce"]>;
}

function applyMutationsJS(
  _reader: MdastReader,
  buffer: ArrayBuffer | Uint8Array,
  _mutations: Mutation[],
  _dataMap: DataMap,
): ArrayBuffer | Uint8Array {
  // Phase 6: return same buffer. Structural mutations require Rust arena rebuild (Phase 8+).
  return buffer;
}
