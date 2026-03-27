/**
 * Top-level compile functions — the primary public API.
 */

import { DataMap } from "./data-map.js";
import { HastReader } from "./hast-reader.js";
import { visitHast } from "./hast-visitor.js";
import { runPluginsOnBuffer, ProcessorContext } from "./pipeline.js";
import type { MdastPluginDefinition, HastPluginDefinition } from "./plugin.js";
import {
  parseToBuffer,
  parseMdxToBuffer,
  mdastBufferToHastBuffer,
  hastBufferToHtmlStr,
  compileHastBufferToJs,
  applyMutations,
} from "../index.js";

// ---------------------------------------------------------------------------
// Plugin initialization
// ---------------------------------------------------------------------------

function initPlugins<T>(
  plugins: { name: string; createOnce(ctx: ProcessorContext): T }[],
): { instance: T; name: string }[] {
  const ctx = new ProcessorContext();
  return plugins.map((def) => ({
    instance: def.createOnce(ctx),
    name: def.name,
  }));
}

// ---------------------------------------------------------------------------
// HAST plugin runner
// ---------------------------------------------------------------------------

function runHastPlugins(
  hastBuf: Uint8Array,
  plugins: HastPluginDefinition[],
): Uint8Array {
  if (plugins.length === 0) return hastBuf;

  const instances = initPlugins(plugins);
  let currentBuffer: Uint8Array = hastBuf;

  for (const { instance } of instances) {
    const reader = new HastReader(currentBuffer);
    const dataMap = new DataMap();
    const result = visitHast(reader, instance, dataMap);

    if (result.hasMutations) {
      currentBuffer = applyMutations(currentBuffer, result.commandBuffer);
    }
  }

  return currentBuffer;
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

export interface CompileOptions {
  mdastPlugins?: MdastPluginDefinition[];
  hastPlugins?: HastPluginDefinition[];
}

export function compileMarkdownToHtml(
  source: string,
  options: CompileOptions = {},
): string {
  const { mdastPlugins = [], hastPlugins = [] } = options;

  let mdastBuf: Uint8Array = parseToBuffer(source);

  if (mdastPlugins.length > 0) {
    const instances = initPlugins(mdastPlugins);
    const result = runPluginsOnBuffer(mdastBuf, instances);
    mdastBuf =
      result.buffer instanceof Uint8Array
        ? result.buffer
        : new Uint8Array(result.buffer);
  }

  let hastBuf = mdastBufferToHastBuffer(mdastBuf);
  hastBuf = runHastPlugins(hastBuf, hastPlugins);

  return hastBufferToHtmlStr(hastBuf);
}

export function compileMdxToJs(
  source: string,
  options: CompileOptions = {},
): string {
  const { mdastPlugins = [], hastPlugins = [] } = options;

  let mdastBuf: Uint8Array = parseMdxToBuffer(source);

  if (mdastPlugins.length > 0) {
    const instances = initPlugins(mdastPlugins);
    const result = runPluginsOnBuffer(mdastBuf, instances);
    mdastBuf =
      result.buffer instanceof Uint8Array
        ? result.buffer
        : new Uint8Array(result.buffer);
  }

  let hastBuf = mdastBufferToHastBuffer(mdastBuf);
  hastBuf = runHastPlugins(hastBuf, hastPlugins);

  return compileHastBufferToJs(hastBuf);
}
