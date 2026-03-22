//! HTML syntax tree: [hast][].
//!
//! [hast]: https://github.com/syntax-tree/hast
#![allow(dead_code)]
#![allow(clippy::to_string_trait_impl)]

extern crate alloc;

#[allow(unused_imports)]
pub use mdast_arena::mdx_types::MdxJsxAttribute;
pub use mdast_arena::mdx_types::{AttributeContent, AttributeValue, Stop};
use mdast_arena::mdx_types::Position;

/// Nodes.
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serializable",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", rename_all = "camelCase")
)]
pub enum Node {
    /// Root.
    Root(Root),
    /// Element.
    Element(Element),
    /// Document type.
    Doctype(Doctype),
    /// Comment.
    Comment(Comment),
    /// Text.
    Text(Text),
    // MDX being passed through.
    /// MDX: JSX flow element (block-level, e.g. `<Foo>\n\n...\n\n</Foo>`).
    /// Serializes as `"mdxJsxFlowElement"` matching the @mdx-js/mdx ecosystem.
    #[cfg_attr(
        feature = "serializable",
        serde(rename = "mdxJsxFlowElement", alias = "mdxJsxElement")
    )]
    MdxJsxElement(MdxJsxElement),
    /// MDX: JSX text element (inline, e.g. `text <Foo /> text`).
    /// Serializes as `"mdxJsxTextElement"` matching the @mdx-js/mdx ecosystem.
    #[cfg_attr(feature = "serializable", serde(rename = "mdxJsxTextElement"))]
    MdxJsxTextElement(MdxJsxElement),
    /// MDX.js ESM.
    MdxjsEsm(MdxjsEsm),
    // MDX: expression.
    MdxExpression(MdxExpression),
}

impl alloc::fmt::Debug for Node {
    /// Debug the wrapped struct.
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        match self {
            Node::Root(x) => write!(f, "{x:?}"),
            Node::Element(x) => write!(f, "{x:?}"),
            Node::Doctype(x) => write!(f, "{x:?}"),
            Node::Comment(x) => write!(f, "{x:?}"),
            Node::Text(x) => write!(f, "{x:?}"),
            Node::MdxJsxElement(x) | Node::MdxJsxTextElement(x) => write!(f, "{x:?}"),
            Node::MdxExpression(x) => write!(f, "{x:?}"),
            Node::MdxjsEsm(x) => write!(f, "{x:?}"),
        }
    }
}

/// Turn a slice of hast nodes into a string.
fn children_to_string(children: &[Node]) -> String {
    children.iter().map(ToString::to_string).collect()
}

impl ToString for Node {
    /// Turn a hast node into a string.
    fn to_string(&self) -> String {
        match self {
            // Parents.
            Node::Root(x) => children_to_string(&x.children),
            Node::Element(x) => children_to_string(&x.children),
            Node::MdxJsxElement(x) | Node::MdxJsxTextElement(x) => children_to_string(&x.children),
            // Literals.
            Node::Comment(x) => x.value.clone(),
            Node::Text(x) => x.value.clone(),
            Node::MdxExpression(x) => x.value.clone(),
            Node::MdxjsEsm(x) => x.value.clone(),
            // Voids.
            Node::Doctype(_) => String::new(),
        }
    }
}

impl Node {
    /// Get children of a hast node.
    #[must_use]
    pub fn children(&self) -> Option<&Vec<Node>> {
        match self {
            // Parent.
            Node::Root(x) => Some(&x.children),
            Node::Element(x) => Some(&x.children),
            Node::MdxJsxElement(x) | Node::MdxJsxTextElement(x) => Some(&x.children),
            // Non-parent.
            _ => None,
        }
    }

    /// Get children of a hast node, mutably.
    pub fn children_mut(&mut self) -> Option<&mut Vec<Node>> {
        match self {
            // Parent.
            Node::Root(x) => Some(&mut x.children),
            Node::Element(x) => Some(&mut x.children),
            Node::MdxJsxElement(x) | Node::MdxJsxTextElement(x) => Some(&mut x.children),
            // Non-parent.
            _ => None,
        }
    }

    /// Get the position of a hast node.
    pub fn position(&self) -> Option<&Position> {
        match self {
            Node::Root(x) => x.position.as_ref(),
            Node::Element(x) => x.position.as_ref(),
            Node::Doctype(x) => x.position.as_ref(),
            Node::Comment(x) => x.position.as_ref(),
            Node::Text(x) => x.position.as_ref(),
            Node::MdxJsxElement(x) | Node::MdxJsxTextElement(x) => x.position.as_ref(),
            Node::MdxExpression(x) => x.position.as_ref(),
            Node::MdxjsEsm(x) => x.position.as_ref(),
        }
    }

    /// Get the position of a hast node, mutably.
    pub fn position_mut(&mut self) -> Option<&mut Position> {
        match self {
            Node::Root(x) => x.position.as_mut(),
            Node::Element(x) => x.position.as_mut(),
            Node::Doctype(x) => x.position.as_mut(),
            Node::Comment(x) => x.position.as_mut(),
            Node::Text(x) => x.position.as_mut(),
            Node::MdxJsxElement(x) | Node::MdxJsxTextElement(x) => x.position.as_mut(),
            Node::MdxExpression(x) => x.position.as_mut(),
            Node::MdxjsEsm(x) => x.position.as_mut(),
        }
    }

    /// Set the position of a hast node.
    pub fn position_set(&mut self, position: Option<Position>) {
        match self {
            Node::Root(x) => x.position = position,
            Node::Element(x) => x.position = position,
            Node::Doctype(x) => x.position = position,
            Node::Comment(x) => x.position = position,
            Node::Text(x) => x.position = position,
            Node::MdxJsxElement(x) | Node::MdxJsxTextElement(x) => x.position = position,
            Node::MdxExpression(x) => x.position = position,
            Node::MdxjsEsm(x) => x.position = position,
        }
    }
}

/// Document.
///
/// ```html
/// > | a
///     ^
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serializable",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct Root {
    // Parent.
    /// Content model.
    pub children: Vec<Node>,
    /// Positional info.
    pub position: Option<Position>,
}

/// Document type.
///
/// ```html
/// > | <!doctype html>
///     ^^^^^^^^^^^^^^^
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serializable",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct Element {
    /// Tag name.
    pub tag_name: String,
    /// Properties — serialized as a JSON object so it is compatible with
    /// the standard hast property format used by rehype plugins.
    #[cfg_attr(
        feature = "serializable",
        serde(
            serialize_with = "serialize_properties",
            deserialize_with = "deserialize_properties"
        )
    )]
    pub properties: Vec<(String, PropertyValue)>,
    // Parent.
    /// Children.
    pub children: Vec<Node>,
    /// Positional info.
    pub position: Option<Position>,
}

/// Property value.
///
/// Covers the full hast `Properties` value type:
/// `boolean | number | string | null | undefined | Array<string | number>`
///
/// `null` and `undefined` are represented as `Null` and are skipped when
/// serializing/deserializing (the custom helpers in this module handle them).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serializable", derive(serde::Serialize), serde(untagged))]
pub enum PropertyValue {
    /// A boolean.
    Boolean(bool),
    /// A number (e.g. `tabIndex: 0`, `rowSpan: 2`).
    Number(f64),
    /// A string.
    String(String),
    /// A comma-separated list of strings/numbers.
    CommaSeparated(Vec<String>),
    /// A space-separated list of strings/numbers.
    SpaceSeparated(Vec<String>),
    /// Null / undefined — treated as absent.
    Null,
}

impl PartialEq for PropertyValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PropertyValue::Boolean(a), PropertyValue::Boolean(b)) => a == b,
            (PropertyValue::Number(a), PropertyValue::Number(b)) => {
                // NaN != NaN by IEEE 754, but for AST equality treat identical bit patterns as equal.
                a.to_bits() == b.to_bits()
            }
            (PropertyValue::String(a), PropertyValue::String(b)) => a == b,
            (PropertyValue::CommaSeparated(a), PropertyValue::CommaSeparated(b)) => a == b,
            (PropertyValue::SpaceSeparated(a), PropertyValue::SpaceSeparated(b)) => a == b,
            (PropertyValue::Null, PropertyValue::Null) => true,
            _ => false,
        }
    }
}

impl Eq for PropertyValue {}

/// Manual `Deserialize` for `PropertyValue` that handles the full hast property
/// value domain (bool, number, string, null, array of string|number).
#[cfg(feature = "serializable")]
impl<'de> serde::Deserialize<'de> for PropertyValue {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{SeqAccess, Visitor};
        use std::fmt;

        struct PVVisitor;

        impl<'de> Visitor<'de> for PVVisitor {
            type Value = PropertyValue;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a boolean, number, string, null, or array of strings/numbers")
            }

            fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(PropertyValue::Boolean(v))
            }

            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(PropertyValue::Number(v as f64))
            }

            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(PropertyValue::Number(v as f64))
            }

            fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Self::Value, E> {
                Ok(PropertyValue::Number(v))
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(PropertyValue::String(v.to_owned()))
            }

            fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(PropertyValue::String(v))
            }

            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                // JSON null
                Ok(PropertyValue::Null)
            }

            fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(PropertyValue::Null)
            }

            fn visit_some<D2: serde::Deserializer<'de>>(
                self,
                d: D2,
            ) -> Result<Self::Value, D2::Error> {
                d.deserialize_any(PVVisitor)
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                // Array<string | number> — deserialize each element as a nested PropertyValue
                // then coerce to string.
                let mut items: Vec<String> = Vec::new();
                while let Some(item) = seq.next_element::<PropertyValue>()? {
                    let s = match item {
                        PropertyValue::String(s) => s,
                        PropertyValue::Number(n) => {
                            if n.fract() == 0.0 && n.is_finite() {
                                format!("{}", n as i64)
                            } else {
                                format!("{n}")
                            }
                        }
                        PropertyValue::Boolean(b) => b.to_string(),
                        PropertyValue::Null => continue,
                        PropertyValue::CommaSeparated(v) => v.join(", "),
                        PropertyValue::SpaceSeparated(v) => v.join(" "),
                    };
                    items.push(s);
                }
                Ok(PropertyValue::SpaceSeparated(items))
            }
        }

        deserializer.deserialize_any(PVVisitor)
    }
}

/// Document type.
///
/// ```html
/// > | <!doctype html>
///     ^^^^^^^^^^^^^^^
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serializable",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct Doctype {
    // Void.
    /// Positional info.
    pub position: Option<Position>,
}

/// Comment.
///
/// ```html
/// > | <!-- a -->
///     ^^^^^^^^^^
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serializable",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct Comment {
    // Text.
    /// Content model.
    pub value: String,
    /// Positional info.
    pub position: Option<Position>,
}

/// Text.
///
/// ```html
/// > | a
///     ^
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serializable",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct Text {
    // Text.
    /// Content model.
    pub value: String,
    /// Positional info.
    pub position: Option<Position>,
}

/// MDX: JSX element.
///
/// ```markdown
/// > | <a />
///     ^^^^^
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serializable",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct MdxJsxElement {
    // JSX element.
    /// Name.
    ///
    /// Fragments have no name.
    pub name: Option<String>,
    /// Attributes.
    pub attributes: Vec<AttributeContent>,
    // Parent.
    /// Content model.
    pub children: Vec<Node>,
    /// Positional info.
    pub position: Option<Position>,
}

/// MDX: expression.
///
/// ```markdown
/// > | {a}
///     ^^^
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serializable",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct MdxExpression {
    // Literal.
    /// Content model.
    pub value: String,
    /// Positional info.
    pub position: Option<Position>,

    /// Custom data on where each slice of `value` came from.
    #[cfg_attr(feature = "serializable", serde(default, rename = "_markdownRsStops"))]
    pub stops: Vec<Stop>,
}

/// MDX: ESM.
///
/// ```markdown
/// > | import a from 'b'
///     ^^^^^^^^^^^^^^^^^
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serializable",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct MdxjsEsm {
    // Literal.
    /// Content model.
    pub value: String,
    /// Positional info.
    pub position: Option<Position>,

    /// Custom data on where each slice of `value` came from.
    /// Optional on deserialization since injected ESM nodes (from rehype plugins
    /// like rehypeApplyFrontmatterExport) don't include this field.
    #[cfg_attr(feature = "serializable", serde(default, rename = "_markdownRsStops"))]
    pub stops: Vec<Stop>,
}

// ── Custom serde helpers for Element::properties ─────────────────────────────

/// Serialize `Vec<(String, PropertyValue)>` as a JSON object.
///
/// Standard hast consumers (rehype plugins) expect `properties` to be a plain
/// `{ key: value }` object, not an array of pairs.
/// `Null` values are omitted (they represent absent/undefined properties).
#[cfg(feature = "serializable")]
fn serialize_properties<S>(
    props: &Vec<(String, PropertyValue)>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeMap;
    let non_null_count = props
        .iter()
        .filter(|(_, v)| !matches!(v, PropertyValue::Null))
        .count();
    let mut map = serializer.serialize_map(Some(non_null_count))?;
    for (key, value) in props {
        if !matches!(value, PropertyValue::Null) {
            map.serialize_entry(key, value)?;
        }
    }
    map.end()
}

/// Deserialize a JSON object into `Vec<(String, PropertyValue)>`.
///
/// Entries whose value is `null` or `undefined` are silently dropped, matching
/// hast semantics where a null property is equivalent to the attribute being absent.
#[cfg(feature = "serializable")]
fn deserialize_properties<'de, D>(deserializer: D) -> Result<Vec<(String, PropertyValue)>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{MapAccess, Visitor};
    use std::fmt;

    struct PropertiesVisitor;

    impl<'de> Visitor<'de> for PropertiesVisitor {
        type Value = Vec<(String, PropertyValue)>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map of property name to value")
        }

        fn visit_map<M: MapAccess<'de>>(self, mut access: M) -> Result<Self::Value, M::Error> {
            let mut props = Vec::new();
            while let Some(key) = access.next_key::<String>()? {
                let value: PropertyValue = access.next_value()?;
                // Skip null/undefined properties (they are absent in HTML).
                if !matches!(value, PropertyValue::Null) {
                    props.push((key, value));
                }
            }
            Ok(props)
        }
    }

    deserializer.deserialize_map(PropertiesVisitor)
}

// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use mdast_arena::mdx_types::Position;
    use pretty_assertions::assert_eq;

    // Literals.

    #[test]
    fn text() {
        let mut node = Node::Text(Text {
            value: "a".into(),
            position: None,
        });

        assert_eq!(
            format!("{:?}", node),
            "Text { value: \"a\", position: None }",
            "should support `Debug`"
        );
        assert_eq!(node.to_string(), "a", "should support `ToString`");
        assert_eq!(node.children_mut(), None, "should support `children_mut`");
        assert_eq!(node.children(), None, "should support `children`");
        assert_eq!(node.position(), None, "should support `position`");
        assert_eq!(node.position_mut(), None, "should support `position`");
        node.position_set(Some(Position::new(1, 1, 0, 1, 2, 1)));
        assert_eq!(
            format!("{:?}", node),
            "Text { value: \"a\", position: Some(1:1-1:2 (0-1)) }",
            "should support `position_set`"
        );
    }

    #[test]
    fn comment() {
        let mut node = Node::Comment(Comment {
            value: "a".into(),
            position: None,
        });

        assert_eq!(
            format!("{:?}", node),
            "Comment { value: \"a\", position: None }",
            "should support `Debug`"
        );
        assert_eq!(node.to_string(), "a", "should support `ToString`");
        assert_eq!(node.children_mut(), None, "should support `children_mut`");
        assert_eq!(node.children(), None, "should support `children`");
        assert_eq!(node.position(), None, "should support `position`");
        assert_eq!(node.position_mut(), None, "should support `position`");
        node.position_set(Some(Position::new(1, 1, 0, 1, 2, 1)));
        assert_eq!(
            format!("{:?}", node),
            "Comment { value: \"a\", position: Some(1:1-1:2 (0-1)) }",
            "should support `position_set`"
        );
    }

    #[test]
    fn mdx_expression() {
        let mut node = Node::MdxExpression(MdxExpression {
            value: "a".into(),
            stops: vec![],
            position: None,
        });

        assert_eq!(
            format!("{:?}", node),
            "MdxExpression { value: \"a\", position: None, stops: [] }",
            "should support `Debug`"
        );
        assert_eq!(node.to_string(), "a", "should support `ToString`");
        assert_eq!(node.children_mut(), None, "should support `children_mut`");
        assert_eq!(node.children(), None, "should support `children`");
        assert_eq!(node.position(), None, "should support `position`");
        assert_eq!(node.position_mut(), None, "should support `position`");
        node.position_set(Some(Position::new(1, 1, 0, 1, 2, 1)));
        assert_eq!(
            format!("{:?}", node),
            "MdxExpression { value: \"a\", position: Some(1:1-1:2 (0-1)), stops: [] }",
            "should support `position_set`"
        );
    }

    #[test]
    fn mdxjs_esm() {
        let mut node = Node::MdxjsEsm(MdxjsEsm {
            value: "a".into(),
            stops: vec![],
            position: None,
        });

        assert_eq!(
            format!("{:?}", node),
            "MdxjsEsm { value: \"a\", position: None, stops: [] }",
            "should support `Debug`"
        );
        assert_eq!(node.to_string(), "a", "should support `ToString`");
        assert_eq!(node.children_mut(), None, "should support `children_mut`");
        assert_eq!(node.children(), None, "should support `children`");
        assert_eq!(node.position(), None, "should support `position`");
        assert_eq!(node.position_mut(), None, "should support `position`");
        node.position_set(Some(Position::new(1, 1, 0, 1, 2, 1)));
        assert_eq!(
            format!("{:?}", node),
            "MdxjsEsm { value: \"a\", position: Some(1:1-1:2 (0-1)), stops: [] }",
            "should support `position_set`"
        );
    }

    // Voids.

    #[test]
    fn doctype() {
        let mut node = Node::Doctype(Doctype { position: None });

        assert_eq!(
            format!("{:?}", node),
            "Doctype { position: None }",
            "should support `Debug`"
        );
        assert_eq!(node.to_string(), "", "should support `ToString`");
        assert_eq!(node.children_mut(), None, "should support `children_mut`");
        assert_eq!(node.children(), None, "should support `children`");
        assert_eq!(node.position(), None, "should support `position`");
        assert_eq!(node.position_mut(), None, "should support `position`");
        node.position_set(Some(Position::new(1, 1, 0, 1, 2, 1)));
        assert_eq!(
            format!("{:?}", node),
            "Doctype { position: Some(1:1-1:2 (0-1)) }",
            "should support `position_set`"
        );
    }

    // Parents.

    #[test]
    fn root() {
        let mut node = Node::Root(Root {
            position: None,
            children: vec![],
        });

        assert_eq!(
            format!("{:?}", node),
            "Root { children: [], position: None }",
            "should support `Debug`"
        );
        assert_eq!(node.to_string(), "", "should support `ToString`");
        assert_eq!(
            node.children_mut(),
            Some(&mut vec![]),
            "should support `children_mut`"
        );
        assert_eq!(node.children(), Some(&vec![]), "should support `children`");
        assert_eq!(node.position(), None, "should support `position`");
        assert_eq!(node.position_mut(), None, "should support `position`");
        node.position_set(Some(Position::new(1, 1, 0, 1, 2, 1)));
        assert_eq!(
            format!("{:?}", node),
            "Root { children: [], position: Some(1:1-1:2 (0-1)) }",
            "should support `position_set`"
        );
    }

    #[test]
    fn element() {
        let mut node = Node::Element(Element {
            tag_name: "a".into(),
            properties: vec![],
            position: None,
            children: vec![],
        });

        assert_eq!(
            format!("{:?}", node),
            "Element { tag_name: \"a\", properties: [], children: [], position: None }",
            "should support `Debug`"
        );
        assert_eq!(node.to_string(), "", "should support `ToString`");
        assert_eq!(
            node.children_mut(),
            Some(&mut vec![]),
            "should support `children_mut`"
        );
        assert_eq!(node.children(), Some(&vec![]), "should support `children`");
        assert_eq!(node.position(), None, "should support `position`");
        assert_eq!(node.position_mut(), None, "should support `position`");
        node.position_set(Some(Position::new(1, 1, 0, 1, 2, 1)));
        assert_eq!(
            format!("{:?}", node),
            "Element { tag_name: \"a\", properties: [], children: [], position: Some(1:1-1:2 (0-1)) }",
            "should support `position_set`"
        );
    }

    #[test]
    fn mdx_jsx_element() {
        let mut node = Node::MdxJsxElement(MdxJsxElement {
            name: None,
            attributes: vec![],
            position: None,
            children: vec![],
        });

        assert_eq!(
            format!("{:?}", node),
            "MdxJsxElement { name: None, attributes: [], children: [], position: None }",
            "should support `Debug`"
        );
        assert_eq!(node.to_string(), "", "should support `ToString`");
        assert_eq!(
            node.children_mut(),
            Some(&mut vec![]),
            "should support `children_mut`"
        );
        assert_eq!(node.children(), Some(&vec![]), "should support `children`");
        assert_eq!(node.position(), None, "should support `position`");
        assert_eq!(node.position_mut(), None, "should support `position`");
        node.position_set(Some(Position::new(1, 1, 0, 1, 2, 1)));
        assert_eq!(
            format!("{:?}", node),
            "MdxJsxElement { name: None, attributes: [], children: [], position: Some(1:1-1:2 (0-1)) }",
            "should support `position_set`"
        );
    }
}
