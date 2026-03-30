//! `tryckeri` — high-level Rust API for the Tryckeri markdown/MDX pipeline.
//!
//! # Quick start
//!
//! ```
//! let html = tryckeri::markdown_to_html("# Hello world");
//! assert!(html.contains("<h1>Hello world</h1>"));
//! ```

/// Parse Markdown source and render it directly to HTML.
pub fn markdown_to_html(source: &str) -> String {
    let (arena, _) = tryckeri_parser::parse(source, &tryckeri_parser::ParseOptions::default());
    tryckeri_hast::mdast_to_html(&arena)
}

/// Compile MDX source directly to JavaScript.
pub fn compile_mdx(source: &str, options: &tryckeri_mdxjs::Options) -> Result<String, String> {
    tryckeri_mdxjs::compile(source, options).map_err(|e| e.to_string())
}
