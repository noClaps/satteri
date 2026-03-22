import { definePlugin } from '../plugin.ts';
import type { MdastNode, Position } from '../types.ts';

interface HeadingEntry {
  depth: number;
  position: Position;
}

export default definePlugin({
  meta: {
    name: 'collect-headings',
    description: 'Collects heading metadata across all processed files',
  },

  createOnce(_processorCtx) {
    const allHeadings: HeadingEntry[] = [];

    const instance = {
      before(_fileCtx: unknown, _visitorCtx: unknown) {},

      heading(node: MdastNode) {
        allHeadings.push({
          depth: node.depth ?? 0,
          position: node.position,
        });
      },

      after(_fileCtx: unknown, _visitorCtx: unknown) {},

      getHeadings(): HeadingEntry[] {
        return allHeadings;
      },
    };

    return instance;
  },
});
