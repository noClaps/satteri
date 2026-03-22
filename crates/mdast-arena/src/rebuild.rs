//! Arena rebuild: apply a list of structural patches to an Arena, producing a new one.

use std::collections::{HashMap, HashSet};

use crate::{Arena, ArenaBuilder, NodeType};

/// A patch to apply during arena rebuild.
#[derive(Debug, Clone)]
pub enum Patch {
    /// Replace a node with a new subtree (built externally)
    Replace { node_id: u32, new_tree: Arena },
    /// Remove a node (and its entire subtree)
    Remove { node_id: u32 },
    /// Insert a new subtree before the target node (as a sibling)
    InsertBefore { node_id: u32, new_tree: Arena },
    /// Insert a new subtree after the target node (as a sibling)
    InsertAfter { node_id: u32, new_tree: Arena },
    /// Wrap a node in a new parent (new_tree is the parent; the original node becomes its child)
    Wrap { node_id: u32, parent_tree: Arena },
    /// Prepend a new subtree as the first child of the target node
    PrependChild { node_id: u32, child_tree: Arena },
    /// Append a new subtree as the last child of the target node
    AppendChild { node_id: u32, child_tree: Arena },
}

/// Apply patches to an arena, producing a new arena.
///
/// Node IDs in the new arena are assigned fresh (monotonically increasing)
/// but the structure is preserved. The source string from the original arena
/// is preserved; new nodes from patch sub-arenas reference type_data bytes
/// verbatim (a known limitation for Phase 6 — full StringRef remapping is
/// Phase 8 work).
pub fn rebuild(arena: &Arena, patches: &[Patch]) -> Arena {
    // Index patches by target node_id for O(1) lookup
    let mut patch_map: HashMap<u32, &Patch> = HashMap::new();
    for patch in patches {
        let node_id = match patch {
            Patch::Replace { node_id, .. } => *node_id,
            Patch::Remove { node_id } => *node_id,
            Patch::InsertBefore { node_id, .. } => *node_id,
            Patch::InsertAfter { node_id, .. } => *node_id,
            Patch::Wrap { node_id, .. } => *node_id,
            Patch::PrependChild { node_id, .. } => *node_id,
            Patch::AppendChild { node_id, .. } => *node_id,
        };
        patch_map.insert(node_id, patch);
    }

    // Collect set of "replaced or removed" node IDs — these are skipped in normal traversal
    let mut deleted: HashSet<u32> = HashSet::new();
    for patch in patches {
        match patch {
            Patch::Remove { node_id } => { deleted.insert(*node_id); }
            Patch::Replace { node_id, .. } => { deleted.insert(*node_id); }
            _ => {}
        }
    }

    // New arena — we keep the original source. Sub-arena type_data bytes are
    // copied verbatim (they may reference a different source; this is the Phase 6
    // known limitation — full StringRef remapping is deferred to Phase 8).
    let new_source = arena.source().to_string();
    let mut builder = ArenaBuilder::new(new_source);

    // Recursively copy the original arena starting from the root (node 0),
    // applying patches as we go.
    copy_node(0, arena, &mut builder, &patch_map, &deleted);

    builder.finish()
}

/// Recursively copy a node from `orig` into `builder`, applying patches.
///
/// Returns `true` if the node was emitted (or a replacement was emitted),
/// `false` if the node was silently skipped (e.g. Remove).
fn copy_node(
    node_id: u32,
    orig: &Arena,
    builder: &mut ArenaBuilder,
    patch_map: &HashMap<u32, &Patch>,
    deleted: &HashSet<u32>,
) -> bool {
    // If this node is in the deleted set (Remove or Replace), skip it here.
    // For Remove: nothing emitted.
    // For Replace: the replacement is emitted by the *parent* when iterating children
    //   (or at root level, handled below). If node_id == 0 (root), we must handle here.
    if deleted.contains(&node_id) {
        // Emit replacement if it's a Replace patch (when called for a node that
        // is being replaced — this path is hit for the root node or when
        // copy_children iterates and finds a replaced child).
        if let Some(Patch::Replace { new_tree, .. }) = patch_map.get(&node_id) {
            emit_subtree(new_tree, builder);
            return true;
        }
        // Remove: nothing to emit
        return false;
    }

    // InsertBefore: emit the new sibling *before* emitting this node
    if let Some(Patch::InsertBefore { new_tree, .. }) = patch_map.get(&node_id) {
        emit_subtree(new_tree, builder);
    }

    // Handle Wrap: the patch node becomes the parent, and the original node
    // becomes a child inside it.
    if let Some(Patch::Wrap { parent_tree, .. }) = patch_map.get(&node_id) {
        // Emit wrapper: we need to open the wrapper's root, emit original as
        // a child, then close. We do this by:
        // 1. Copy the parent_tree structure except its leaf nodes are replaced
        //    by our node.
        // Because parent_tree may have its own structure, we emit the whole
        // parent_tree but then the original node needs to be inserted as a child.
        // For Phase 6, we implement a simpler version: the parent_tree is assumed
        // to be a single node (wrapper) with no children. The original node
        // becomes its only child.
        emit_wrap_node(parent_tree, node_id, orig, builder, patch_map, deleted);

        // InsertAfter (after the wrapped group)
        if let Some(Patch::InsertAfter { new_tree, .. }) = patch_map.get(&node_id) {
            emit_subtree(new_tree, builder);
        }
        return true;
    }

    // Open this node in the new arena
    let node = orig.get_node(node_id);
    let node_type = NodeType::from_u8(node.node_type)
        .expect("unknown node type in arena — corrupt data");

    builder.open_node(node_type);

    // Copy position data
    builder.set_position_current(
        node.start_offset,
        node.end_offset,
        node.start_line,
        node.start_column,
        node.end_line,
        node.end_column,
    );

    // Copy type-specific data bytes verbatim
    let type_data = orig.get_type_data(node_id);
    if !type_data.is_empty() {
        builder.set_data_current(type_data);
    }

    // PrependChild: emit a new child *before* original children
    if let Some(Patch::PrependChild { child_tree, .. }) = patch_map.get(&node_id) {
        emit_subtree(child_tree, builder);
    }

    // Children: iterate, handling per-child patches
    let child_ids: Vec<u32> = orig.get_children(node_id).to_vec();
    for child_id in child_ids {
        if deleted.contains(&child_id) {
            // This child is Replace or Remove
            if let Some(Patch::Replace { new_tree, .. }) = patch_map.get(&child_id) {
                emit_subtree(new_tree, builder);
            }
            // Remove: nothing emitted
        } else {
            // InsertBefore for this child (handled in copy_node recursion)
            copy_node(child_id, orig, builder, patch_map, deleted);
        }
    }

    // AppendChild: emit a new child *after* original children
    if let Some(Patch::AppendChild { child_tree, .. }) = patch_map.get(&node_id) {
        emit_subtree(child_tree, builder);
    }

    builder.close_node();

    // InsertAfter: emit new sibling *after* this node
    if let Some(Patch::InsertAfter { new_tree, .. }) = patch_map.get(&node_id) {
        emit_subtree(new_tree, builder);
    }

    true
}

/// Emit all nodes from a sub-arena into the builder.
/// Starts from the sub-arena root (node 0) and recursively copies structure.
///
/// Note: type_data bytes are copied verbatim. For structural nodes (Heading,
/// Paragraph, etc. built with NodeBuilder without inline text content), this
/// works correctly. For nodes with StringRef type data referencing the sub-arena's
/// own source, those StringRefs will reference the original arena's source instead,
/// which is a known Phase 6 limitation documented here.
fn emit_subtree(sub_arena: &Arena, builder: &mut ArenaBuilder) {
    if sub_arena.is_empty() {
        return;
    }
    emit_subtree_node(0, sub_arena, builder);
}

/// Recursively emit nodes from sub_arena starting at `node_id`.
fn emit_subtree_node(node_id: u32, sub_arena: &Arena, builder: &mut ArenaBuilder) {
    let node = sub_arena.get_node(node_id);
    let node_type = NodeType::from_u8(node.node_type)
        .expect("unknown node type in sub-arena — corrupt data");

    builder.open_node(node_type);

    builder.set_position_current(
        node.start_offset,
        node.end_offset,
        node.start_line,
        node.start_column,
        node.end_line,
        node.end_column,
    );

    let type_data = sub_arena.get_type_data(node_id);
    if !type_data.is_empty() {
        builder.set_data_current(type_data);
    }

    let child_ids: Vec<u32> = sub_arena.get_children(node_id).to_vec();
    for child_id in child_ids {
        emit_subtree_node(child_id, sub_arena, builder);
    }

    builder.close_node();
}

/// Emit a Wrap: open the wrapper node (first node from parent_tree's root),
/// then emit the original node as a child, then close the wrapper.
///
/// This assumes parent_tree's root is the single wrapper. Any children already
/// in parent_tree's root are ignored in favor of the original node becoming the
/// sole child. This is the Phase 6 Wrap implementation.
fn emit_wrap_node(
    parent_tree: &Arena,
    original_node_id: u32,
    orig: &Arena,
    builder: &mut ArenaBuilder,
    patch_map: &HashMap<u32, &Patch>,
    deleted: &HashSet<u32>,
) {
    if parent_tree.is_empty() {
        // Degenerate: no wrapper, just emit original
        copy_node(original_node_id, orig, builder, patch_map, deleted);
        return;
    }

    let wrapper = parent_tree.get_node(0);
    let node_type = NodeType::from_u8(wrapper.node_type)
        .expect("unknown node type in wrapper arena");

    builder.open_node(node_type);
    builder.set_position_current(
        wrapper.start_offset,
        wrapper.end_offset,
        wrapper.start_line,
        wrapper.start_column,
        wrapper.end_line,
        wrapper.end_column,
    );
    let wrapper_data = parent_tree.get_type_data(0);
    if !wrapper_data.is_empty() {
        builder.set_data_current(wrapper_data);
    }

    // Emit the original node as the child (ignoring any children in parent_tree)
    copy_node(original_node_id, orig, builder, patch_map, deleted);

    builder.close_node();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ArenaBuilder, NodeType};

    /// Build the "# Hello\n\nWorld" arena for testing.
    fn build_hello_world() -> Arena {
        use crate::codec::{encode_heading_data, encode_string_ref_data};
        use crate::node::StringRef;

        let source = "# Hello\n\nWorld".to_string();
        let mut b = ArenaBuilder::new(source);

        b.open_node(NodeType::Root);
        b.set_position_current(0, 14, 1, 1, 2, 6);

        b.open_node(NodeType::Heading);
        b.set_position_current(0, 7, 1, 1, 1, 8);
        b.set_data_current(&encode_heading_data(1));

        b.open_node(NodeType::Text);
        b.set_position_current(2, 7, 1, 3, 1, 8);
        b.set_data_current(&encode_string_ref_data(StringRef::new(2, 5)));
        b.close_node(); // text

        b.close_node(); // heading

        b.open_node(NodeType::Paragraph);
        b.set_position_current(9, 14, 2, 1, 2, 6);

        b.open_node(NodeType::Text);
        b.set_position_current(9, 14, 2, 1, 2, 6);
        b.set_data_current(&encode_string_ref_data(StringRef::new(9, 5)));
        b.close_node(); // text

        b.close_node(); // paragraph
        b.close_node(); // root

        b.finish()
    }

    #[test]
    fn empty_patches_preserves_structure() {
        let orig = build_hello_world();
        let rebuilt = rebuild(&orig, &[]);
        assert_eq!(rebuilt.len(), orig.len(), "node count must be the same");
        // Root still has 2 children
        assert_eq!(rebuilt.get_children(0).len(), 2);
    }

    #[test]
    fn remove_leaf_node() {
        // Remove the Text node inside Heading (node 2 in the original tree).
        // Original: Root(0) -> Heading(1) -> Text(2), Paragraph(3) -> Text(4)
        let orig = build_hello_world();
        // Find the Text child of Heading
        let heading_id = orig.get_children(0)[0]; // id=1
        let text_in_heading = orig.get_children(heading_id)[0]; // id=2

        let patches = vec![Patch::Remove { node_id: text_in_heading }];
        let rebuilt = rebuild(&orig, &patches);

        // We should have 4 nodes: Root, Heading (now empty), Paragraph, Text(World)
        assert_eq!(rebuilt.len(), 4, "text under heading should be removed");

        // Heading child in rebuilt arena — find heading
        let new_root_children = rebuilt.get_children(0);
        assert_eq!(new_root_children.len(), 2);
        let new_heading_id = new_root_children[0];
        assert_eq!(rebuilt.get_node(new_heading_id).node_type, NodeType::Heading as u8);
        assert_eq!(rebuilt.get_children(new_heading_id).len(), 0, "heading should have no children");
    }

    #[test]
    fn remove_non_leaf_removes_subtree() {
        let orig = build_hello_world();
        // Remove the Heading (and its Text child)
        let heading_id = orig.get_children(0)[0]; // id=1
        let patches = vec![Patch::Remove { node_id: heading_id }];
        let rebuilt = rebuild(&orig, &patches);

        // Root + Paragraph + Text(World) = 3 nodes
        assert_eq!(rebuilt.len(), 3);
        let new_root_children = rebuilt.get_children(0);
        assert_eq!(new_root_children.len(), 1);
        assert_eq!(rebuilt.get_node(new_root_children[0]).node_type, NodeType::Paragraph as u8);
    }

    #[test]
    fn replace_leaf_node() {
        let orig = build_hello_world();
        let heading_id = orig.get_children(0)[0];
        let text_id = orig.get_children(heading_id)[0];

        // Build a replacement: a ThematicBreak (no children, no data)
        let mut replacement_builder = ArenaBuilder::new(orig.source().to_string());
        replacement_builder.open_node(NodeType::ThematicBreak);
        replacement_builder.close_node();
        let replacement = replacement_builder.finish();

        let patches = vec![Patch::Replace { node_id: text_id, new_tree: replacement }];
        let rebuilt = rebuild(&orig, &patches);

        // Same node count (Text replaced by ThematicBreak, 1-for-1)
        assert_eq!(rebuilt.len(), orig.len());
        // Find ThematicBreak under Heading
        let new_heading_id = rebuilt.get_children(0)[0];
        let child_of_heading = rebuilt.get_children(new_heading_id)[0];
        assert_eq!(
            rebuilt.get_node(child_of_heading).node_type,
            NodeType::ThematicBreak as u8
        );
    }

    #[test]
    fn replace_root_child_with_different_type() {
        let orig = build_hello_world();
        let heading_id = orig.get_children(0)[0];

        // Replace Heading with a Paragraph
        let mut replacement_builder = ArenaBuilder::new(orig.source().to_string());
        replacement_builder.open_node(NodeType::Paragraph);
        replacement_builder.close_node();
        let replacement = replacement_builder.finish();

        let patches = vec![Patch::Replace { node_id: heading_id, new_tree: replacement }];
        let rebuilt = rebuild(&orig, &patches);

        // Root should still have 2 children; first one is now Paragraph
        let root_children = rebuilt.get_children(0);
        assert_eq!(root_children.len(), 2);
        assert_eq!(rebuilt.get_node(root_children[0]).node_type, NodeType::Paragraph as u8);
        // Second child should still be the original Paragraph
        assert_eq!(rebuilt.get_node(root_children[1]).node_type, NodeType::Paragraph as u8);
    }

    #[test]
    fn insert_before_node() {
        let orig = build_hello_world();
        let para_id = orig.get_children(0)[1]; // Paragraph node

        // Insert a ThematicBreak before the Paragraph
        let mut new_tree_builder = ArenaBuilder::new(orig.source().to_string());
        new_tree_builder.open_node(NodeType::ThematicBreak);
        new_tree_builder.close_node();
        let new_tree = new_tree_builder.finish();

        let patches = vec![Patch::InsertBefore { node_id: para_id, new_tree }];
        let rebuilt = rebuild(&orig, &patches);

        // Root should now have 3 children: Heading, ThematicBreak, Paragraph
        let root_children = rebuilt.get_children(0);
        assert_eq!(root_children.len(), 3);
        assert_eq!(rebuilt.get_node(root_children[0]).node_type, NodeType::Heading as u8);
        assert_eq!(rebuilt.get_node(root_children[1]).node_type, NodeType::ThematicBreak as u8);
        assert_eq!(rebuilt.get_node(root_children[2]).node_type, NodeType::Paragraph as u8);
    }

    #[test]
    fn insert_after_node() {
        let orig = build_hello_world();
        let heading_id = orig.get_children(0)[0]; // Heading node

        let mut new_tree_builder = ArenaBuilder::new(orig.source().to_string());
        new_tree_builder.open_node(NodeType::ThematicBreak);
        new_tree_builder.close_node();
        let new_tree = new_tree_builder.finish();

        let patches = vec![Patch::InsertAfter { node_id: heading_id, new_tree }];
        let rebuilt = rebuild(&orig, &patches);

        // Root should now have 3 children: Heading, ThematicBreak, Paragraph
        let root_children = rebuilt.get_children(0);
        assert_eq!(root_children.len(), 3);
        assert_eq!(rebuilt.get_node(root_children[0]).node_type, NodeType::Heading as u8);
        assert_eq!(rebuilt.get_node(root_children[1]).node_type, NodeType::ThematicBreak as u8);
        assert_eq!(rebuilt.get_node(root_children[2]).node_type, NodeType::Paragraph as u8);
    }

    #[test]
    fn append_child() {
        let orig = build_hello_world();
        let heading_id = orig.get_children(0)[0];

        let mut child_builder = ArenaBuilder::new(orig.source().to_string());
        child_builder.open_node(NodeType::Break);
        child_builder.close_node();
        let child_tree = child_builder.finish();

        let patches = vec![Patch::AppendChild { node_id: heading_id, child_tree }];
        let rebuilt = rebuild(&orig, &patches);

        // Heading should now have 2 children: original Text + new Break
        let new_heading_id = rebuilt.get_children(0)[0];
        let heading_children = rebuilt.get_children(new_heading_id);
        assert_eq!(heading_children.len(), 2);
        assert_eq!(rebuilt.get_node(heading_children[0]).node_type, NodeType::Text as u8);
        assert_eq!(rebuilt.get_node(heading_children[1]).node_type, NodeType::Break as u8);
    }

    #[test]
    fn prepend_child() {
        let orig = build_hello_world();
        let heading_id = orig.get_children(0)[0];

        let mut child_builder = ArenaBuilder::new(orig.source().to_string());
        child_builder.open_node(NodeType::Break);
        child_builder.close_node();
        let child_tree = child_builder.finish();

        let patches = vec![Patch::PrependChild { node_id: heading_id, child_tree }];
        let rebuilt = rebuild(&orig, &patches);

        // Heading should now have 2 children: new Break + original Text
        let new_heading_id = rebuilt.get_children(0)[0];
        let heading_children = rebuilt.get_children(new_heading_id);
        assert_eq!(heading_children.len(), 2);
        assert_eq!(rebuilt.get_node(heading_children[0]).node_type, NodeType::Break as u8);
        assert_eq!(rebuilt.get_node(heading_children[1]).node_type, NodeType::Text as u8);
    }

    #[test]
    fn multiple_patches_applied_together() {
        let orig = build_hello_world();
        let heading_id = orig.get_children(0)[0];
        let para_id = orig.get_children(0)[1];

        // Remove the heading AND insert a ThematicBreak after paragraph
        let mut new_tree_builder = ArenaBuilder::new(orig.source().to_string());
        new_tree_builder.open_node(NodeType::ThematicBreak);
        new_tree_builder.close_node();
        let new_tree = new_tree_builder.finish();

        let patches = vec![
            Patch::Remove { node_id: heading_id },
            Patch::InsertAfter { node_id: para_id, new_tree },
        ];
        let rebuilt = rebuild(&orig, &patches);

        // Root should have 2 children: original Paragraph + new ThematicBreak
        let root_children = rebuilt.get_children(0);
        assert_eq!(root_children.len(), 2);
        assert_eq!(rebuilt.get_node(root_children[0]).node_type, NodeType::Paragraph as u8);
        assert_eq!(rebuilt.get_node(root_children[1]).node_type, NodeType::ThematicBreak as u8);
    }
}
