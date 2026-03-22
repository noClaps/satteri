//! `tryckeri-hast` — HAST arena, arena→HAST conversion, and HTML serialization.

pub mod codec;
pub mod convert;
pub mod from_binary;
pub mod node;
pub mod node_types;
pub mod serialize;
pub mod to_binary;

pub use convert::arena_to_hast;
pub use from_binary::hast_buffer_to_html;
pub use node::{HastArena, HastBuilder, HastNode, HastNodeType, Property, PropertyValue};
pub use serialize::hast_to_html;
pub use to_binary::arena_to_hast_buffer;

/// Convert an arena directly to an HTML string.
pub fn arena_to_html(arena: &mdast_arena::Arena) -> String {
    let hast = arena_to_hast(arena);
    hast_to_html(&hast)
}
