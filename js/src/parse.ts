/**
 * Native parser binding. Calls the Rust NAPI module to parse Markdown
 * into a raw arena buffer.
 *
 * The native module is loaded lazily so the rest of the JS layer can be
 * imported without the `.node` file being present (e.g. in tests that build
 * their own buffers).
 */

import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

let nativeModule: {
  parseToBuffer(source: string): Buffer;
  parseToUint8Array(source: string): Uint8Array;
  parseToHastBuffer(source: string): Buffer;
  parseMdxToBuffer(source: string): Buffer;
  parseMdxToHastBuffer(source: string): Buffer;
  parseToHtml(source: string): string;
  parseMdxToHtml(source: string): string;
  mdastBufferToHastBuffer(buf: Buffer | Uint8Array): Buffer;
  hastBufferToHtmlStr(buf: Buffer | Uint8Array): string;
  compileMdx(source: string): string;
  compileMdxFromBuffer(buf: Buffer | Uint8Array): string;
} | null = null;

function loadNative() {
  if (nativeModule !== null) return nativeModule;
  // Try common locations for the built native module.
  const candidates = [
    join(__dirname, '..', 'tryckeri_napi.linux-x64-gnu.node'),
    join(__dirname, '..', 'tryckeri_napi.darwin-x64.node'),
    join(__dirname, '..', 'tryckeri_napi.darwin-arm64.node'),
    join(__dirname, '..', 'tryckeri_napi.win32-x64-msvc.node'),
  ];
  for (const path of candidates) {
    try {
      nativeModule = require(path);
      return nativeModule!;
    } catch {
      // try next
    }
  }
  throw new Error(
    'tryckeri native module not found. Run `cargo build --release -p tryckeri-napi` ' +
    'and copy the resulting .so/.dylib/.dll to js/ as tryckeri_napi.<platform>.node'
  );
}

/**
 * Parse Markdown source and return a raw arena buffer as Uint8Array.
 * Requires the native Rust module to be built.
 */
export function parseToBuffer(source: string): Uint8Array {
  return loadNative().parseToUint8Array(source);
}

export function parseToHastBuffer(source: string): Buffer {
  return loadNative().parseToHastBuffer(source);
}

export function mdastBufferToHastBuffer(buf: Buffer | Uint8Array): Buffer {
  return loadNative().mdastBufferToHastBuffer(buf instanceof Buffer ? buf : Buffer.from(buf));
}

export function hastBufferToHtmlStr(buf: Buffer | Uint8Array): string {
  return loadNative().hastBufferToHtmlStr(buf instanceof Buffer ? buf : Buffer.from(buf));
}

/**
 * Compile MDX source directly to JavaScript.
 * Requires the native Rust module to be built.
 */
export function compileMdx(source: string): string {
  return loadNative().compileMdx(source);
}

/**
 * Compile a pre-parsed MDAST binary buffer to MDX JavaScript output.
 * The buffer must have been produced by `parseToBuffer` or equivalent,
 * parsed with MDX constructs enabled.
 */
export function compileMdxFromBuffer(buf: Buffer | Uint8Array): string {
  return loadNative().compileMdxFromBuffer(buf instanceof Buffer ? buf : Buffer.from(buf));
}

/** Parse Markdown source to HTML in a single native call (fastest path). */
export function parseToHtml(source: string): string {
  return loadNative().parseToHtml(source);
}

/** Parse MDX source to HTML in a single native call. */
export function parseMdxToHtml(source: string): string {
  return loadNative().parseMdxToHtml(source);
}
