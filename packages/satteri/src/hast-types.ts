// Re-export standard hast types so consumers don't need direct @types/hast deps
export type {
  Root,
  Element,
  Text,
  Comment,
  Doctype,
  Properties,
  Nodes,
  RootContent,
  ElementContent,
  Data,
  Literal,
  Parent,
} from "hast";

// MDX hast types
export type {
  MdxJsxFlowElementHast,
  MdxJsxTextElementHast,
  MdxJsxAttribute,
  MdxJsxExpressionAttribute,
  MdxJsxAttributeValueExpression,
  MdxFlowExpressionHast,
  MdxTextExpressionHast,
  MdxjsEsmHast,
} from "./mdx-types.js";

// Custom extension types
export type { HastRaw } from "./types.js";
