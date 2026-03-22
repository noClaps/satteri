//! HAST node type constants and property kind constants.

pub const HAST_ROOT: u8 = 0;
pub const HAST_ELEMENT: u8 = 1;
pub const HAST_TEXT: u8 = 2;
pub const HAST_COMMENT: u8 = 3;
pub const HAST_DOCTYPE: u8 = 4;
pub const HAST_RAW: u8 = 5;

pub const PROP_STRING: u8 = 0;
pub const PROP_BOOL_TRUE: u8 = 1;
pub const PROP_BOOL_FALSE: u8 = 2;
pub const PROP_SPACE_SEP: u8 = 3;
pub const PROP_COMMA_SEP: u8 = 4;
