//! `mdast-arena` — arena-allocated MDAST node structures.
//!
//! This crate provides:
//! - `NodeType` enum and `ArenaNode` struct
//! - `StringRef` for zero-copy string references into the source text
//! - `Arena` for owning all nodes and associated data
//! - `ArenaBuilder` for incremental tree construction
//! - `LineIndex` for offset→(line, column) conversion
//! - Raw buffer export/import for Phase 2's zero-copy transfer layer
//! - Codec helpers for encoding/decoding type-specific data structs

pub mod arena;
pub mod builder;
pub mod codec;
pub mod line_index;
pub mod node;
pub mod raw_buffer;
pub mod read_arena;
pub mod mdx_types;
pub mod rebuild;

// Flat re-exports for convenience.
pub use arena::Arena;
pub use builder::ArenaBuilder;
pub use codec::{
    CodeData, ColumnAlign, DefinitionData, ExpressionData, FootnoteDefinitionData, HeadingData,
    ImageData, LinkData, ListData, ListItemData, MathData, MdxJsxElementData, ReferenceData,
    TableData,
    decode_code_data, decode_definition_data, decode_expression_data,
    decode_footnote_definition_data, decode_heading_data, decode_image_data, decode_link_data,
    decode_list_data, decode_list_item_data, decode_math_data, decode_mdx_jsx_element_data,
    decode_reference_data, decode_string_ref_data, decode_table_data,
    encode_code_data, encode_definition_data, encode_expression_data,
    encode_footnote_definition_data, encode_heading_data, encode_image_data, encode_link_data,
    encode_list_data, encode_list_item_data, encode_math_data, encode_mdx_jsx_element_data,
    encode_reference_data, encode_string_ref_data, encode_table_data,
};
pub use line_index::LineIndex;
pub use node::{ArenaNode, NodeType, StringRef, NODE_STRUCT_SIZE};
pub use raw_buffer::{ArenaView, BufferError, BufferHeader, BUFFER_MAGIC, BUFFER_VERSION};
pub use read_arena::ReadArena;
pub use rebuild::{Patch, rebuild};
