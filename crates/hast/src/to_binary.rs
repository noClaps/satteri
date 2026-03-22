//! Convert an MDAST binary buffer to a HAST binary buffer.

use mdast_arena::{
    Arena, ArenaBuilder, ArenaView, BufferError, NodeType, StringRef,
    decode_code_data, decode_heading_data, decode_image_data, decode_link_data,
    decode_list_data, decode_list_item_data, decode_math_data, decode_string_ref_data,
    decode_definition_data, decode_reference_data,
};

use crate::codec::encode_text_data;
use crate::node_types::*;

/// Convert an MDAST binary buffer to a HAST binary buffer.
pub fn arena_to_hast_buffer(mdast_buf: &[u8]) -> Result<Vec<u8>, BufferError> {
    let view = Arena::from_raw_buffer(mdast_buf)?;
    let mut builder = ArenaBuilder::new(String::new());

    // Pre-pass: collect definitions for link/image reference resolution.
    let defs = collect_definitions(&view);

    // Convert starting from root (node 0).
    convert_node(0, &view, &mut builder, &defs);

    let hast_arena = builder.finish();
    Ok(hast_arena.to_raw_buffer())
}

// ---------------------------------------------------------------------------
// Definition collection (pre-pass)
// ---------------------------------------------------------------------------

struct Definition {
    identifier: String,
    url: String,
    title: Option<String>,
}

fn collect_definitions(view: &ArenaView) -> Vec<Definition> {
    let mut defs = Vec::new();
    for id in 0..view.node_count() {
        let node = view.get_node(id);
        if node.node_type == NodeType::Definition as u8 {
            let data = view.get_type_data(id);
            if data.len() >= 32 {
                let dd = decode_definition_data(data);
                let identifier = view.get_str(dd.identifier).to_string();
                let url = view.get_str(dd.url).to_string();
                let title = if dd.title.len > 0 {
                    Some(view.get_str(dd.title).to_string())
                } else {
                    None
                };
                defs.push(Definition { identifier, url, title });
            }
        }
    }
    defs
}

fn find_def<'a>(defs: &'a [Definition], identifier: &str) -> Option<&'a Definition> {
    defs.iter().find(|d| d.identifier == identifier)
}

// ---------------------------------------------------------------------------
// Prop helper types
// ---------------------------------------------------------------------------

/// Pre-built property data: refs already interned in the builder's string pool.
struct PropData {
    name_ref: StringRef,
    value_kind: u8,
    value_ref: StringRef,
}

fn build_props(builder: &mut ArenaBuilder, specs: &[(&str, u8, StringRef)]) -> Vec<PropData> {
    specs
        .iter()
        .map(|&(name, kind, value_ref)| {
            let name_ref = builder.alloc_string(name);
            PropData { name_ref, value_kind: kind, value_ref }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Element open/leaf helpers
// ---------------------------------------------------------------------------

/// Open a HAST_ELEMENT node with the given tag and no properties.
fn open_element(builder: &mut ArenaBuilder, tag: &str) {
    builder.open_node_raw(HAST_ELEMENT);
    let tag_ref = builder.alloc_string(tag);
    let encoded = crate::codec::encode_element_data(tag_ref, &[]);
    builder.set_data_current(&encoded);
}

/// Open a HAST_ELEMENT node with the given tag and pre-built properties.
fn open_element_with_props(builder: &mut ArenaBuilder, tag: &str, props: &[PropData]) {
    builder.open_node_raw(HAST_ELEMENT);
    let tag_ref = builder.alloc_string(tag);
    let prop_tuples: Vec<(StringRef, u8, StringRef)> = props
        .iter()
        .map(|p| (p.name_ref, p.value_kind, p.value_ref))
        .collect();
    let encoded = crate::codec::encode_element_data(tag_ref, &prop_tuples);
    builder.set_data_current(&encoded);
}

/// Add a void (self-closing) HAST_ELEMENT with no properties.
fn add_void_element(builder: &mut ArenaBuilder, tag: &str) {
    builder.open_node_raw(HAST_ELEMENT);
    let tag_ref = builder.alloc_string(tag);
    let encoded = crate::codec::encode_element_data(tag_ref, &[]);
    builder.set_data_current(&encoded);
    builder.close_node();
}

/// Add a void (self-closing) HAST_ELEMENT with pre-built properties.
fn add_void_element_with_props(builder: &mut ArenaBuilder, tag: &str, props: &[PropData]) {
    builder.open_node_raw(HAST_ELEMENT);
    let tag_ref = builder.alloc_string(tag);
    let prop_tuples: Vec<(StringRef, u8, StringRef)> = props
        .iter()
        .map(|p| (p.name_ref, p.value_kind, p.value_ref))
        .collect();
    let encoded = crate::codec::encode_element_data(tag_ref, &prop_tuples);
    builder.set_data_current(&encoded);
    builder.close_node();
}

/// Add a HAST_TEXT leaf node with the given string.
fn add_text_node(builder: &mut ArenaBuilder, text: &str) {
    let text_ref = builder.alloc_string(text);
    let leaf_id = builder.add_leaf_raw(HAST_TEXT);
    builder.arena_mut().set_type_data(leaf_id, &encode_text_data(text_ref));
}

/// Add a HAST_RAW leaf node with the given string.
fn add_raw_node(builder: &mut ArenaBuilder, html: &str) {
    let html_ref = builder.alloc_string(html);
    let leaf_id = builder.add_leaf_raw(HAST_RAW);
    builder.arena_mut().set_type_data(leaf_id, &encode_text_data(html_ref));
}

// ---------------------------------------------------------------------------
// Node conversion
// ---------------------------------------------------------------------------

fn convert_node(node_id: u32, view: &ArenaView, builder: &mut ArenaBuilder, defs: &[Definition]) {
    let node = view.get_node(node_id);
    let raw_type = node.node_type;

    match NodeType::from_u8(raw_type) {
        Some(NodeType::Root) => {
            builder.open_node_raw(HAST_ROOT);
            convert_children(node_id, view, builder, defs);
            builder.close_node();
        }

        Some(NodeType::Paragraph) => {
            open_element(builder, "p");
            convert_children(node_id, view, builder, defs);
            builder.close_node();
        }

        Some(NodeType::Heading) => {
            let data = view.get_type_data(node_id);
            let depth = if data.is_empty() {
                1
            } else {
                decode_heading_data(data).depth
            };
            let tag = match depth {
                1 => "h1",
                2 => "h2",
                3 => "h3",
                4 => "h4",
                5 => "h5",
                _ => "h6",
            };
            open_element(builder, tag);
            convert_children(node_id, view, builder, defs);
            builder.close_node();
        }

        Some(NodeType::ThematicBreak) => {
            add_void_element(builder, "hr");
        }

        Some(NodeType::Blockquote) => {
            open_element(builder, "blockquote");
            convert_children(node_id, view, builder, defs);
            builder.close_node();
        }

        Some(NodeType::List) => {
            let data = view.get_type_data(node_id);
            let list_data = decode_list_data(data);
            let tag = if list_data.ordered { "ol" } else { "ul" };
            if list_data.ordered && list_data.start != 1 {
                let start_str = list_data.start.to_string();
                let start_ref = builder.alloc_string(&start_str);
                let props = build_props(builder, &[("start", PROP_STRING, start_ref)]);
                open_element_with_props(builder, tag, &props);
            } else {
                open_element(builder, tag);
            }
            convert_children(node_id, view, builder, defs);
            builder.close_node();
        }

        Some(NodeType::ListItem) => {
            open_element(builder, "li");
            let data = view.get_type_data(node_id);
            if !data.is_empty() {
                let item_data = decode_list_item_data(data);
                if item_data.checked != 2 {
                    // Task list item — add disabled checkbox
                    let type_ref = builder.alloc_string("checkbox");
                    if item_data.checked == 1 {
                        let props = build_props(builder, &[
                            ("type", PROP_STRING, type_ref),
                            ("disabled", PROP_BOOL_TRUE, StringRef::empty()),
                            ("checked", PROP_BOOL_TRUE, StringRef::empty()),
                        ]);
                        add_void_element_with_props(builder, "input", &props);
                    } else {
                        let props = build_props(builder, &[
                            ("type", PROP_STRING, type_ref),
                            ("disabled", PROP_BOOL_TRUE, StringRef::empty()),
                        ]);
                        add_void_element_with_props(builder, "input", &props);
                    }
                }
            }
            convert_children(node_id, view, builder, defs);
            builder.close_node();
        }

        Some(NodeType::Html) => {
            let data = view.get_type_data(node_id);
            let string_ref = decode_string_ref_data(data);
            let html = view.get_str(string_ref).to_string();
            add_raw_node(builder, &html);
        }

        Some(NodeType::Code) => {
            let data = view.get_type_data(node_id);
            let code_data = decode_code_data(data);
            let value = view.get_str(code_data.value).to_string();

            open_element(builder, "pre");
            if code_data.lang.len > 0 {
                let lang = view.get_str(code_data.lang).to_string();
                let class_val = format!("language-{}", lang);
                let class_ref = builder.alloc_string(&class_val);
                let props = build_props(builder, &[("class", PROP_SPACE_SEP, class_ref)]);
                open_element_with_props(builder, "code", &props);
            } else {
                open_element(builder, "code");
            }
            add_text_node(builder, &value);
            builder.close_node(); // code
            builder.close_node(); // pre
        }

        Some(NodeType::Text) => {
            let data = view.get_type_data(node_id);
            let string_ref = decode_string_ref_data(data);
            let text = view.get_str(string_ref).to_string();
            add_text_node(builder, &text);
        }

        Some(NodeType::Emphasis) => {
            open_element(builder, "em");
            convert_children(node_id, view, builder, defs);
            builder.close_node();
        }

        Some(NodeType::Strong) => {
            open_element(builder, "strong");
            convert_children(node_id, view, builder, defs);
            builder.close_node();
        }

        Some(NodeType::InlineCode) => {
            let data = view.get_type_data(node_id);
            let string_ref = decode_string_ref_data(data);
            let code = view.get_str(string_ref).to_string();
            open_element(builder, "code");
            add_text_node(builder, &code);
            builder.close_node();
        }

        Some(NodeType::Break) => {
            add_void_element(builder, "br");
        }

        Some(NodeType::Link) => {
            let data = view.get_type_data(node_id);
            let link_data = decode_link_data(data);
            let url = view.get_str(link_data.url).to_string();
            let url_ref = builder.alloc_string(&url);
            if link_data.title.len > 0 {
                let title = view.get_str(link_data.title).to_string();
                let title_ref = builder.alloc_string(&title);
                let props = build_props(builder, &[
                    ("href", PROP_STRING, url_ref),
                    ("title", PROP_STRING, title_ref),
                ]);
                open_element_with_props(builder, "a", &props);
            } else {
                let props = build_props(builder, &[("href", PROP_STRING, url_ref)]);
                open_element_with_props(builder, "a", &props);
            }
            convert_children(node_id, view, builder, defs);
            builder.close_node();
        }

        Some(NodeType::Image) => {
            let data = view.get_type_data(node_id);
            let img_data = decode_image_data(data);
            let url = view.get_str(img_data.url).to_string();
            let alt = view.get_str(img_data.alt).to_string();
            let url_ref = builder.alloc_string(&url);
            let alt_ref = builder.alloc_string(&alt);
            if img_data.title.len > 0 {
                let title = view.get_str(img_data.title).to_string();
                let title_ref = builder.alloc_string(&title);
                let props = build_props(builder, &[
                    ("src", PROP_STRING, url_ref),
                    ("alt", PROP_STRING, alt_ref),
                    ("title", PROP_STRING, title_ref),
                ]);
                add_void_element_with_props(builder, "img", &props);
            } else {
                let props = build_props(builder, &[
                    ("src", PROP_STRING, url_ref),
                    ("alt", PROP_STRING, alt_ref),
                ]);
                add_void_element_with_props(builder, "img", &props);
            }
        }

        Some(NodeType::Delete) => {
            open_element(builder, "del");
            convert_children(node_id, view, builder, defs);
            builder.close_node();
        }

        Some(NodeType::Table) => {
            open_element(builder, "table");
            let child_ids = view.get_children(node_id).to_vec();
            if !child_ids.is_empty() {
                open_element(builder, "thead");
                convert_table_row(child_ids[0], view, builder, defs, true);
                builder.close_node(); // thead

                if child_ids.len() > 1 {
                    open_element(builder, "tbody");
                    for &row_id in &child_ids[1..] {
                        convert_table_row(row_id, view, builder, defs, false);
                    }
                    builder.close_node(); // tbody
                }
            }
            builder.close_node(); // table
        }

        Some(NodeType::Math) => {
            let data = view.get_type_data(node_id);
            let math_data = decode_math_data(data);
            let value = view.get_str(math_data.value).to_string();
            let class_ref = builder.alloc_string("language-math math-display");
            let props = build_props(builder, &[("class", PROP_SPACE_SEP, class_ref)]);
            open_element(builder, "pre");
            open_element_with_props(builder, "code", &props);
            add_text_node(builder, &value);
            builder.close_node(); // code
            builder.close_node(); // pre
        }

        Some(NodeType::InlineMath) => {
            let data = view.get_type_data(node_id);
            let string_ref = decode_string_ref_data(data);
            let value = view.get_str(string_ref).to_string();
            let class_ref = builder.alloc_string("language-math math-inline");
            let props = build_props(builder, &[("class", PROP_SPACE_SEP, class_ref)]);
            open_element_with_props(builder, "code", &props);
            add_text_node(builder, &value);
            builder.close_node();
        }

        Some(NodeType::Definition)
        | Some(NodeType::Yaml)
        | Some(NodeType::Toml)
        | Some(NodeType::FootnoteDefinition) => {
            // No HAST output
        }

        Some(NodeType::LinkReference) => {
            let data = view.get_type_data(node_id);
            if data.len() >= 20 {
                let rd = decode_reference_data(data);
                let identifier = view.get_str(rd.identifier).to_string();
                if let Some(def) = find_def(defs, &identifier) {
                    let url = def.url.clone();
                    let url_ref = builder.alloc_string(&url);
                    if let Some(ref title) = def.title {
                        let title_ref = builder.alloc_string(title);
                        let props = build_props(builder, &[
                            ("href", PROP_STRING, url_ref),
                            ("title", PROP_STRING, title_ref),
                        ]);
                        open_element_with_props(builder, "a", &props);
                    } else {
                        let props = build_props(builder, &[("href", PROP_STRING, url_ref)]);
                        open_element_with_props(builder, "a", &props);
                    }
                    convert_children(node_id, view, builder, defs);
                    builder.close_node();
                } else {
                    // Unresolved: output children as-is
                    convert_children(node_id, view, builder, defs);
                }
            }
        }

        Some(NodeType::ImageReference) => {
            let data = view.get_type_data(node_id);
            if data.len() >= 20 {
                let rd = decode_reference_data(data);
                let identifier = view.get_str(rd.identifier).to_string();
                if let Some(def) = find_def(defs, &identifier) {
                    let alt = extract_text_content(node_id, view);
                    let url = def.url.clone();
                    let url_ref = builder.alloc_string(&url);
                    let alt_ref = builder.alloc_string(&alt);
                    if let Some(ref title) = def.title {
                        let title_ref = builder.alloc_string(title);
                        let props = build_props(builder, &[
                            ("src", PROP_STRING, url_ref),
                            ("alt", PROP_STRING, alt_ref),
                            ("title", PROP_STRING, title_ref),
                        ]);
                        add_void_element_with_props(builder, "img", &props);
                    } else {
                        let props = build_props(builder, &[
                            ("src", PROP_STRING, url_ref),
                            ("alt", PROP_STRING, alt_ref),
                        ]);
                        add_void_element_with_props(builder, "img", &props);
                    }
                }
            }
        }

        Some(NodeType::FootnoteReference) => {
            // Skip for now
        }

        _ => {
            // Unknown/MDX: recurse into children
            convert_children(node_id, view, builder, defs);
        }
    }
}

fn convert_children(node_id: u32, view: &ArenaView, builder: &mut ArenaBuilder, defs: &[Definition]) {
    let children = view.get_children(node_id).to_vec();
    for child_id in children {
        convert_node(child_id, view, builder, defs);
    }
}

fn convert_table_row(
    row_id: u32,
    view: &ArenaView,
    builder: &mut ArenaBuilder,
    defs: &[Definition],
    is_header: bool,
) {
    open_element(builder, "tr");
    let cell_ids = view.get_children(row_id).to_vec();
    let cell_tag = if is_header { "th" } else { "td" };
    for cell_id in cell_ids {
        open_element(builder, cell_tag);
        convert_children(cell_id, view, builder, defs);
        builder.close_node();
    }
    builder.close_node(); // tr
}

fn extract_text_content(node_id: u32, view: &ArenaView) -> String {
    let mut out = String::new();
    extract_text_recursive(node_id, view, &mut out);
    out
}

fn extract_text_recursive(node_id: u32, view: &ArenaView, out: &mut String) {
    let node = view.get_node(node_id);
    if node.node_type == NodeType::Text as u8 {
        let data = view.get_type_data(node_id);
        if !data.is_empty() {
            let sr = decode_string_ref_data(data);
            out.push_str(view.get_str(sr));
        }
    }
    for &child_id in view.get_children(node_id) {
        extract_text_recursive(child_id, view, out);
    }
}
