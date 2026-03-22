//! Codec helpers for HAST type-specific data encoding/decoding.
//!
//! Element type_data layout:
//!   [tag_name: StringRef(8B)][prop_count: u32(4B)][_pad: u32(4B)] = 16-byte header
//!   then prop_count * PropertyEntry (20 bytes each):
//!     [name: StringRef(8B)][value_type: u8(1B)][_pad: [u8;3](3B)][value: StringRef(8B)]
//!
//! Text/Comment/Raw type_data: just StringRef (8 bytes).

use mdast_arena::StringRef;

// ---------------------------------------------------------------------------
// StringRef encode/decode
// ---------------------------------------------------------------------------

pub fn encode_string_ref(sr: StringRef) -> [u8; 8] {
    let mut out = [0u8; 8];
    out[0..4].copy_from_slice(&sr.offset.to_le_bytes());
    out[4..8].copy_from_slice(&sr.len.to_le_bytes());
    out
}

pub fn decode_string_ref(data: &[u8]) -> StringRef {
    assert!(data.len() >= 8, "need at least 8 bytes for StringRef");
    let offset = u32::from_le_bytes(data[0..4].try_into().unwrap());
    let len = u32::from_le_bytes(data[4..8].try_into().unwrap());
    StringRef::new(offset, len)
}

// ---------------------------------------------------------------------------
// Element encode/decode
// ---------------------------------------------------------------------------

/// Encode element type_data.
/// props: slice of (name: StringRef, value_type: u8, value: StringRef)
pub fn encode_element_data(tag_name: StringRef, props: &[(StringRef, u8, StringRef)]) -> Vec<u8> {
    // 16-byte header: tag_name(8) + prop_count(4) + _pad(4)
    let mut out = Vec::with_capacity(16 + props.len() * 20);

    out.extend_from_slice(&encode_string_ref(tag_name));
    out.extend_from_slice(&(props.len() as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes()); // _pad

    // Property entries: 20 bytes each
    for &(name, value_type, value) in props {
        out.extend_from_slice(&encode_string_ref(name));
        out.push(value_type);
        out.extend_from_slice(&[0u8; 3]); // _pad
        out.extend_from_slice(&encode_string_ref(value));
    }

    out
}

/// Decode the tag name StringRef from element type_data.
pub fn decode_element_tag(data: &[u8]) -> StringRef {
    decode_string_ref(&data[0..8])
}

/// Decode the property count from element type_data.
pub fn decode_element_prop_count(data: &[u8]) -> u32 {
    u32::from_le_bytes(data[8..12].try_into().unwrap())
}

/// Decode a property entry by index from element type_data.
/// Returns (name: StringRef, value_type: u8, value: StringRef).
pub fn decode_element_prop(data: &[u8], index: u32) -> (StringRef, u8, StringRef) {
    let base = 16 + index as usize * 20;
    let name = decode_string_ref(&data[base..base + 8]);
    let value_type = data[base + 8];
    let value = decode_string_ref(&data[base + 12..base + 20]);
    (name, value_type, value)
}

// ---------------------------------------------------------------------------
// Text/Comment/Raw encode/decode — just a StringRef
// ---------------------------------------------------------------------------

pub fn encode_text_data(sr: StringRef) -> Vec<u8> {
    encode_string_ref(sr).to_vec()
}

pub fn decode_text_data(data: &[u8]) -> StringRef {
    decode_string_ref(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_ref_round_trip() {
        let sr = StringRef::new(42, 10);
        let encoded = encode_string_ref(sr);
        let decoded = decode_string_ref(&encoded);
        assert_eq!(decoded.offset, 42);
        assert_eq!(decoded.len, 10);
    }

    #[test]
    fn element_no_props() {
        let tag = StringRef::new(0, 3);
        let data = encode_element_data(tag, &[]);
        assert_eq!(data.len(), 16);
        assert_eq!(decode_element_tag(&data).offset, 0);
        assert_eq!(decode_element_tag(&data).len, 3);
        assert_eq!(decode_element_prop_count(&data), 0);
    }

    #[test]
    fn element_with_props() {
        let tag = StringRef::new(0, 1);
        let name = StringRef::new(5, 4);
        let value = StringRef::new(10, 6);
        let props = vec![(name, crate::node_types::PROP_STRING, value)];
        let data = encode_element_data(tag, &props);
        assert_eq!(data.len(), 36); // 16 + 20
        assert_eq!(decode_element_prop_count(&data), 1);
        let (n, kind, v) = decode_element_prop(&data, 0);
        assert_eq!(n.offset, 5);
        assert_eq!(n.len, 4);
        assert_eq!(kind, crate::node_types::PROP_STRING);
        assert_eq!(v.offset, 10);
        assert_eq!(v.len, 6);
    }

    #[test]
    fn text_data_round_trip() {
        let sr = StringRef::new(100, 20);
        let data = encode_text_data(sr);
        let decoded = decode_text_data(&data);
        assert_eq!(decoded.offset, 100);
        assert_eq!(decoded.len, 20);
    }
}
