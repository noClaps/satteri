//! ArenaBuilder — construct an Arena incrementally during parsing.
//!
//! Nodes are opened (pushed on a stack) and closed (popped, children
//! finalised).  Leaf nodes are opened and immediately closed.

use crate::arena::Arena;
use crate::node::{NodeType, StringRef};

/// Builds an `Arena` using an open/close node pattern suitable for
/// depth-first tree construction (e.g. SAX-style parsers).
pub struct ArenaBuilder {
    arena: Arena,
    /// Stack of `(node_id, children_collected_so_far)`.
    stack: Vec<(u32, Vec<u32>)>,
}

impl ArenaBuilder {
    pub fn new(source: String) -> Self {
        ArenaBuilder {
            arena: Arena::new(source),
            stack: Vec::new(),
        }
    }

    /// Open a new node (push onto stack), returns its node_id.
    pub fn open_node(&mut self, node_type: NodeType) -> u32 {
        let node_id = self.arena.alloc_node(node_type);
        self.stack.push((node_id, Vec::new()));
        node_id
    }

    /// Close the current node (pop from stack, attach children, attach to
    /// parent). Returns the closed node's ID.
    pub fn close_node(&mut self) -> u32 {
        let (node_id, children) = self
            .stack
            .pop()
            .expect("close_node called with empty stack");

        // Finalise children into the flat children array.
        self.arena.set_children(node_id, &children);

        // Register this node as a child of the new stack top (its parent).
        if let Some((parent_id, parent_children)) = self.stack.last_mut() {
            parent_children.push(node_id);
            self.arena.set_parent(node_id, *parent_id);
        }

        node_id
    }

    /// Add a leaf node (opens and immediately closes it). Returns the node ID.
    pub fn add_leaf(&mut self, node_type: NodeType) -> u32 {
        self.open_node(node_type);
        self.close_node()
    }

    /// Open a new node using a raw u8 type byte (bypasses NodeType enum).
    /// Used when building HAST or other non-MDAST arenas in the same binary format.
    pub fn open_node_raw(&mut self, node_type_byte: u8) -> u32 {
        let node_id = self.arena.alloc_node_raw(node_type_byte);
        self.stack.push((node_id, Vec::new()));
        node_id
    }

    /// Add a leaf node using a raw u8 type byte. Opens and immediately closes it.
    pub fn add_leaf_raw(&mut self, node_type_byte: u8) -> u32 {
        self.open_node_raw(node_type_byte);
        self.close_node()
    }

    /// Set position on the current top-of-stack node.
    #[allow(clippy::too_many_arguments)]
    pub fn set_position_current(
        &mut self,
        start_offset: u32,
        end_offset: u32,
        start_line: u32,
        start_column: u32,
        end_line: u32,
        end_column: u32,
    ) {
        let node_id = self
            .stack
            .last()
            .expect("set_position_current called with empty stack")
            .0;
        self.arena
            .set_position(node_id, start_offset, end_offset, start_line, start_column, end_line, end_column);
    }

    /// Set type-specific data on the current top-of-stack node.
    pub fn set_data_current(&mut self, data: &[u8]) {
        let node_id = self
            .stack
            .last()
            .expect("set_data_current called with empty stack")
            .0;
        self.arena.set_type_data(node_id, data);
    }

    /// Append a computed string to the arena's source buffer and return a
    /// `StringRef` pointing to it.  Delegates to `Arena::alloc_string`.
    pub fn alloc_string(&mut self, s: &str) -> StringRef {
        self.arena.alloc_string(s)
    }

    /// Change the type of an already-opened node (e.g. Link → LinkReference).
    pub fn change_node_type(&mut self, node_id: u32, new_type: NodeType) {
        self.arena.change_node_type(node_id, new_type);
    }

    /// Get the node ID of the current top-of-stack node (without popping).
    pub fn current_node_id(&self) -> u32 {
        self.stack
            .last()
            .expect("current_node_id called with empty stack")
            .0
    }

    /// Get the current stack depth (number of open nodes).
    pub fn stack_depth(&self) -> usize {
        self.stack.len()
    }

    /// Get the node ID at a specific stack position (0 = bottom/root).
    pub fn stack_node_id(&self, depth: usize) -> Option<u32> {
        self.stack.get(depth).map(|(id, _)| *id)
    }

    /// Get read-only access to the underlying Arena.
    pub fn arena_ref(&self) -> &Arena {
        &self.arena
    }

    /// Get mutable access to the pending children of the current top-of-stack node.
    pub fn current_children_mut(&mut self) -> &mut Vec<u32> {
        &mut self.stack.last_mut().expect("empty stack").1
    }

    /// Get mutable access to the underlying Arena.
    pub fn arena_mut(&mut self) -> &mut Arena {
        &mut self.arena
    }

    /// Finish building — closes any remaining open nodes and returns the
    /// completed Arena.
    pub fn finish(mut self) -> Arena {
        // Close remaining open nodes (root last).
        while !self.stack.is_empty() {
            self.close_node();
        }
        self.arena
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeType;

    #[test]
    fn simple_open_close() {
        let mut builder = ArenaBuilder::new("# Hello".to_string());
        let root = builder.open_node(NodeType::Root);
        let heading = builder.open_node(NodeType::Heading);
        let text = builder.add_leaf(NodeType::Text);
        let heading_closed = builder.close_node();
        let root_closed = builder.close_node();
        assert_eq!(heading_closed, heading);
        assert_eq!(root_closed, root);

        let arena = builder.finish();
        assert_eq!(arena.len(), 3);
        assert_eq!(arena.get_children(root), &[heading]);
        assert_eq!(arena.get_children(heading), &[text]);
        assert_eq!(arena.get_node(text).parent, heading);
        assert_eq!(arena.get_node(heading).parent, root);
    }

    #[test]
    fn finish_closes_open_nodes() {
        let mut builder = ArenaBuilder::new(String::new());
        builder.open_node(NodeType::Root);
        builder.open_node(NodeType::Paragraph);
        builder.add_leaf(NodeType::Text);
        // Do NOT close explicitly — finish() should handle it.
        let arena = builder.finish();
        assert_eq!(arena.len(), 3);
    }

    #[test]
    fn leaf_has_no_children() {
        let mut builder = ArenaBuilder::new(String::new());
        builder.open_node(NodeType::Root);
        let leaf = builder.add_leaf(NodeType::Break);
        builder.close_node();
        let arena = builder.finish();
        assert_eq!(arena.get_children(leaf), &[]);
    }

    #[test]
    fn position_and_data_current() {
        let mut builder = ArenaBuilder::new("hello".to_string());
        let id = builder.open_node(NodeType::Text);
        builder.set_position_current(0, 5, 1, 1, 1, 6);
        builder.set_data_current(&[42u8]);
        builder.close_node();
        let arena = builder.finish();
        let node = arena.get_node(id);
        assert_eq!(node.start_offset, 0);
        assert_eq!(node.end_offset, 5);
        assert_eq!(node.data_len, 1);
    }
}
