//! Turn an HTML AST into a JavaScript AST.

use crate::hast;
use crate::oxc::{parse_esm_to_tree, parse_expression_to_tree, serialize};
use crate::oxc_utils::{
    create_jsx_attr_name_from_str, create_jsx_name_from_str, inter_element_whitespace,
    position_to_span,
};
use core::str;
use std::cell::Cell;

use mdast_arena::mdx_types::{self as message, Location, MdxExpressionKind};
use oxc_allocator::{Allocator, Box as OxcBox, Vec as OxcVec};
use oxc_ast::ast::{
    Expression, ExpressionStatement, JSXAttribute, JSXAttributeItem, JSXAttributeValue, JSXChild,
    JSXClosingElement, JSXClosingFragment, JSXElement, JSXEmptyExpression, JSXExpression,
    JSXExpressionContainer, JSXFragment, JSXOpeningElement, JSXOpeningFragment, JSXSpreadAttribute,
    Program, Statement, StringLiteral,
};
use oxc_span::{Atom, Span, SPAN};
use oxc_syntax::node::NodeId;
use rustc_hash::FxHashSet;

/// Result.
pub struct MdxProgram<'a> {
    /// File path.
    pub path: Option<String>,
    /// Allocator that owns all AST data.
    pub allocator: &'a Allocator,
    /// JS AST.
    pub program: Program<'a>,
    /// Comments relating to AST (stored separately since OXC comments are on Program).
    pub comments: Vec<MdxComment>,
}

/// A comment stored outside the OXC AST.
#[derive(Debug, Clone)]
pub struct MdxComment {
    /// Block or line comment.
    pub kind: MdxCommentKind,
    /// Text of the comment.
    pub text: String,
    /// Span of the comment.
    pub span: Span,
}

/// Comment kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MdxCommentKind {
    Block,
    Line,
}

impl MdxProgram<'_> {
    /// Serialize to JS.
    pub fn serialize(&self) -> String {
        serialize(&self.program)
    }
}

/// Whether we're in HTML or SVG.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Space {
    Html,
    Svg,
}

/// Context used to compile hast into OXC's ES AST.
struct Context<'a> {
    space: Space,
    comments: Vec<MdxComment>,
    esm: Vec<Statement<'a>>,
    location: Option<&'a Location>,
    allocator: &'a Allocator,
}

/// Compile hast into OXC's ES AST.
pub fn hast_util_to_oxc<'a>(
    tree: &hast::Node,
    path: Option<String>,
    location: Option<&'a Location>,
    explicit_jsxs: &mut FxHashSet<Span>,
    allocator: &'a Allocator,
) -> Result<MdxProgram<'a>, message::Message> {
    let mut context = Context {
        space: Space::Html,
        comments: vec![],
        esm: vec![],
        location,
        allocator,
    };
    let expr = match one(&mut context, tree, explicit_jsxs)? {
        Some(JSXChild::Fragment(x)) => Some(Expression::JSXFragment(x)),
        Some(JSXChild::Element(x)) => Some(Expression::JSXElement(x)),
        Some(child) => {
            let mut children = OxcVec::with_capacity_in(1, allocator);
            children.push(child);
            Some(Expression::JSXFragment(OxcBox::new_in(
                create_fragment(allocator, children, tree),
                allocator,
            )))
        }
        None => None,
    };

    // Add the ESM.
    let mut body = OxcVec::from_iter_in(context.esm, allocator);

    // We have some content, wrap it.
    if let Some(expr) = expr {
        body.push(Statement::ExpressionStatement(OxcBox::new_in(
            ExpressionStatement {
                node_id: Cell::new(NodeId::DUMMY),
                span: SPAN,
                expression: expr,
            },
            allocator,
        )));
    }

    let program = Program {
        node_id: Cell::new(NodeId::DUMMY),
        span: position_to_span(tree.position()),
        source_type: oxc_span::SourceType::mjs().with_jsx(true),
        source_text: "",
        comments: OxcVec::new_in(allocator),
        hashbang: None,
        directives: OxcVec::new_in(allocator),
        body,
        scope_id: Cell::default(),
    };

    Ok(MdxProgram {
        path,
        allocator,
        program,
        comments: context.comments,
    })
}

/// Transform one node.
fn one<'a>(
    context: &mut Context<'a>,
    node: &hast::Node,
    explicit_jsxs: &mut FxHashSet<Span>,
) -> Result<Option<JSXChild<'a>>, message::Message> {
    let value = match node {
        hast::Node::Comment(x) => Some(transform_comment(context, node, x)),
        hast::Node::Element(x) => transform_element(context, node, x, explicit_jsxs)?,
        hast::Node::MdxJsxElement(x) | hast::Node::MdxJsxTextElement(x) => transform_mdx_jsx_element(context, node, x, explicit_jsxs)?,
        hast::Node::MdxExpression(x) => transform_mdx_expression(context, node, x)?,
        hast::Node::MdxjsEsm(x) => transform_mdxjs_esm(context, node, x)?,
        hast::Node::Root(x) => transform_root(context, node, x, explicit_jsxs)?,
        hast::Node::Text(x) => transform_text(context, node, x),
        hast::Node::Doctype(_) => None,
    };
    Ok(value)
}

/// Transform children of `parent`.
fn all<'a>(
    context: &mut Context<'a>,
    parent: &hast::Node,
    explicit_jsxs: &mut FxHashSet<Span>,
) -> Result<OxcVec<'a, JSXChild<'a>>, message::Message> {
    let mut result = OxcVec::new_in(context.allocator);
    if let Some(children) = parent.children() {
        for child in children {
            if let Some(child) = one(context, child, explicit_jsxs)? {
                result.push(child);
            }
        }
    }

    Ok(result)
}

/// [`Comment`][hast::Comment].
fn transform_comment<'a>(
    context: &mut Context<'a>,
    node: &hast::Node,
    comment: &hast::Comment,
) -> JSXChild<'a> {
    context.comments.push(MdxComment {
        kind: MdxCommentKind::Block,
        text: comment.value.clone(),
        span: position_to_span(node.position()),
    });

    let alloc = context.allocator;
    JSXChild::ExpressionContainer(OxcBox::new_in(
        JSXExpressionContainer {
            node_id: Cell::new(NodeId::DUMMY),
            span: position_to_span(node.position()),
            expression: JSXExpression::EmptyExpression(JSXEmptyExpression {
                node_id: Cell::new(NodeId::DUMMY),
                span: position_to_span(node.position()),
            }),
        },
        alloc,
    ))
}

/// [`Element`][hast::Element].
fn transform_element<'a>(
    context: &mut Context<'a>,
    node: &hast::Node,
    element: &hast::Element,
    explicit_jsxs: &mut FxHashSet<Span>,
) -> Result<Option<JSXChild<'a>>, message::Message> {
    let space = context.space;

    if space == Space::Html && element.tag_name == "svg" {
        context.space = Space::Svg;
    }

    let children = all(context, node, explicit_jsxs)?;
    context.space = space;

    let alloc = context.allocator;
    let mut attrs = OxcVec::new_in(alloc);

    for prop in &element.properties {
        let value = match &prop.1 {
            hast::PropertyValue::Boolean(x) => {
                if *x {
                    None
                } else {
                    continue;
                }
            }
            hast::PropertyValue::Number(x) => {
                // Serialize numbers as string literals (e.g. tabIndex={0} → tabIndex="0").
                // Use integer formatting when the value is a whole number to avoid "1.0" etc.
                let s = if x.fract() == 0.0 && x.is_finite() {
                    format!("{}", *x as i64)
                } else {
                    format!("{x}")
                };
                Some(JSXAttributeValue::StringLiteral(OxcBox::new_in(
                    StringLiteral {
                        node_id: Cell::new(NodeId::DUMMY),
                        span: SPAN,
                        value: Atom::from(alloc.alloc_str(&s)),
                        raw: None,
                        lone_surrogates: false,
                    },
                    alloc,
                )))
            }
            hast::PropertyValue::String(x) => {
                Some(JSXAttributeValue::StringLiteral(OxcBox::new_in(
                    StringLiteral {
                        node_id: Cell::new(NodeId::DUMMY),
                        span: SPAN,
                        value: Atom::from(alloc.alloc_str(x)),
                        raw: None,
                        lone_surrogates: false,
                    },
                    alloc,
                )))
            }
            hast::PropertyValue::CommaSeparated(x) => {
                let joined = x.join(", ");
                Some(JSXAttributeValue::StringLiteral(OxcBox::new_in(
                    StringLiteral {
                        node_id: Cell::new(NodeId::DUMMY),
                        span: SPAN,
                        value: Atom::from(alloc.alloc_str(&joined)),
                        raw: None,
                        lone_surrogates: false,
                    },
                    alloc,
                )))
            }
            hast::PropertyValue::SpaceSeparated(x) => {
                let joined = x.join(" ");
                Some(JSXAttributeValue::StringLiteral(OxcBox::new_in(
                    StringLiteral {
                        node_id: Cell::new(NodeId::DUMMY),
                        span: SPAN,
                        value: Atom::from(alloc.alloc_str(&joined)),
                        raw: None,
                        lone_surrogates: false,
                    },
                    alloc,
                )))
            }
            hast::PropertyValue::Null => {
                // Null/undefined properties are absent — skip them.
                continue;
            }
        };

        let attr_name = prop_to_attr_name(&prop.0);

        attrs.push(JSXAttributeItem::Attribute(OxcBox::new_in(
            JSXAttribute {
                node_id: Cell::new(NodeId::DUMMY),
                span: SPAN,
                name: create_jsx_attr_name_from_str(alloc, &attr_name),
                value,
            },
            alloc,
        )));
    }

    Ok(Some(JSXChild::Element(OxcBox::new_in(
        create_element(alloc, &element.tag_name, attrs, children, node),
        alloc,
    ))))
}

/// [`MdxJsxElement`][hast::MdxJsxElement].
fn transform_mdx_jsx_element<'a>(
    context: &mut Context<'a>,
    node: &hast::Node,
    element: &hast::MdxJsxElement,
    explicit_jsxs: &mut FxHashSet<Span>,
) -> Result<Option<JSXChild<'a>>, message::Message> {
    let space = context.space;

    if let Some(name) = &element.name
        && space == Space::Html
        && name == "svg"
    {
        context.space = Space::Svg;
    }

    let children = all(context, node, explicit_jsxs)?;
    context.space = space;

    let alloc = context.allocator;
    let mut attrs = OxcVec::new_in(alloc);

    for attr_content in &element.attributes {
        let attr = match attr_content {
            hast::AttributeContent::Property(prop) => {
                let value = match prop.value.as_ref() {
                    Some(hast::AttributeValue::Literal(x)) => {
                        Some(JSXAttributeValue::StringLiteral(OxcBox::new_in(
                            StringLiteral {
                                node_id: Cell::new(NodeId::DUMMY),
                                span: SPAN,
                                value: Atom::from(alloc.alloc_str(x)),
                                raw: None,
                                lone_surrogates: false,
                            },
                            alloc,
                        )))
                    }
                    Some(hast::AttributeValue::Expression(expression)) => {
                        let expr = parse_expression_to_tree(
                            &expression.value,
                            &MdxExpressionKind::AttributeValueExpression,
                            &expression.stops,
                            context.location,
                            alloc,
                        )?
                        .unwrap();
                        Some(JSXAttributeValue::ExpressionContainer(OxcBox::new_in(
                            JSXExpressionContainer {
                                node_id: Cell::new(NodeId::DUMMY),
                                span: SPAN,
                                expression: JSXExpression::from(expr),
                            },
                            alloc,
                        )))
                    }
                    None => None,
                };

                JSXAttributeItem::Attribute(OxcBox::new_in(
                    JSXAttribute {
                        node_id: Cell::new(NodeId::DUMMY),
                        span: SPAN,
                        name: create_jsx_attr_name_from_str(alloc, &prop.name),
                        value,
                    },
                    alloc,
                ))
            }
            hast::AttributeContent::Expression(d) => {
                let expr = parse_expression_to_tree(
                    &d.value,
                    &MdxExpressionKind::AttributeExpression,
                    &d.stops,
                    context.location,
                    alloc,
                )?;
                JSXAttributeItem::SpreadAttribute(OxcBox::new_in(
                    JSXSpreadAttribute {
                        node_id: Cell::new(NodeId::DUMMY),
                        span: SPAN,
                        argument: expr.unwrap(),
                    },
                    alloc,
                ))
            }
        };

        attrs.push(attr);
    }

    Ok(Some(if let Some(name) = &element.name {
        explicit_jsxs.insert(position_to_span(node.position()));
        JSXChild::Element(OxcBox::new_in(
            create_element(alloc, name, attrs, children, node),
            alloc,
        ))
    } else {
        JSXChild::Fragment(OxcBox::new_in(
            create_fragment(alloc, children, node),
            alloc,
        ))
    }))
}

/// [`MdxExpression`][hast::MdxExpression].
fn transform_mdx_expression<'a>(
    context: &mut Context<'a>,
    node: &hast::Node,
    expression: &hast::MdxExpression,
) -> Result<Option<JSXChild<'a>>, message::Message> {
    let alloc = context.allocator;
    let expr = parse_expression_to_tree(
        &expression.value,
        &MdxExpressionKind::Expression,
        &expression.stops,
        context.location,
        alloc,
    )?;
    let span = position_to_span(node.position());
    let child = if let Some(expr) = expr {
        JSXExpression::from(expr)
    } else {
        JSXExpression::EmptyExpression(JSXEmptyExpression {
            node_id: Cell::new(NodeId::DUMMY),
            span,
        })
    };

    Ok(Some(JSXChild::ExpressionContainer(OxcBox::new_in(
        JSXExpressionContainer {
            node_id: Cell::new(NodeId::DUMMY),
            expression: child,
            span,
        },
        alloc,
    ))))
}

/// [`MdxjsEsm`][hast::MdxjsEsm].
fn transform_mdxjs_esm<'a>(
    context: &mut Context<'a>,
    _node: &hast::Node,
    esm: &hast::MdxjsEsm,
) -> Result<Option<JSXChild<'a>>, message::Message> {
    let alloc = context.allocator;
    let mut program = parse_esm_to_tree(&esm.value, &esm.stops, context.location, alloc)?;
    let body = std::mem::replace(&mut program.body, OxcVec::new_in(alloc));
    for stmt in body {
        context.esm.push(stmt);
    }
    Ok(None)
}

/// [`Root`][hast::Root].
fn transform_root<'a>(
    context: &mut Context<'a>,
    node: &hast::Node,
    _root: &hast::Root,
    explicit_jsxs: &mut FxHashSet<Span>,
) -> Result<Option<JSXChild<'a>>, message::Message> {
    let alloc = context.allocator;
    let children_vec = all(context, node, explicit_jsxs)?;
    let mut children: Vec<JSXChild<'a>> = children_vec.into_iter().collect();
    let mut queue = vec![];
    let mut nodes = vec![];
    let mut seen = false;

    children.reverse();

    // Remove initial/final whitespace.
    while let Some(child) = children.pop() {
        let mut stash = false;

        if let JSXChild::ExpressionContainer(container) = &child
            && let JSXExpression::StringLiteral(str_lit) = &container.expression
            && inter_element_whitespace(str_lit.value.as_str())
        {
            stash = true;
        }

        if stash {
            if seen {
                queue.push(child);
            }
        } else {
            if !queue.is_empty() {
                nodes.append(&mut queue);
            }
            nodes.push(child);
            seen = true;
        }
    }

    let nodes = OxcVec::from_iter_in(nodes, alloc);

    Ok(Some(JSXChild::Fragment(OxcBox::new_in(
        create_fragment(alloc, nodes, node),
        alloc,
    ))))
}

/// [`Text`][hast::Text].
fn transform_text<'a>(
    context: &mut Context<'a>,
    node: &hast::Node,
    text: &hast::Text,
) -> Option<JSXChild<'a>> {
    if text.value.is_empty() {
        None
    } else {
        let alloc = context.allocator;
        let span = position_to_span(node.position());
        Some(JSXChild::ExpressionContainer(OxcBox::new_in(
            JSXExpressionContainer {
                node_id: Cell::new(NodeId::DUMMY),
                expression: JSXExpression::StringLiteral(OxcBox::new_in(
                    StringLiteral {
                        node_id: Cell::new(NodeId::DUMMY),
                        span,
                        value: Atom::from(alloc.alloc_str(&text.value)),
                        raw: None,
                        lone_surrogates: false,
                    },
                    alloc,
                )),
                span,
            },
            alloc,
        )))
    }
}

/// Create an element.
fn create_element<'a>(
    alloc: &'a Allocator,
    name: &str,
    attrs: OxcVec<'a, JSXAttributeItem<'a>>,
    children: OxcVec<'a, JSXChild<'a>>,
    node: &hast::Node,
) -> JSXElement<'a> {
    let span = position_to_span(node.position());
    let self_closing = children.is_empty();

    JSXElement {
        node_id: Cell::new(NodeId::DUMMY),
        span,
        opening_element: OxcBox::new_in(
            JSXOpeningElement {
                node_id: Cell::new(NodeId::DUMMY),
                span: SPAN,
                name: create_jsx_name_from_str(alloc, name),
                attributes: attrs,
                type_arguments: None,
            },
            alloc,
        ),
        closing_element: if self_closing {
            None
        } else {
            Some(OxcBox::new_in(
                JSXClosingElement {
                    node_id: Cell::new(NodeId::DUMMY),
                    span: SPAN,
                    name: create_jsx_name_from_str(alloc, name),
                },
                alloc,
            ))
        },
        children,
    }
}

/// Create a fragment.
fn create_fragment<'a>(
    _alloc: &'a Allocator,
    children: OxcVec<'a, JSXChild<'a>>,
    node: &hast::Node,
) -> JSXFragment<'a> {
    JSXFragment {
        node_id: Cell::new(NodeId::DUMMY),
        span: position_to_span(node.position()),
        opening_fragment: JSXOpeningFragment {
            node_id: Cell::new(NodeId::DUMMY),
            span: SPAN,
        },
        closing_fragment: JSXClosingFragment {
            node_id: Cell::new(NodeId::DUMMY),
            span: SPAN,
        },
        children,
    }
}

/// Turn a hast property into something that particularly React understands.
fn prop_to_attr_name(prop: &str) -> String {
    // Arbitrary data props, kebab case them.
    if prop.len() > 4 && prop.starts_with("data") {
        let mut result = String::with_capacity(prop.len() + 2);
        let bytes = prop.as_bytes();
        let mut index = 4;
        let mut start = index;
        let mut valid = true;

        result.push_str("data");

        while index < bytes.len() {
            let byte = bytes[index];
            let mut dash = index == 4;

            match byte {
                b'A'..=b'Z' => dash = true,
                b'-' | b'.' | b':' | b'0'..=b'9' | b'a'..=b'z' => {}
                _ => {
                    valid = false;
                    break;
                }
            }

            if dash {
                result.push_str(&prop[start..index]);
                if byte != b'-' {
                    result.push('-');
                }
                result.push(byte.to_ascii_lowercase().into());
                start = index + 1;
            }

            index += 1;
        }

        if valid {
            result.push_str(&prop[start..]);
            return result;
        }
    }

    PROP_TO_REACT_PROP
        .iter()
        .find(|d| d.0 == prop)
        .or_else(|| PROP_TO_ATTR_EXCEPTIONS_SHARED.iter().find(|d| d.0 == prop))
        .map_or_else(|| prop.into(), |d| d.1.into())
}

const PROP_TO_REACT_PROP: [(&str, &str); 17] = [
    ("classId", "classID"),
    ("dataType", "datatype"),
    ("itemId", "itemID"),
    ("strokeDashArray", "strokeDasharray"),
    ("strokeDashOffset", "strokeDashoffset"),
    ("strokeLineCap", "strokeLinecap"),
    ("strokeLineJoin", "strokeLinejoin"),
    ("strokeMiterLimit", "strokeMiterlimit"),
    ("typeOf", "typeof"),
    ("xLinkActuate", "xlinkActuate"),
    ("xLinkArcRole", "xlinkArcrole"),
    ("xLinkHref", "xlinkHref"),
    ("xLinkRole", "xlinkRole"),
    ("xLinkShow", "xlinkShow"),
    ("xLinkTitle", "xlinkTitle"),
    ("xLinkType", "xlinkType"),
    ("xmlnsXLink", "xmlnsXlink"),
];

const PROP_TO_ATTR_EXCEPTIONS_SHARED: [(&str, &str); 48] = [
    ("ariaActiveDescendant", "aria-activedescendant"),
    ("ariaAtomic", "aria-atomic"),
    ("ariaAutoComplete", "aria-autocomplete"),
    ("ariaBusy", "aria-busy"),
    ("ariaChecked", "aria-checked"),
    ("ariaColCount", "aria-colcount"),
    ("ariaColIndex", "aria-colindex"),
    ("ariaColSpan", "aria-colspan"),
    ("ariaControls", "aria-controls"),
    ("ariaCurrent", "aria-current"),
    ("ariaDescribedBy", "aria-describedby"),
    ("ariaDetails", "aria-details"),
    ("ariaDisabled", "aria-disabled"),
    ("ariaDropEffect", "aria-dropeffect"),
    ("ariaErrorMessage", "aria-errormessage"),
    ("ariaExpanded", "aria-expanded"),
    ("ariaFlowTo", "aria-flowto"),
    ("ariaGrabbed", "aria-grabbed"),
    ("ariaHasPopup", "aria-haspopup"),
    ("ariaHidden", "aria-hidden"),
    ("ariaInvalid", "aria-invalid"),
    ("ariaKeyShortcuts", "aria-keyshortcuts"),
    ("ariaLabel", "aria-label"),
    ("ariaLabelledBy", "aria-labelledby"),
    ("ariaLevel", "aria-level"),
    ("ariaLive", "aria-live"),
    ("ariaModal", "aria-modal"),
    ("ariaMultiLine", "aria-multiline"),
    ("ariaMultiSelectable", "aria-multiselectable"),
    ("ariaOrientation", "aria-orientation"),
    ("ariaOwns", "aria-owns"),
    ("ariaPlaceholder", "aria-placeholder"),
    ("ariaPosInSet", "aria-posinset"),
    ("ariaPressed", "aria-pressed"),
    ("ariaReadOnly", "aria-readonly"),
    ("ariaRelevant", "aria-relevant"),
    ("ariaRequired", "aria-required"),
    ("ariaRoleDescription", "aria-roledescription"),
    ("ariaRowCount", "aria-rowcount"),
    ("ariaRowIndex", "aria-rowindex"),
    ("ariaRowSpan", "aria-rowspan"),
    ("ariaSelected", "aria-selected"),
    ("ariaSetSize", "aria-setsize"),
    ("ariaSort", "aria-sort"),
    ("ariaValueMax", "aria-valuemax"),
    ("ariaValueMin", "aria-valuemin"),
    ("ariaValueNow", "aria-valuenow"),
    ("ariaValueText", "aria-valuetext"),
];
