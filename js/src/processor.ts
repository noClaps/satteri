import { ProcessorContext, runPluginsOnBuffer } from './pipeline.ts';
import { ArenaReader } from './arena-reader.ts';
import { materializeTree } from './materializer.ts';
import { DataMap } from './data-map.ts';
import type { PluginDefinition } from './plugin.ts';
import type { MdastNode } from './types.ts';
import type { Diagnostic } from './visitor.ts';

export { ProcessorContext };

export interface ProcessBufferResult {
  buffer: ArrayBuffer | Uint8Array;
  dataMap: DataMap;
  diagnostics: Diagnostic[];
  mutationCount: number;
  structuralMutationCount: number;
}

export interface ProcessTreeResult {
  tree: MdastNode;
  dataMap: DataMap;
  diagnostics: Diagnostic[];
  mutationCount: number;
}

export function createProcessor({ plugins = [] }: { plugins?: PluginDefinition[] } = {}): Processor {
  return new Processor(plugins);
}

class Processor {
  readonly #pluginDefs: PluginDefinition[];
  readonly #processorCtx: ProcessorContext;
  #initializedPlugins: { instance: ReturnType<PluginDefinition['createOnce']>; name: string }[] | null = null;

  constructor(pluginDefs: PluginDefinition[]) {
    for (const def of pluginDefs) {
      if (!def.meta?.name || typeof def.createOnce !== 'function') {
        throw new Error(`Invalid plugin: ${JSON.stringify(def.meta)}`);
      }
    }
    this.#pluginDefs = pluginDefs;
    this.#processorCtx = new ProcessorContext();
  }

  #getPluginInstances(): { instance: ReturnType<PluginDefinition['createOnce']>; name: string }[] {
    if (this.#initializedPlugins === null) {
      this.#initializedPlugins = this.#pluginDefs.map(def => ({
        instance: def.createOnce(this.#processorCtx),
        name: def.meta.name,
      }));
    }
    return this.#initializedPlugins;
  }

  processBuffer(buffer: ArrayBuffer | Uint8Array, options: { filename?: string } = {}): ProcessBufferResult {
    return runPluginsOnBuffer(buffer, this.#getPluginInstances(), options);
  }

  processBufferToTree(buffer: ArrayBuffer | Uint8Array, options: { filename?: string } = {}): ProcessTreeResult {
    const result = this.processBuffer(buffer, options);
    const reader = new ArenaReader(result.buffer);
    const tree = materializeTree(reader, result.dataMap);
    return {
      tree,
      dataMap: result.dataMap,
      diagnostics: result.diagnostics,
      mutationCount: result.mutationCount,
    };
  }

  getDiagnostics(): Diagnostic[] {
    return this.#processorCtx.getDiagnostics();
  }
}
