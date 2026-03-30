//! Parity tests: the in-memory and binary-buffer HTML serialization paths
//! must produce identical output for all supported Markdown constructs.

use tryckeri_hast::{hast_buffer_to_html, mdast_to_hast_buffer, mdast_to_html};

/// Parse `md` through both serialization paths and assert identical HTML output.
///
/// Path A (in-memory): parse → mdast_to_html
/// Path B (binary):    parse → to_raw_buffer → mdast_to_hast_buffer → hast_buffer_to_html
fn assert_parity(md: &str) {
    let (arena, _errors) = tryckeri_parser::parse(md, &tryckeri_parser::ParseOptions::default());

    // Path A: in-memory
    let html_memory = mdast_to_html(&arena);

    // Path B: binary buffer round-trip
    let mdast_buf = arena.to_raw_buffer();
    let hast_buf = mdast_to_hast_buffer(&mdast_buf).expect("mdast_to_hast_buffer failed");
    let html_binary = hast_buffer_to_html(&hast_buf).expect("hast_buffer_to_html failed");

    assert_eq!(
        html_memory, html_binary,
        "Parity mismatch for input:\n---\n{md}\n---\nIn-memory HTML:\n{html_memory}\nBinary HTML:\n{html_binary}"
    );
}

#[test]
fn parity_heading_h1() {
    assert_parity("# Heading 1");
}

#[test]
fn parity_heading_h2() {
    assert_parity("## Heading 2");
}

#[test]
fn parity_heading_h6() {
    assert_parity("###### Heading 6");
}

#[test]
fn parity_paragraph() {
    assert_parity("A simple paragraph.");
}

#[test]
fn parity_emphasis() {
    assert_parity("Some *emphasized* text.");
}

#[test]
fn parity_strong() {
    assert_parity("Some **strong** text.");
}

#[test]
fn parity_emphasis_and_strong() {
    assert_parity("Both ***bold and italic*** text.");
}

#[test]
fn parity_inline_code() {
    assert_parity("Use `inline code` here.");
}

#[test]
fn parity_link_without_title() {
    assert_parity("[example](https://example.com)");
}

#[test]
fn parity_link_with_title() {
    assert_parity("[example](https://example.com \"Example Title\")");
}

#[test]
fn parity_image_without_title() {
    assert_parity("![alt text](image.png)");
}

#[test]
fn parity_image_with_title() {
    assert_parity("![alt text](image.png \"Image Title\")");
}

#[test]
fn parity_fenced_code_block() {
    assert_parity("```rust\nfn main() {}\n```");
}

#[test]
fn parity_indented_code_block() {
    assert_parity("    fn main() {}");
}

#[test]
fn parity_code_block_no_language() {
    assert_parity("```\nplain code\n```");
}

#[test]
fn parity_blockquote() {
    assert_parity("> This is a blockquote.");
}

#[test]
fn parity_nested_blockquote() {
    assert_parity("> Outer\n>\n> > Inner");
}

#[test]
fn parity_unordered_list() {
    assert_parity("- Item 1\n- Item 2\n- Item 3");
}

#[test]
fn parity_ordered_list() {
    assert_parity("1. First\n2. Second\n3. Third");
}

#[test]
fn parity_nested_list() {
    assert_parity("- Parent\n  - Child\n  - Child 2\n- Parent 2");
}

#[test]
fn parity_table() {
    assert_parity("| A | B |\n|---|---|\n| 1 | 2 |\n| 3 | 4 |");
}

#[test]
fn parity_table_with_alignment() {
    assert_parity("| Left | Center | Right |\n|:-----|:------:|------:|\n| a | b | c |");
}

#[test]
fn parity_thematic_break() {
    assert_parity("---");
}

#[test]
fn parity_hard_line_break() {
    assert_parity("Line one  \nLine two");
}

#[test]
fn parity_text_escaping() {
    assert_parity("a < b & c > d");
}

#[test]
fn parity_nested_emphasis_in_heading() {
    assert_parity("## A *nested* **heading**");
}

#[test]
fn parity_link_in_paragraph() {
    assert_parity("Visit [the site](https://example.com) for more.");
}

#[test]
fn parity_code_in_emphasis() {
    assert_parity("*use `code` here*");
}

#[test]
fn parity_multiple_paragraphs() {
    assert_parity("First paragraph.\n\nSecond paragraph.\n\nThird paragraph.");
}

#[test]
fn parity_blockquote_with_formatting() {
    assert_parity("> A quote with **bold** and *italic* text.");
}

#[test]
fn parity_list_with_paragraphs() {
    assert_parity("- Item one\n\n- Item two\n\n- Item three");
}

#[test]
fn parity_complex_document() {
    assert_parity(
        "\
# Title

A paragraph with **bold**, *italic*, and `code`.

> A blockquote.

- List item 1
- List item 2

---

[Link](https://example.com)",
    );
}
