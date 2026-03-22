import { definePlugin } from '../plugin.ts';
import type { MdastNode } from '../types.ts';

export default function flattenHeadings({ maxDepth = 3 } = {}): ReturnType<typeof definePlugin> {
  return definePlugin({
    meta: {
      name: 'flatten-headings',
      description: `Clamps all heading depths to a maximum of ${maxDepth}`,
    },

    createOnce(_ctx) {
      return {
        heading(node: MdastNode) {
          if ((node.depth ?? 0) > maxDepth) {
            return { ...node, depth: maxDepth };
          }
        },
      };
    },
  });
}
