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
  applyMutationsAndConvertToHast,
  applyMutationsAndRenderHtml,
  applyMutationsAndCompileJs,
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

interface HastPipelineResult {
  /** The buffer after all plugins (or all-but-last if pendingCommands is set). */
  buffer: Uint8Array;
  /** If the last plugin produced mutations, they're deferred here for fusion. */
  pendingCommands: Uint8Array | null;
}

/**
 * Run HAST plugins, deferring the last plugin's mutations so the caller can
 * fuse applyMutations with the final render/compile step.
 */
function runHastPlugins(hastBuf: Uint8Array, plugins: HastPluginDefinition[]): HastPipelineResult {
  if (plugins.length === 0) return { buffer: hastBuf, pendingCommands: null };

  const instances = initPlugins(plugins);
  let currentBuffer: Uint8Array = hastBuf;

  for (let i = 0; i < instances.length; i++) {
    const result = visitHast(new HastReader(currentBuffer), instances[i]!.instance, new DataMap());

    if (result.hasMutations) {
      if (i === instances.length - 1) {
        // Last plugin — defer mutations for fusion
        return { buffer: currentBuffer, pendingCommands: result.commandBuffer };
      }
      currentBuffer = applyMutations(currentBuffer, result.commandBuffer);
    }
  }

  return { buffer: currentBuffer, pendingCommands: null };
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

  let hastBuf: Uint8Array;
  if (mdastPlugins.length > 0) {
    const instances = initPlugins(mdastPlugins);
    const mdastResult = runPluginsOnBuffer(mdastBuf, instances, { deferLast: true });
    if (mdastResult.pendingCommands) {
      hastBuf = applyMutationsAndConvertToHast(
        extractBuffer(mdastResult),
        mdastResult.pendingCommands,
      );
    } else {
      hastBuf = mdastBufferToHastBuffer(extractBuffer(mdastResult));
    }
  } else {
    hastBuf = mdastBufferToHastBuffer(mdastBuf);
  }
  mdastBuf = null;

  const { buffer, pendingCommands } = runHastPlugins(hastBuf, hastPlugins);
  if (pendingCommands) {
    return applyMutationsAndRenderHtml(buffer, pendingCommands);
  }
  return hastBufferToHtmlStr(buffer);
}

export function compileMdxToJs(source: string, options: CompileOptions = {}): string {
  const { mdastPlugins = [], hastPlugins = [], optimizeStatic } = options;

  // Fast path: no plugins → single NAPI call, zero JS overhead
  if (mdastPlugins.length === 0 && hastPlugins.length === 0) {
    const mdxOptions = optimizeStatic ? { optimizeStatic } : undefined;
    return compileMdx(source, mdxOptions);
  }

  let mdastBuf: Uint8Array | null = parseMdxToBuffer(source);

  let hastBuf: Uint8Array;
  if (mdastPlugins.length > 0) {
    const instances = initPlugins(mdastPlugins);
    const mdastResult = runPluginsOnBuffer(mdastBuf, instances, { deferLast: true });
    if (mdastResult.pendingCommands) {
      hastBuf = applyMutationsAndConvertToHast(
        extractBuffer(mdastResult),
        mdastResult.pendingCommands,
      );
    } else {
      hastBuf = mdastBufferToHastBuffer(extractBuffer(mdastResult));
    }
  } else {
    hastBuf = mdastBufferToHastBuffer(mdastBuf);
  }
  mdastBuf = null;

  const mdxOptions = optimizeStatic ? { optimizeStatic } : undefined;
  const { buffer, pendingCommands } = runHastPlugins(hastBuf, hastPlugins);
  if (pendingCommands) {
    return applyMutationsAndCompileJs(buffer, pendingCommands, mdxOptions);
  }
  return compileHastBufferToJs(buffer, mdxOptions);
}
