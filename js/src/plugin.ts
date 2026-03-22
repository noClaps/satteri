import type { PluginInstance, VisitorContext } from './visitor.ts';
import type { MdastNode } from './types.ts';
import type { ProcessorContext } from './pipeline.ts';

export interface PluginMeta {
  name: string;
  version?: string;
  description?: string;
}

export interface PluginDefinition {
  meta: PluginMeta;
  createOnce(context: ProcessorContext): PluginInstance;
}

/**
 * Define a plugin. Returns the definition unchanged (identity function),
 * but enforces the plugin contract and provides type documentation.
 */
export function definePlugin(definition: PluginDefinition): PluginDefinition {
  if (!definition.meta?.name) {
    throw new Error('Plugin definition must have a meta.name');
  }
  if (typeof definition.createOnce !== 'function') {
    throw new Error('Plugin definition must have a createOnce function');
  }
  return definition;
}

export type { PluginInstance, VisitorContext, MdastNode };
