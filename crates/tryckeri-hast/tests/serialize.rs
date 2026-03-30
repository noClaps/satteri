//! Integration tests for HAST → HTML serialization and end-to-end conversion.

use tryckeri_hast::{hast_to_html, mdast_to_html, HastBuilder, Property, PropertyValue};
use tryckeri_mdast::{
    encode_code_data, encode_heading_data, encode_image_data, encode_link_data,
    encode_string_ref_data, MdastBuilder, MdastNodeType, StringRef,
};

#[test]
fn serialize_simple_paragraph() {
    let mut builder = HastBuilder::new();
    builder.open_root();
    builder.open_element("p");
    builder.add_text("Hello");
    builder.close();
    let hast = builder.finish();
    assert_eq!(hast_to_html(&hast), "<p>Hello</p>");
}

#[test]
fn serialize_element_with_attribute() {
    let mut builder = HastBuilder::new();
    builder.open_root();
    let link_id = builder.open_element("a");
    let name = builder.alloc_string("href");
    let val = builder.alloc_string("url");
    builder.set_properties(
        link_id,
        &[Property {
            name,
            value: PropertyValue::String(val),
        }],
    );
    builder.add_text("text");
    builder.close();
    let hast = builder.finish();
    assert_eq!(hast_to_html(&hast), "<a href=\"url\">text</a>");
}

#[test]
fn serialize_boolean_attribute_true() {
    let mut builder = HastBuilder::new();
    builder.open_root();
    let input_id = builder.open_element("input");
    let name = builder.alloc_string("disabled");
    builder.set_properties(
        input_id,
        &[Property {
            name,
            value: PropertyValue::Bool(true),
        }],
    );
    builder.close();
    let hast = builder.finish();
    assert_eq!(hast_to_html(&hast), "<input disabled>");
}

#[test]
fn serialize_boolean_attribute_false_omitted() {
    let mut builder = HastBuilder::new();
    builder.open_root();
    let input_id = builder.open_element("input");
    let name = builder.alloc_string("checked");
    builder.set_properties(
        input_id,
        &[Property {
            name,
            value: PropertyValue::Bool(false),
        }],
    );
    builder.close();
    let hast = builder.finish();
    // false boolean attribute should be omitted entirely
    assert_eq!(hast_to_html(&hast), "<input>");
}

#[test]
fn serialize_void_element_br() {
    let mut builder = HastBuilder::new();
    builder.open_root();
    builder.open_element("br");
    builder.close();
    let hast = builder.finish();
    assert_eq!(hast_to_html(&hast), "<br>");
}

#[test]
fn serialize_void_element_hr() {
    let mut builder = HastBuilder::new();
    builder.open_root();
    builder.open_element("hr");
    builder.close();
    let hast = builder.finish();
    assert_eq!(hast_to_html(&hast), "<hr>");
}

#[test]
fn serialize_text_escaping() {
    let mut builder = HastBuilder::new();
    builder.open_root();
    builder.open_element("p");
    builder.add_text("<foo> & \"bar\"");
    builder.close();
    let hast = builder.finish();
    // Double quotes don't need escaping in text content (only in attributes)
    assert_eq!(hast_to_html(&hast), "<p>&lt;foo&gt; &amp; \"bar\"</p>");
}

#[test]
fn serialize_attribute_escaping() {
    let mut builder = HastBuilder::new();
    builder.open_root();
    let link_id = builder.open_element("a");
    let name = builder.alloc_string("href");
    let val = builder.alloc_string("url?a=1&b=\"2\"");
    builder.set_properties(
        link_id,
        &[Property {
            name,
            value: PropertyValue::String(val),
        }],
    );
    builder.add_text("link");
    builder.close();
    let hast = builder.finish();
    assert_eq!(
        hast_to_html(&hast),
        "<a href=\"url?a=1&amp;b=&quot;2&quot;\">link</a>"
    );
}

#[test]
fn serialize_nested_elements() {
    let mut builder = HastBuilder::new();
    builder.open_root();
    builder.open_element("p");
    builder.open_element("strong");
    builder.add_text("bold");
    builder.close();
    builder.close();
    let hast = builder.finish();
    assert_eq!(hast_to_html(&hast), "<p><strong>bold</strong></p>");
}

#[test]
fn serialize_multiple_children_in_list() {
    let mut builder = HastBuilder::new();
    builder.open_root();
    builder.open_element("ul");
    builder.open_element("li");
    builder.add_text("a");
    builder.close();
    builder.open_element("li");
    builder.add_text("b");
    builder.close();
    builder.close();
    let hast = builder.finish();
    assert_eq!(hast_to_html(&hast), "<ul><li>a</li><li>b</li></ul>");
}

#[test]
fn serialize_comment_node() {
    let mut builder = HastBuilder::new();
    builder.open_root();
    builder.add_comment("comment");
    let hast = builder.finish();
    assert_eq!(hast_to_html(&hast), "<!--comment-->");
}

#[test]
fn serialize_doctype() {
    let mut builder = HastBuilder::new();
    builder.open_root();
    builder.add_doctype();
    let hast = builder.finish();
    assert_eq!(hast_to_html(&hast), "<!doctype html>");
}

#[test]
fn serialize_raw_passthrough_not_escaped() {
    let mut builder = HastBuilder::new();
    builder.open_root();
    builder.add_raw("<div class=\"foo\">raw &amp; content</div>");
    let hast = builder.finish();
    // Raw nodes must not be escaped
    assert_eq!(
        hast_to_html(&hast),
        "<div class=\"foo\">raw &amp; content</div>"
    );
}

#[test]
fn serialize_space_separated_class_property() {
    let mut builder = HastBuilder::new();
    builder.open_root();
    let span_id = builder.open_element("span");
    let name = builder.alloc_string("class");
    let val = builder.alloc_string("foo bar");
    builder.set_properties(
        span_id,
        &[Property {
            name,
            value: PropertyValue::SpaceSeparated(val),
        }],
    );
    builder.add_text("text");
    builder.close();
    let hast = builder.finish();
    assert_eq!(hast_to_html(&hast), "<span class=\"foo bar\">text</span>");
}

fn build_h1_paragraph_mdast() -> tryckeri_mdast::MdastArena {
    let source = "# Hello\n\nWorld".to_string();
    let mut builder = MdastBuilder::new(source);
    builder.open_node(MdastNodeType::Root);
    builder.open_node(MdastNodeType::Heading);
    builder.set_data_current(&encode_heading_data(1));
    builder.open_node(MdastNodeType::Text);
    builder.set_data_current(&encode_string_ref_data(StringRef::new(2, 5))); // "Hello"
    builder.close_node();
    builder.close_node();
    builder.open_node(MdastNodeType::Paragraph);
    builder.open_node(MdastNodeType::Text);
    builder.set_data_current(&encode_string_ref_data(StringRef::new(9, 5))); // "World"
    builder.close_node();
    builder.close_node();
    builder.close_node();
    builder.finish()
}

#[test]
fn e2e_heading_and_paragraph() {
    let mdast = build_h1_paragraph_mdast();
    let html = mdast_to_html(&mdast);
    assert_eq!(html, "<h1>Hello</h1>\n<p>World</p>");
}

#[test]
fn e2e_link() {
    // "[click](https://example.com)"
    let source = "[click](https://example.com)".to_string();
    let mut builder = MdastBuilder::new(source);
    builder.open_node(MdastNodeType::Root);
    builder.open_node(MdastNodeType::Paragraph);
    builder.open_node(MdastNodeType::Link);
    builder.set_data_current(&encode_link_data(
        StringRef::new(8, 19), // "https://example.com"
        StringRef::empty(),
    ));
    builder.open_node(MdastNodeType::Text);
    builder.set_data_current(&encode_string_ref_data(StringRef::new(1, 5))); // "click"
    builder.close_node();
    builder.close_node();
    builder.close_node();
    builder.close_node();
    let mdast = builder.finish();

    let html = mdast_to_html(&mdast);
    assert_eq!(html, "<p><a href=\"https://example.com\">click</a></p>");
}

#[test]
fn e2e_emphasis() {
    let source = "em".to_string();
    let mut builder = MdastBuilder::new(source);
    builder.open_node(MdastNodeType::Root);
    builder.open_node(MdastNodeType::Paragraph);
    builder.open_node(MdastNodeType::Emphasis);
    builder.open_node(MdastNodeType::Text);
    builder.set_data_current(&encode_string_ref_data(StringRef::new(0, 2))); // "em"
    builder.close_node();
    builder.close_node();
    builder.close_node();
    builder.close_node();
    let mdast = builder.finish();

    let html = mdast_to_html(&mdast);
    assert_eq!(html, "<p><em>em</em></p>");
}

#[test]
fn e2e_code_block_with_language() {
    // Code block: lang="rust", value="fn main() {}"
    let source = "rust\nfn main() {}".to_string();
    let mut builder = MdastBuilder::new(source);
    builder.open_node(MdastNodeType::Root);
    builder.open_node(MdastNodeType::Code);
    builder.set_data_current(&encode_code_data(
        StringRef::new(0, 4),  // lang: "rust"
        StringRef::empty(),    // meta: empty
        StringRef::new(5, 12), // value: "fn main() {}"
        b'`',
    ));
    builder.close_node();
    builder.close_node();
    let mdast = builder.finish();

    let html = mdast_to_html(&mdast);
    assert_eq!(
        html,
        "<pre><code class=\"language-rust\">fn main() {}</code></pre>"
    );
}

#[test]
fn e2e_image() {
    let source = "![alt](img.png)".to_string();
    let mut builder = MdastBuilder::new(source);
    builder.open_node(MdastNodeType::Root);
    builder.open_node(MdastNodeType::Paragraph);
    builder.open_node(MdastNodeType::Image);
    builder.set_data_current(&encode_image_data(
        StringRef::new(7, 7), // "img.png"
        StringRef::new(2, 3), // "alt"
        StringRef::empty(),
    ));
    builder.close_node();
    builder.close_node();
    builder.close_node();
    let mdast = builder.finish();

    let html = mdast_to_html(&mdast);
    assert_eq!(html, "<p><img src=\"img.png\" alt=\"alt\"></p>");
}
