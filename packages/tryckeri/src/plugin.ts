import type { MdastPluginInstance } from "./mdast/mdast-visitor.js";
import type { HastVisitorInstance } from "./hast/hast-visitor.js";
import type { ProcessorContext } from "./pipeline.js";

export interface MdastPluginDefinition {
  name: string;
  createOnce(context: ProcessorContext): MdastPluginInstance;
}

export interface HastPluginDefinition {
  name: string;
  createOnce(context: ProcessorContext): HastVisitorInstance;
}

export function defineMdastPlugin(definition: MdastPluginDefinition): MdastPluginDefinition {
  if (!definition.name) {
    throw new Error("Plugin definition must have a name");
  }
  if (typeof definition.createOnce !== "function") {
    throw new Error("Plugin definition must have a createOnce function");
  }
  return definition;
}

export function defineHastPlugin(definition: HastPluginDefinition): HastPluginDefinition {
  if (!definition.name) {
    throw new Error("Plugin definition must have a name");
  }
  if (typeof definition.createOnce !== "function") {
    throw new Error("Plugin definition must have a createOnce function");
  }
  return definition;
}
