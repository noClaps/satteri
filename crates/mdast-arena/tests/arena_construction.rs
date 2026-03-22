//! Integration tests for arena construction.

use mdast_arena::{Arena, ArenaBuilder, NodeType};

/// Build a simple Heading node with a Text child and verify structure.
#[test]
fn heading_with_text_child() {
    let source = "# Hello";
    let mut builder = ArenaBuilder::new(source.to_string());

    let root_id = builder.open_node(NodeType::Root);
    builder.set_position_current(0, 7, 1, 1, 1, 8);

    let heading_id = builder.open_node(NodeType::Heading);
    builder.set_position_current(0, 7, 1, 1, 1, 8);

    let text_id = builder.open_node(NodeType::Text);
    builder.set_position_current(2, 7, 1, 3, 1, 8);
    builder.close_node(); // text

    builder.close_node(); // heading
    builder.close_node(); // root

    let arena = builder.finish();

    assert_eq!(arena.len(), 3);

    let root = arena.get_node(root_id);
    assert_eq!(root.node_type, NodeType::Root as u8);
    assert_eq!(root.children_count, 1);

    let heading = arena.get_node(heading_id);
    assert_eq!(heading.node_type, NodeType::Heading as u8);
    assert_eq!(heading.parent, root_id);
    assert_eq!(heading.children_count, 1);

    let text = arena.get_node(text_id);
    assert_eq!(text.node_type, NodeType::Text as u8);
    assert_eq!(text.parent, heading_id);
    assert_eq!(text.children_count, 0);

    assert_eq!(arena.get_children(root_id), &[heading_id]);
    assert_eq!(arena.get_children(heading_id), &[text_id]);
    assert_eq!(arena.get_children(text_id), &[]);
}

/// Build a multi-level tree and verify children relationships.
#[test]
fn multi_level_tree_children() {
    let mut builder = ArenaBuilder::new(String::new());

    let root = builder.open_node(NodeType::Root);
    let p1 = builder.open_node(NodeType::Paragraph);
    let t1 = builder.add_leaf(NodeType::Text);
    let t2 = builder.add_leaf(NodeType::Text);
    builder.close_node(); // paragraph 1

    let p2 = builder.open_node(NodeType::Paragraph);
    let t3 = builder.add_leaf(NodeType::Text);
    builder.close_node(); // paragraph 2

    builder.close_node(); // root

    let arena = builder.finish();

    assert_eq!(arena.get_children(root), &[p1, p2]);
    assert_eq!(arena.get_children(p1), &[t1, t2]);
    assert_eq!(arena.get_children(p2), &[t3]);
}

/// Verify parent IDs are set correctly throughout the tree.
#[test]
fn parent_ids_correct() {
    let mut builder = ArenaBuilder::new(String::new());

    let root = builder.open_node(NodeType::Root);
    let bq = builder.open_node(NodeType::Blockquote);
    let p = builder.open_node(NodeType::Paragraph);
    let t = builder.add_leaf(NodeType::Text);
    builder.close_node(); // paragraph
    builder.close_node(); // blockquote
    builder.close_node(); // root

    let arena = builder.finish();

    // Root has no parent (sentinel = u32::MAX)
    assert_eq!(arena.get_node(root).parent, u32::MAX);
    assert_eq!(arena.get_node(bq).parent, root);
    assert_eq!(arena.get_node(p).parent, bq);
    assert_eq!(arena.get_node(t).parent, p);
}

/// Test the ArenaBuilder open/close pattern with direct Arena methods mixed in.
#[test]
fn arena_direct_methods() {
    let mut arena = Arena::new("foo bar".to_string());

    let root = arena.alloc_node(NodeType::Root);
    let para = arena.alloc_node(NodeType::Paragraph);
    let text = arena.alloc_node(NodeType::Text);

    arena.set_position(root, 0, 7, 1, 1, 1, 8);
    arena.set_position(para, 0, 7, 1, 1, 1, 8);
    arena.set_position(text, 0, 7, 1, 1, 1, 8);

    arena.set_children(root, &[para]);
    arena.set_children(para, &[text]);

    assert_eq!(arena.get_children(root), &[para]);
    assert_eq!(arena.get_children(para), &[text]);
    assert_eq!(arena.get_node(para).parent, root);
    assert_eq!(arena.get_node(text).parent, para);
}

/// A deeply nested structure to check parent/child at every level.
#[test]
fn deep_nesting() {
    let mut builder = ArenaBuilder::new(String::new());
    let root = builder.open_node(NodeType::Root);
    let bq = builder.open_node(NodeType::Blockquote);
    let list = builder.open_node(NodeType::List);
    let item = builder.open_node(NodeType::ListItem);
    let para = builder.open_node(NodeType::Paragraph);
    let leaf = builder.add_leaf(NodeType::Text);
    builder.close_node(); // paragraph
    builder.close_node(); // list item
    builder.close_node(); // list
    builder.close_node(); // blockquote
    builder.close_node(); // root

    let arena = builder.finish();

    assert_eq!(arena.get_node(root).parent, u32::MAX);
    assert_eq!(arena.get_node(bq).parent, root);
    assert_eq!(arena.get_node(list).parent, bq);
    assert_eq!(arena.get_node(item).parent, list);
    assert_eq!(arena.get_node(para).parent, item);
    assert_eq!(arena.get_node(leaf).parent, para);
    assert_eq!(arena.get_children(para), &[leaf]);
}
