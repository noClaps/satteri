//! `tryckeri-hast` — HAST conversion and HTML serialization.

pub mod codec;
pub mod convert;
pub(crate) mod html;
pub mod node_types;
pub mod render;
pub mod text_content;

pub use convert::mdast_arena_to_hast_arena;
pub use render::{hast_arena_to_html, render_node};
pub use text_content::text_content;

/// Convert an mdast arena directly to an HTML string.
pub fn mdast_to_html(arena: &tryckeri_arena::Arena) -> String {
    let hast = mdast_arena_to_hast_arena(arena);
    hast_arena_to_html(&hast)
}
