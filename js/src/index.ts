export { ArenaReader, NodeType, NodeTypeName } from './arena-reader.ts';
export { DataMap } from './data-map.ts';
export { materializeNode, materializeTree, TYPE_NAMES } from './materializer.ts';
export { visitArena, MutationType } from './visitor.ts';
export { definePlugin } from './plugin.ts';
export { createProcessor, ProcessorContext } from './processor.ts';
export { parseToBuffer, parseToHastBuffer, mdastBufferToHastBuffer, hastBufferToHtmlStr, compileMdx, compileMdxFromBuffer } from './parse.ts';

// HAST support
export {
  HastArenaReader,
  HAST_ROOT, HAST_ELEMENT, HAST_TEXT, HAST_COMMENT, HAST_DOCTYPE, HAST_RAW,
  PROP_STRING, PROP_BOOL_TRUE, PROP_BOOL_FALSE, PROP_SPACE_SEP, PROP_COMMA_SEP,
} from './hast-reader.ts';
export type { HastProperty } from './hast-reader.ts';
export { materializeHastNode, materializeHastTree } from './hast-materializer.ts';
export type { HastNode } from './hast-materializer.ts';
export { visitHastArena } from './hast-visitor.ts';
export type { HastVisitorInstance, HastVisitorContext, VisitResult as HastVisitResult } from './hast-visitor.ts';

// Built-in plugins
export { default as headingIds } from './plugins/heading-ids.ts';
export { default as lintHeadingDepth } from './plugins/lint-heading-depth.ts';
export { default as flattenHeadings } from './plugins/flatten-headings.ts';
export { default as collectHeadings } from './plugins/collect-headings.ts';

export type { MdastNode, Position, Point } from './types.ts';
