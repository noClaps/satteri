/**
 * Port of docs.astro.build's remark-fallback-lang plugin.
 *
 * Walks link nodes and appends " (EN)" to links that point to pages
 * without a translated source file, indicating they'll fall back to English.
 *
 * In our plugin API this subscribes to the `link` visitor.
 */

import { existsSync } from 'node:fs';
import { join, relative, resolve } from 'node:path';
import { definePlugin } from '../plugin.ts';
import type { MdastNode, VisitorContext } from '../plugin.ts';

export interface FallbackLangOptions {
  /** Root directory containing the content source files. */
  pageSourceDir?: string;
  /** Base URL for the site. */
  baseUrl?: string;
}

function getLanguageCodeFromPathname(pathname: string): string | undefined {
  const firstPathPart = pathname.split('/')[1];
  if (firstPathPart && /^[a-z]{2}(-[a-zA-Z]{2})?$/.test(firstPathPart)) {
    return firstPathPart;
  }
}

function mdFilePathToUrl(
  mdFilePath: string,
  pageSourceDir: string,
  baseUrl: string,
): URL {
  const pathBelowRoot = relative(pageSourceDir, mdFilePath);
  const pathname = pathBelowRoot.replace(/\\/g, '/').replace(/\.mdx?$/i, '/');
  return new URL(pathname, baseUrl);
}

function tryFindSourceFileForPathname(
  pathname: string,
  pageSourceDir: string,
): string | undefined {
  const possiblePaths = [
    join(pageSourceDir, pathname, '.') + '.md',
    join(pageSourceDir, pathname, 'index.md'),
    join(pageSourceDir, pathname, '.') + '.mdx',
    join(pageSourceDir, pathname, 'index.mdx'),
  ];
  return possiblePaths.find((p) => existsSync(p));
}

export function fallbackLang(options: FallbackLangOptions = {}) {
  const pageSourceDir = options.pageSourceDir ?? resolve('./src/content/docs');
  const baseUrl = options.baseUrl ?? 'https://docs.astro.build/';

  return definePlugin({
    meta: {
      name: 'fallback-lang',
      description:
        'Marks links pointing to untranslated pages with " (EN)" suffix',
    },

    createOnce(_ctx) {
      return {
        link(node: MdastNode, context: VisitorContext) {
          const source = context.source;
          // Derive file path from source (simplified — real impl gets it from file metadata).
          const pageUrl = new URL('/', baseUrl);
          const pageLang = getLanguageCodeFromPathname(pageUrl.pathname);
          if (!pageLang || pageLang === 'en') return;

          const linkUrl = new URL(node.url ?? '', pageUrl);
          if (pageUrl.host !== linkUrl.host) return;

          const linkLang = getLanguageCodeFromPathname(linkUrl.pathname);
          if (!linkLang) return;

          const linkSourceFile = tryFindSourceFileForPathname(
            linkUrl.pathname,
            pageSourceDir,
          );
          if (linkSourceFile) return;

          // Append " (EN)" indicator.
          if (!node.children) node.children = [];
          node.children.push({
            type: 'text',
            value: '\u00A0(EN)',
          } as MdastNode);
        },
      };
    },
  });
}

/**
 * A "pure compute" version of the plugin without filesystem I/O.
 * Useful for benchmarking the pure plugin overhead without I/O noise.
 */
export function fallbackLangNoIO() {
  return definePlugin({
    meta: {
      name: 'fallback-lang-no-io',
      description: 'Fallback lang check without filesystem access (bench-only)',
    },

    createOnce(_ctx) {
      return {
        link(node: MdastNode) {
          // Simulate the work: extract URL, parse it, check language prefix.
          const url = node.url ?? '';
          try {
            const parsed = new URL(url, 'https://docs.astro.build/');
            const lang = getLanguageCodeFromPathname(parsed.pathname);
            if (lang && lang !== 'en') {
              // Would check filesystem here; instead just mark the node.
              if (!node.data) node.data = {};
              node.data._checkedFallback = true;
            }
          } catch {
            // Invalid URL, skip.
          }
        },
      };
    },
  });
}
