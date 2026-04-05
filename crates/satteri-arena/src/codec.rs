//! Generic codec helpers for arena type-data encoding.

use crate::node::StringRef;

pub fn encode_string_ref_data(sr: StringRef) -> Vec<u8> {
    sr.as_bytes().to_vec()
}

pub fn decode_string_ref_data(bytes: &[u8]) -> StringRef {
    StringRef::from_bytes(bytes)
}
