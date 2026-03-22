//! Core Arena struct and methods.

use crate::node::{ArenaNode, NodeType, StringRef};

/// The central arena that owns all nodes and associated data for one parse.
///
/// Strings are NOT copied — the arena holds the source and nodes reference it
/// via `StringRef` (byte offset + length into `source`).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Arena {
    /// All nodes in order of creation.
    pub(crate) nodes: Vec<ArenaNode>,
    /// Flat array of child node IDs, indexed by node.children_start..+children_count.
    pub(crate) children: Vec<u32>,
    /// Variable-length type-specific data, packed.
    pub(crate) type_data: Vec<u8>,
    /// The source Markdown text.
    pub(crate) source: String,
}

impl Arena {
    pub fn new(source: String) -> Self {
        Arena {
            nodes: Vec::new(),
            children: Vec::new(),
            type_data: Vec::new(),
            source,
        }
    }

    /// Allocate a new node and return its ID (== index in `nodes`).
    pub fn alloc_node(&mut self, node_type: NodeType) -> u32 {
        let id = self.nodes.len() as u32;
        self.nodes.push(ArenaNode::new(id, node_type));
        id
    }

    /// Allocate a new node using a raw u8 type byte (bypasses NodeType enum).
    /// Use this when building HAST or other non-MDAST arenas in the same binary format.
    pub fn alloc_node_raw(&mut self, node_type_byte: u8) -> u32 {
        let id = self.nodes.len() as u32;
        let mut node = ArenaNode::new(id, NodeType::Root); // placeholder
        node.node_type = node_type_byte;
        self.nodes.push(node);
        id
    }

    /// Set the parent of a node.
    pub fn set_parent(&mut self, node_id: u32, parent_id: u32) {
        self.nodes[node_id as usize].parent = parent_id;
    }

    /// Set position data on a node.
    #[allow(clippy::too_many_arguments)]
    pub fn set_position(
        &mut self,
        node_id: u32,
        start_offset: u32,
        end_offset: u32,
        start_line: u32,
        start_column: u32,
        end_line: u32,
        end_column: u32,
    ) {
        let node = &mut self.nodes[node_id as usize];
        node.start_offset = start_offset;
        node.end_offset = end_offset;
        node.start_line = start_line;
        node.start_column = start_column;
        node.end_line = end_line;
        node.end_column = end_column;
    }

    /// Set children for a node — appends the slice to the flat children array
    /// and records the start index and count on the node.
    pub fn set_children(&mut self, node_id: u32, child_ids: &[u32]) {
        let start = self.children.len() as u32;
        self.children.extend_from_slice(child_ids);
        let node = &mut self.nodes[node_id as usize];
        node.children_start = start;
        node.children_count = child_ids.len() as u32;
        // Update parent references
        for &child_id in child_ids {
            self.nodes[child_id as usize].parent = node_id;
        }
    }

    /// Add a single child during incremental building.
    ///
    /// NOTE: This appends to the end of the children flat array each time it
    /// is called.  It is meant to be called once per child when finalising a
    /// node (i.e. from `ArenaBuilder::close_node`), NOT one-by-one during
    /// open construction.  The builder accumulates children in its stack and
    /// calls `set_children` when closing a node.
    pub fn add_child(&mut self, parent_id: u32, child_id: u32) {
        let start = self.children.len() as u32;
        self.children.push(child_id);
        let parent = &mut self.nodes[parent_id as usize];
        // If this node has no children yet, initialise children_start.
        if parent.children_count == 0 {
            parent.children_start = start;
        }
        parent.children_count += 1;
        self.nodes[child_id as usize].parent = parent_id;
    }

    /// Write type-specific data bytes into type_data, update node's
    /// `data_offset`/`data_len`.
    pub fn set_type_data(&mut self, node_id: u32, data: &[u8]) {
        let offset = self.type_data.len() as u32;
        self.type_data.extend_from_slice(data);
        let node = &mut self.nodes[node_id as usize];
        node.data_offset = offset;
        node.data_len = data.len() as u32;
    }

    /// Get a node by ID.
    pub fn get_node(&self, node_id: u32) -> &ArenaNode {
        &self.nodes[node_id as usize]
    }

    /// Get a node mutably.
    pub fn get_node_mut(&mut self, node_id: u32) -> &mut ArenaNode {
        &mut self.nodes[node_id as usize]
    }

    /// Get all child IDs for a node.
    pub fn get_children(&self, node_id: u32) -> &[u32] {
        let node = &self.nodes[node_id as usize];
        let start = node.children_start as usize;
        let end = start + node.children_count as usize;
        &self.children[start..end]
    }

    /// Read a string from the source by StringRef.
    pub fn get_str(&self, string_ref: StringRef) -> &str {
        let start = string_ref.offset as usize;
        let end = start + string_ref.len as usize;
        &self.source[start..end]
    }

    /// Get the source text.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Number of nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Append a computed string to the source buffer and return a `StringRef`
    /// pointing to it.  Use this for strings that don't map directly to the
    /// original source (e.g. decoded character references, normalised
    /// identifiers, alt text built from label children).
    pub fn alloc_string(&mut self, s: &str) -> StringRef {
        let offset = self.source.len() as u32;
        let len = s.len() as u32;
        self.source.push_str(s);
        StringRef::new(offset, len)
    }

    /// Change the node type of an already-allocated node (used when we
    /// discover at close time that a Link/Image is actually a reference).
    pub fn change_node_type(&mut self, node_id: u32, new_type: NodeType) {
        self.nodes[node_id as usize].node_type = new_type as u8;
    }

    /// Access the raw type_data bytes (for tests and the raw buffer layer).
    pub fn arena_type_data(&self) -> &[u8] {
        &self.type_data
    }

    /// Get the type-specific data bytes for a given node.
    pub fn get_type_data(&self, node_id: u32) -> &[u8] {
        let node = &self.nodes[node_id as usize];
        let start = node.data_offset as usize;
        let end = start + node.data_len as usize;
        &self.type_data[start..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_and_retrieve() {
        let mut arena = Arena::new("hello world".to_string());
        let id = arena.alloc_node(NodeType::Text);
        assert_eq!(id, 0);
        assert_eq!(arena.len(), 1);
        let node = arena.get_node(id);
        assert_eq!(node.node_type, NodeType::Text as u8);
    }

    #[test]
    fn set_position_roundtrip() {
        let mut arena = Arena::new(String::new());
        let id = arena.alloc_node(NodeType::Paragraph);
        arena.set_position(id, 0, 10, 1, 1, 1, 11);
        let node = arena.get_node(id);
        assert_eq!(node.start_offset, 0);
        assert_eq!(node.end_offset, 10);
        assert_eq!(node.start_line, 1);
        assert_eq!(node.end_column, 11);
    }

    #[test]
    fn set_children_updates_parent() {
        let mut arena = Arena::new(String::new());
        let parent = arena.alloc_node(NodeType::Paragraph);
        let child1 = arena.alloc_node(NodeType::Text);
        let child2 = arena.alloc_node(NodeType::Text);
        arena.set_children(parent, &[child1, child2]);
        assert_eq!(arena.get_children(parent), &[child1, child2]);
        assert_eq!(arena.get_node(child1).parent, parent);
        assert_eq!(arena.get_node(child2).parent, parent);
    }

    #[test]
    fn get_str_works() {
        let source = "Hello, world!".to_string();
        let arena = Arena::new(source);
        let sr = StringRef::new(7, 5);
        assert_eq!(arena.get_str(sr), "world");
    }

    #[test]
    fn type_data_roundtrip() {
        let mut arena = Arena::new(String::new());
        let id = arena.alloc_node(NodeType::Heading);
        arena.set_type_data(id, &[2u8]);
        let node = arena.get_node(id);
        assert_eq!(node.data_len, 1);
        let stored = &arena.type_data[node.data_offset as usize..][..node.data_len as usize];
        assert_eq!(stored, &[2u8]);
    }
}
