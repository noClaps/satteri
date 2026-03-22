//! Integration tests for arena rebuild.
//!
//! Tests apply patches to the "# Hello\n\nWorld" arena and verify the resulting structure.

use mdast_arena::{
    rebuild::{rebuild, Patch},
    MdastArena, MdastBuilder, NodeType,
};

/// Build the standard "# Hello\n\nWorld" test arena.
///
/// Tree structure:
///   Root (0)
///     Heading depth=1 (1)
///       Text "Hello" (2)
///     Paragraph (3)
///       Text "World" (4)
fn build_hello_world() -> MdastArena {
    use mdast_arena::codec::{encode_heading_data, encode_string_ref_data};
    use mdast_arena::StringRef;

    let source = "# Hello\n\nWorld".to_string();
    let mut b = MdastBuilder::new(source);

    b.open_node(NodeType::Root);
    b.set_position_current(0, 14, 1, 1, 2, 6);

    b.open_node(NodeType::Heading);
    b.set_position_current(0, 7, 1, 1, 1, 8);
    b.set_data_current(&encode_heading_data(1));

    b.open_node(NodeType::Text);
    b.set_position_current(2, 7, 1, 3, 1, 8);
    b.set_data_current(&encode_string_ref_data(StringRef::new(2, 5)));
    b.close_node();

    b.close_node(); // heading

    b.open_node(NodeType::Paragraph);
    b.set_position_current(9, 14, 2, 1, 2, 6);

    b.open_node(NodeType::Text);
    b.set_position_current(9, 14, 2, 1, 2, 6);
    b.set_data_current(&encode_string_ref_data(StringRef::new(9, 5)));
    b.close_node();

    b.close_node(); // paragraph
    b.close_node(); // root

    b.finish()
}

/// Helper: make a small single-node sub-arena for use in patches.
fn single_node_arena(node_type: NodeType) -> MdastArena {
    let mut b = MdastBuilder::new(String::new());
    b.open_node(node_type);
    b.close_node();
    b.finish()
}

// ── Test 1: Empty patches ────────────────────────────────────────────────────

/// Empty patches → same structure (all nodes preserved, just fresh IDs).
#[test]
fn empty_patches_preserves_all_nodes() {
    let orig = build_hello_world();
    let rebuilt = rebuild(&orig, &[]);

    assert_eq!(rebuilt.len(), orig.len(), "node count unchanged");

    // Tree shape: Root → [Heading → [Text], Paragraph → [Text]]
    assert_eq!(rebuilt.get_children(0).len(), 2);
    let h = rebuilt.get_children(0)[0];
    let p = rebuilt.get_children(0)[1];
    assert_eq!(rebuilt.get_node(h).node_type, NodeType::Heading as u8);
    assert_eq!(rebuilt.get_node(p).node_type, NodeType::Paragraph as u8);
    assert_eq!(rebuilt.get_children(h).len(), 1);
    assert_eq!(rebuilt.get_children(p).len(), 1);
}

// ── Test 2: Remove a leaf node ───────────────────────────────────────────────

/// Remove the Text inside Heading → heading becomes childless, 4 nodes total.
#[test]
fn remove_leaf_node() {
    let orig = build_hello_world();
    let heading_id = orig.get_children(0)[0];
    let text_in_heading = orig.get_children(heading_id)[0];

    let rebuilt = rebuild(
        &orig,
        &[Patch::Remove {
            node_id: text_in_heading,
        }],
    );

    assert_eq!(rebuilt.len(), 4, "one node removed");
    let new_h = rebuilt.get_children(0)[0];
    assert_eq!(rebuilt.get_node(new_h).node_type, NodeType::Heading as u8);
    assert_eq!(
        rebuilt.get_children(new_h).len(),
        0,
        "heading has no children now"
    );

    // Paragraph + its Text are still present
    let new_p = rebuilt.get_children(0)[1];
    assert_eq!(rebuilt.get_node(new_p).node_type, NodeType::Paragraph as u8);
    assert_eq!(rebuilt.get_children(new_p).len(), 1);
}

// ── Test 3: Remove a non-leaf node ───────────────────────────────────────────

/// Remove the Heading (and its subtree) → Root + Paragraph + Text = 3 nodes.
#[test]
fn remove_non_leaf_removes_subtree() {
    let orig = build_hello_world();
    let heading_id = orig.get_children(0)[0];

    let rebuilt = rebuild(
        &orig,
        &[Patch::Remove {
            node_id: heading_id,
        }],
    );

    assert_eq!(rebuilt.len(), 3, "heading + its text child removed");
    let root_children = rebuilt.get_children(0);
    assert_eq!(root_children.len(), 1);
    assert_eq!(
        rebuilt.get_node(root_children[0]).node_type,
        NodeType::Paragraph as u8
    );
}

// ── Test 4: Replace a leaf node ──────────────────────────────────────────────

/// Replace Text under Heading with ThematicBreak.
#[test]
fn replace_leaf_node() {
    let orig = build_hello_world();
    let heading_id = orig.get_children(0)[0];
    let text_id = orig.get_children(heading_id)[0];

    let replacement = single_node_arena(NodeType::ThematicBreak);
    let rebuilt = rebuild(
        &orig,
        &[Patch::Replace {
            node_id: text_id,
            new_tree: replacement,
        }],
    );

    assert_eq!(
        rebuilt.len(),
        orig.len(),
        "same node count (1-for-1 replacement)"
    );
    let new_h = rebuilt.get_children(0)[0];
    let child_of_h = rebuilt.get_children(new_h)[0];
    assert_eq!(
        rebuilt.get_node(child_of_h).node_type,
        NodeType::ThematicBreak as u8
    );
}

// ── Test 5: Replace root's child with a different node type ──────────────────

/// Replace Heading (and its subtree) with a new Paragraph (no children).
#[test]
fn replace_root_child_with_different_type() {
    let orig = build_hello_world();
    let heading_id = orig.get_children(0)[0];

    let replacement = single_node_arena(NodeType::Paragraph);
    let rebuilt = rebuild(
        &orig,
        &[Patch::Replace {
            node_id: heading_id,
            new_tree: replacement,
        }],
    );

    // The heading + its text child (2 nodes) are replaced by 1 Paragraph
    // So: Root + new Paragraph + original Paragraph + Text(World) = 4 nodes
    assert_eq!(rebuilt.len(), 4);
    let root_children = rebuilt.get_children(0);
    assert_eq!(root_children.len(), 2);
    // First child is now a Paragraph (the replacement)
    assert_eq!(
        rebuilt.get_node(root_children[0]).node_type,
        NodeType::Paragraph as u8
    );
    // Second child is still the original Paragraph
    assert_eq!(
        rebuilt.get_node(root_children[1]).node_type,
        NodeType::Paragraph as u8
    );
}

// ── Test 6: Insert before a node ─────────────────────────────────────────────

/// Insert ThematicBreak before Paragraph → Root has 3 children.
#[test]
fn insert_before_node() {
    let orig = build_hello_world();
    let para_id = orig.get_children(0)[1];

    let new_tree = single_node_arena(NodeType::ThematicBreak);
    let rebuilt = rebuild(
        &orig,
        &[Patch::InsertBefore {
            node_id: para_id,
            new_tree,
        }],
    );

    let root_children = rebuilt.get_children(0);
    assert_eq!(root_children.len(), 3);
    assert_eq!(
        rebuilt.get_node(root_children[0]).node_type,
        NodeType::Heading as u8
    );
    assert_eq!(
        rebuilt.get_node(root_children[1]).node_type,
        NodeType::ThematicBreak as u8
    );
    assert_eq!(
        rebuilt.get_node(root_children[2]).node_type,
        NodeType::Paragraph as u8
    );
}

// ── Test 7: Insert after a node ──────────────────────────────────────────────

/// Insert ThematicBreak after Heading → Root has 3 children.
#[test]
fn insert_after_node() {
    let orig = build_hello_world();
    let heading_id = orig.get_children(0)[0];

    let new_tree = single_node_arena(NodeType::ThematicBreak);
    let rebuilt = rebuild(
        &orig,
        &[Patch::InsertAfter {
            node_id: heading_id,
            new_tree,
        }],
    );

    let root_children = rebuilt.get_children(0);
    assert_eq!(root_children.len(), 3);
    assert_eq!(
        rebuilt.get_node(root_children[0]).node_type,
        NodeType::Heading as u8
    );
    assert_eq!(
        rebuilt.get_node(root_children[1]).node_type,
        NodeType::ThematicBreak as u8
    );
    assert_eq!(
        rebuilt.get_node(root_children[2]).node_type,
        NodeType::Paragraph as u8
    );
}

// ── Test 8: Append child ─────────────────────────────────────────────────────

/// Append a Break node to Heading → Heading now has 2 children.
#[test]
fn append_child() {
    let orig = build_hello_world();
    let heading_id = orig.get_children(0)[0];

    let child_tree = single_node_arena(NodeType::Break);
    let rebuilt = rebuild(
        &orig,
        &[Patch::AppendChild {
            node_id: heading_id,
            child_tree,
        }],
    );

    let new_h = rebuilt.get_children(0)[0];
    let h_children = rebuilt.get_children(new_h);
    assert_eq!(h_children.len(), 2);
    assert_eq!(
        rebuilt.get_node(h_children[0]).node_type,
        NodeType::Text as u8
    );
    assert_eq!(
        rebuilt.get_node(h_children[1]).node_type,
        NodeType::Break as u8
    );
}

// ── Test 9: Prepend child ────────────────────────────────────────────────────

/// Prepend a Break node to Heading → Heading now has 2 children (Break first).
#[test]
fn prepend_child() {
    let orig = build_hello_world();
    let heading_id = orig.get_children(0)[0];

    let child_tree = single_node_arena(NodeType::Break);
    let rebuilt = rebuild(
        &orig,
        &[Patch::PrependChild {
            node_id: heading_id,
            child_tree,
        }],
    );

    let new_h = rebuilt.get_children(0)[0];
    let h_children = rebuilt.get_children(new_h);
    assert_eq!(h_children.len(), 2);
    assert_eq!(
        rebuilt.get_node(h_children[0]).node_type,
        NodeType::Break as u8
    );
    assert_eq!(
        rebuilt.get_node(h_children[1]).node_type,
        NodeType::Text as u8
    );
}

// ── Test 10: Multiple patches in one rebuild ──────────────────────────────────

/// Remove heading AND insert ThematicBreak after paragraph.
#[test]
fn multiple_patches_applied_together() {
    let orig = build_hello_world();
    let heading_id = orig.get_children(0)[0];
    let para_id = orig.get_children(0)[1];

    let new_tree = single_node_arena(NodeType::ThematicBreak);

    let patches = vec![
        Patch::Remove {
            node_id: heading_id,
        },
        Patch::InsertAfter {
            node_id: para_id,
            new_tree,
        },
    ];
    let rebuilt = rebuild(&orig, &patches);

    // Root → [Paragraph → [Text(World)], ThematicBreak]
    let root_children = rebuilt.get_children(0);
    assert_eq!(root_children.len(), 2);
    assert_eq!(
        rebuilt.get_node(root_children[0]).node_type,
        NodeType::Paragraph as u8
    );
    assert_eq!(
        rebuilt.get_node(root_children[1]).node_type,
        NodeType::ThematicBreak as u8
    );

    // Total: Root + Paragraph + Text(World) + ThematicBreak = 4 nodes
    assert_eq!(rebuilt.len(), 4);
}

// ── Extra: parent-child integrity ────────────────────────────────────────────

/// After rebuild, all parent references must be consistent.
#[test]
fn parent_references_consistent_after_rebuild() {
    let orig = build_hello_world();
    let para_id = orig.get_children(0)[1];

    let new_tree = single_node_arena(NodeType::ThematicBreak);
    let rebuilt = rebuild(
        &orig,
        &[Patch::InsertAfter {
            node_id: para_id,
            new_tree,
        }],
    );

    // Verify parent references for all non-root nodes
    let root = 0u32;
    for child_id in rebuilt.get_children(root) {
        let child = rebuilt.get_node(*child_id);
        assert_eq!(
            child.parent, root,
            "child of root should have root as parent"
        );

        for grandchild_id in rebuilt.get_children(*child_id) {
            let gc = rebuilt.get_node(*grandchild_id);
            assert_eq!(gc.parent, *child_id, "grandchild parent mismatch");
        }
    }
}
