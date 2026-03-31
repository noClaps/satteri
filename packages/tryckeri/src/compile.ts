/**
 * Top-level compile functions — the primary public API.
 *
 * When no plugins are provided, these functions use the fast pure-Rust path
 * (single NAPI call, zero JS overhead). Plugins trigger the full JS pipeline.
 */

import { DataMap } from "./data-map.js";
import { HastReader } from "./hast/hast-reader.js";
import { visitHast } from "./hast/hast-visitor.js";
import { runPluginsOnBuffer, ProcessorContext } from "./pipeline.js";
import type { MdastPluginDefinition, HastPluginDefinition } from "./plugin.js";
import {
  parseToBuffer,
  parseToHtml,
  parseMdxToBuffer,
  mdastBufferToHastBuffer,
  hastBufferToHtmlStr,
  compileMdx,
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

/** Extract just the buffer from a RunResult, discarding dataMap/diagnostics references. */
function extractBuffer(result: { buffer: ArrayBuffer | Uint8Array }): Uint8Array {
  return result.buffer instanceof Uint8Array ? result.buffer : new Uint8Array(result.buffer);
}

// ---------------------------------------------------------------------------
// HAST plugin runner
// ---------------------------------------------------------------------------

function runHastPlugins(hastBuf: Uint8Array, plugins: HastPluginDefinition[]): Uint8Array {
  if (plugins.length === 0) return hastBuf;

  const instances = initPlugins(plugins);
  let currentBuffer: Uint8Array = hastBuf;

  for (const { instance } of instances) {
    // Scope reader/dataMap so they don't pin the buffer after this iteration
    const result = visitHast(new HastReader(currentBuffer), instance, new DataMap());

    if (result.hasMutations) {
      currentBuffer = applyMutations(currentBuffer, result.commandBuffer);
    }
  }

  return currentBuffer;
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/** Configuration for static subtree collapsing during MDX compilation. */
export interface OptimizeStaticConfig {
  /** Component/element name to wrap collapsed HTML in (e.g. "Fragment", "div"). */
  component: string;
  /** Prop name for the HTML string (e.g. "set:html", "dangerouslySetInnerHTML"). */
  prop: string;
  /** If true, prop value is wrapped as `{ __html: "..." }` (React-style). Default: false. */
  wrapPropValue?: boolean;
  /** Element tag names to exclude from collapsing (e.g. ["h1", "p"]). */
  ignoreElements?: string[];
}

export interface CompileOptions {
  mdastPlugins?: MdastPluginDefinition[];
  hastPlugins?: HastPluginDefinition[];
  /**
   * When set, fully-static subtrees are collapsed into raw HTML strings
   * instead of nested `_jsx()` calls, reducing JS output size.
   */
  optimizeStatic?: OptimizeStaticConfig;
}

export function compileMarkdownToHtml(source: string, options: CompileOptions = {}): string {
  const { mdastPlugins = [], hastPlugins = [] } = options;

  // Fast path: no plugins → single NAPI call, zero JS overhead
  if (mdastPlugins.length === 0 && hastPlugins.length === 0) {
    return parseToHtml(source);
  }

  let mdastBuf: Uint8Array | null = parseToBuffer(source);

  if (mdastPlugins.length > 0) {
    const instances = initPlugins(mdastPlugins);
    mdastBuf = extractBuffer(runPluginsOnBuffer(mdastBuf, instances));
  }

  const hastBuf = mdastBufferToHastBuffer(mdastBuf);
  mdastBuf = null;

  return hastBufferToHtmlStr(runHastPlugins(hastBuf, hastPlugins));
}

export function compileMdxToJs(source: string, options: CompileOptions = {}): string {
  const { mdastPlugins = [], hastPlugins = [], optimizeStatic } = options;

  // Fast path: no plugins → single NAPI call, zero JS overhead
  if (mdastPlugins.length === 0 && hastPlugins.length === 0) {
    const mdxOptions = optimizeStatic ? { optimizeStatic } : undefined;
    return compileMdx(source, mdxOptions);
  }

  let mdastBuf: Uint8Array | null = parseMdxToBuffer(source);

  if (mdastPlugins.length > 0) {
    const instances = initPlugins(mdastPlugins);
    mdastBuf = extractBuffer(runPluginsOnBuffer(mdastBuf, instances));
  }

  const hastBuf = mdastBufferToHastBuffer(mdastBuf);
  mdastBuf = null;

  const mdxOptions = optimizeStatic ? { optimizeStatic } : undefined;
  return compileHastBufferToJs(runHastPlugins(hastBuf, hastPlugins), mdxOptions);
}
