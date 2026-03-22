import { definePlugin } from '../plugin.ts';
import type { MdastNode } from '../types.ts';

export default function lintHeadingDepth({ maxDepth = 3 } = {}): ReturnType<typeof definePlugin> {
  return definePlugin({
    meta: {
      name: 'lint-heading-depth',
      description: `Reports headings deeper than h${maxDepth}`,
    },

    createOnce(_ctx) {
      return {
        heading(node: MdastNode, context) {
          if ((node.depth ?? 0) > maxDepth) {
            context.report({
              message: `Heading depth ${node.depth} exceeds maximum of ${maxDepth}`,
              node,
              severity: 'warning',
            });
          }
        },
      };
    },
  });
}
