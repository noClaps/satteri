//! Arena → HAST conversion.

use mdast_arena::codec::{
    decode_code_data, decode_heading_data, decode_image_data, decode_link_data,
    decode_list_data, decode_list_item_data, decode_string_ref_data,
};
use mdast_arena::{Arena, NodeType};

use crate::node::{HastArena, HastBuilder, Property, PropertyValue};

/// Convert an arena directly to a HAST arena.
pub fn arena_to_hast(arena: &Arena) -> HastArena {
    let mut builder = HastBuilder::new();
    builder.open_root();

    let root_children = arena.get_children(0).to_vec();
    for child_id in root_children {
        convert_node(child_id, arena, &mut builder);
    }

    builder.finish()
}

fn convert_node(node_id: u32, arena: &Arena, builder: &mut HastBuilder) {
    let node = arena.get_node(node_id);
    let node_type = match NodeType::from_u8(node.node_type) {
        Some(t) => t,
        None => return,
    };

    match node_type {
        NodeType::Paragraph => {
            builder.open_element("p");
            convert_children(node_id, arena, builder);
            builder.close();
        }

        NodeType::Heading => {
            let data = arena.get_type_data(node_id);
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
            builder.open_element(tag);
            convert_children(node_id, arena, builder);
            builder.close();
        }

        NodeType::ThematicBreak => {
            builder.open_element("hr");
            builder.close();
        }

        NodeType::Blockquote => {
            builder.open_element("blockquote");
            convert_children(node_id, arena, builder);
            builder.close();
        }

        NodeType::List => {
            let data = arena.get_type_data(node_id);
            let list_data = decode_list_data(data);
            let tag = if list_data.ordered { "ol" } else { "ul" };
            let elem_id = builder.open_element(tag);

            if list_data.ordered && list_data.start != 1 {
                builder.set_properties(
                    elem_id,
                    vec![Property {
                        name: "start".to_string(),
                        value: PropertyValue::String(list_data.start.to_string()),
                    }],
                );
            }

            convert_children(node_id, arena, builder);
            builder.close();
        }

        NodeType::ListItem => {
            builder.open_element("li");
            let data = arena.get_type_data(node_id);
            if !data.is_empty() {
                let item_data = decode_list_item_data(data);
                if item_data.checked != 2 {
                    // Task list item — add checkbox
                    let checkbox_id = builder.open_element("input");
                    let mut props = vec![
                        Property {
                            name: "type".to_string(),
                            value: PropertyValue::String("checkbox".to_string()),
                        },
                        Property {
                            name: "disabled".to_string(),
                            value: PropertyValue::Bool(true),
                        },
                    ];
                    if item_data.checked == 1 {
                        props.push(Property {
                            name: "checked".to_string(),
                            value: PropertyValue::Bool(true),
                        });
                    }
                    builder.set_properties(checkbox_id, props);
                    builder.close(); // input
                }
            }
            convert_children(node_id, arena, builder);
            builder.close();
        }

        NodeType::Html => {
            let data = arena.get_type_data(node_id);
            let string_ref = decode_string_ref_data(data);
            let html = arena.get_str(string_ref).to_string();
            builder.add_raw(html);
        }

        NodeType::Code => {
            let data = arena.get_type_data(node_id);
            let code_data = decode_code_data(data);

            builder.open_element("pre");
            let code_id = builder.open_element("code");

            if code_data.lang.len > 0 {
                let lang = arena.get_str(code_data.lang).to_string();
                builder.set_properties(
                    code_id,
                    vec![Property {
                        name: "class".to_string(),
                        value: PropertyValue::SpaceSeparated(vec![format!("language-{}", lang)]),
                    }],
                );
            }

            let value = arena.get_str(code_data.value).to_string();
            builder.add_text(value);
            builder.close(); // code
            builder.close(); // pre
        }

        NodeType::Definition => {
            // Definitions don't produce HAST output
        }

        NodeType::Text => {
            let data = arena.get_type_data(node_id);
            let string_ref = decode_string_ref_data(data);
            let text = arena.get_str(string_ref).to_string();
            builder.add_text(text);
        }

        NodeType::Emphasis => {
            builder.open_element("em");
            convert_children(node_id, arena, builder);
            builder.close();
        }

        NodeType::Strong => {
            builder.open_element("strong");
            convert_children(node_id, arena, builder);
            builder.close();
        }

        NodeType::InlineCode => {
            let data = arena.get_type_data(node_id);
            let string_ref = decode_string_ref_data(data);
            let code = arena.get_str(string_ref).to_string();
            builder.open_element("code");
            builder.add_text(code);
            builder.close();
        }

        NodeType::Break => {
            builder.open_element("br");
            builder.close();
        }

        NodeType::Link => {
            let data = arena.get_type_data(node_id);
            let link_data = decode_link_data(data);
            let url = arena.get_str(link_data.url).to_string();

            let elem_id = builder.open_element("a");
            let mut props = vec![Property {
                name: "href".to_string(),
                value: PropertyValue::String(url),
            }];
            if link_data.title.len > 0 {
                let title = arena.get_str(link_data.title).to_string();
                props.push(Property {
                    name: "title".to_string(),
                    value: PropertyValue::String(title),
                });
            }
            builder.set_properties(elem_id, props);
            convert_children(node_id, arena, builder);
            builder.close();
        }

        NodeType::Image => {
            let data = arena.get_type_data(node_id);
            let img_data = decode_image_data(data);
            let url = arena.get_str(img_data.url).to_string();
            let alt = arena.get_str(img_data.alt).to_string();

            let elem_id = builder.open_element("img");
            let mut props = vec![
                Property {
                    name: "src".to_string(),
                    value: PropertyValue::String(url),
                },
                Property {
                    name: "alt".to_string(),
                    value: PropertyValue::String(alt),
                },
            ];
            if img_data.title.len > 0 {
                let title = arena.get_str(img_data.title).to_string();
                props.push(Property {
                    name: "title".to_string(),
                    value: PropertyValue::String(title),
                });
            }
            builder.set_properties(elem_id, props);
            builder.close();
        }

        NodeType::Table => {
            builder.open_element("table");
            let child_ids = arena.get_children(node_id).to_vec();
            if !child_ids.is_empty() {
                builder.open_element("thead");
                convert_table_row(child_ids[0], arena, builder, true);
                builder.close(); // thead

                if child_ids.len() > 1 {
                    builder.open_element("tbody");
                    for &row_id in &child_ids[1..] {
                        convert_table_row(row_id, arena, builder, false);
                    }
                    builder.close(); // tbody
                }
            }
            builder.close(); // table
        }

        NodeType::Delete => {
            builder.open_element("del");
            convert_children(node_id, arena, builder);
            builder.close();
        }

        NodeType::FootnoteDefinition
        | NodeType::FootnoteReference
        | NodeType::LinkReference
        | NodeType::ImageReference => {
            // Skip for Phase 7 — reference resolution requires a pre-pass
        }

        _ => {
            // Unknown or unhandled: recurse into children
            convert_children(node_id, arena, builder);
        }
    }
}

fn convert_children(node_id: u32, arena: &Arena, builder: &mut HastBuilder) {
    let children = arena.get_children(node_id).to_vec();
    for child_id in children {
        convert_node(child_id, arena, builder);
    }
}

fn convert_table_row(
    row_id: u32,
    arena: &Arena,
    builder: &mut HastBuilder,
    is_header: bool,
) {
    builder.open_element("tr");
    let cell_ids = arena.get_children(row_id).to_vec();
    let cell_tag = if is_header { "th" } else { "td" };
    for cell_id in cell_ids {
        builder.open_element(cell_tag);
        convert_children(cell_id, arena, builder);
        builder.close();
    }
    builder.close(); // tr
}
