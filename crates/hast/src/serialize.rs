//! HAST → HTML serialization.

use crate::node::{HastArena, HastNodeType, PropertyValue};

/// Serialize a HAST arena to an HTML string.
pub fn hast_to_html(hast: &HastArena) -> String {
    let mut out = String::with_capacity(256);
    serialize_node(0, hast, &mut out);
    out
}

fn serialize_node(node_id: u32, hast: &HastArena, out: &mut String) {
    let node = hast.get_node(node_id);

    match node.node_type {
        HastNodeType::Root => {
            for &child_id in hast.get_children(node_id) {
                serialize_node(child_id, hast, out);
            }
        }

        HastNodeType::Element => {
            let tag = node.tag_name.as_deref().unwrap_or("div");
            let is_void = is_void_element(tag);

            out.push('<');
            out.push_str(tag);

            let props = hast.get_properties(node_id);
            for prop in props {
                if prop.value.is_bool_false() {
                    continue;
                }
                out.push(' ');
                out.push_str(&prop.name);
                match &prop.value {
                    PropertyValue::Bool(true) => {
                        // boolean attribute: just the name, no value
                    }
                    _ => {
                        out.push_str("=\"");
                        let val = prop.value.to_html_string();
                        out.push_str(&escape_attribute(&val));
                        out.push('"');
                    }
                }
            }

            if is_void {
                out.push_str(" />");
            } else {
                out.push('>');
                for &child_id in hast.get_children(node_id) {
                    serialize_node(child_id, hast, out);
                }
                out.push_str("</");
                out.push_str(tag);
                out.push('>');
            }
        }

        HastNodeType::Text => {
            if let Some(text) = &node.value {
                out.push_str(&escape_text(text));
            }
        }

        HastNodeType::Raw => {
            if let Some(html) = &node.value {
                out.push_str(html);
            }
        }

        HastNodeType::Comment => {
            if let Some(text) = &node.value {
                out.push_str("<!--");
                out.push_str(text);
                out.push_str("-->");
            }
        }

        HastNodeType::Doctype => {
            out.push_str("<!doctype html>");
        }
    }
}

/// HTML void elements (self-closing, no children)
fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input"
            | "link" | "meta" | "param" | "source" | "track" | "wbr"
    )
}

/// Escape text content for HTML
fn escape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

/// Escape attribute values for HTML
fn escape_attribute(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}
