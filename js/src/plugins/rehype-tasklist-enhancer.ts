/**
 * Port of docs.astro.build's rehype-tasklist-enhancer plugin.
 *
 * Operates on the HAST (HTML AST) binary arena. Walks `element` nodes
 * to find task list items and wraps checkboxes + siblings in <label>/<span>.
 *
 * This is a HAST-level plugin (rehype equivalent), not an MDAST plugin.
 */

import type { HastNode } from '../hast-materializer.ts';
import type { HastVisitorInstance, HastVisitorContext } from '../hast-visitor.ts';

/**
 * Create a HAST visitor instance that enhances task list markup.
 */
export function rehypeTasklistEnhancer(): HastVisitorInstance {
  return {
    element(node: HastNode, ctx: HastVisitorContext) {
      // Find <li class="task-list-item">
      if (node.tagName !== 'li') return;
      const classes = node.properties?.className;
      if (!Array.isArray(classes) || !classes.includes('task-list-item')) return;

      // Find the checkbox <input> inside
      const children = node.children;
      if (!children) return;

      let checkboxIndex = -1;
      for (let i = 0; i < children.length; i++) {
        if (children[i].type === 'element' && children[i].tagName === 'input') {
          checkboxIndex = i;
          break;
        }
      }
      if (checkboxIndex < 0) return;

      // Split children: [before+checkbox] and [after]
      const head = children.slice(0, checkboxIndex + 1);
      const tail = children.slice(checkboxIndex + 1);

      // Wrap in <label> containing head + <span> around tail
      const span: HastNode = {
        type: 'element',
        tagName: 'span',
        properties: {},
        children: tail,
        data: null,
        _nodeId: -1,
      };

      const label: HastNode = {
        type: 'element',
        tagName: 'label',
        properties: {},
        children: [...head, span],
        data: null,
        _nodeId: -1,
      };

      // Replace children with single label
      node.children = [label];
    },
  };
}
