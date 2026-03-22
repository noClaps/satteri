import { definePlugin } from '../plugin.ts';
import type { MdastNode } from '../types.ts';

function slugify(text: string): string {
  return text
    .toLowerCase()
    .trim()
    .replace(/[^\w\s-]/g, '')
    .replace(/[\s_-]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

function extractText(node: MdastNode): string {
  if (node.type === 'text' || node.type === 'inlineCode') {
    return node.value ?? '';
  }
  if (node.children) {
    return node.children.map(extractText).join('');
  }
  return '';
}

export default definePlugin({
  meta: {
    name: 'heading-ids',
    description: 'Adds slug IDs to headings via node.data.hProperties.id',
  },

  createOnce(_ctx) {
    return {
      heading(node: MdastNode) {
        const text = extractText(node);
        const id = slugify(text);
        node.data = { ...node.data, id, hProperties: { id } };
      },
    };
  },
});
