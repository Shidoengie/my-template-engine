use std::collections::HashMap;

use crate::spans::{Span, Spanned};
#[derive(Debug, Clone)]
pub struct Element {
    pub name: String,
    pub props: HashMap<String, Spanned<Value>>,
    pub children: Vec<Spanned<Node>>,
    pub start_tag_span: Span,
    pub end_tag_span: Option<Span>,
}
#[derive(Debug, Clone)]
pub enum Node {
    Text(String),
    Comment(String),
    Element(Element),
}
#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Null,
    Bool(bool),
    Element,
}
pub trait IntoNodespan {
    fn to_nodespan(self, span: Span) -> Spanned<Node>;
}

macro_rules! nodes_from {
    ($($name:ident)*) => {
        $(
            impl ::core::convert::From<$name> for Node {
                fn from(node: $name) -> Self {
                    Self::$name(node)
                }
            }
            impl IntoNodespan for $name {
                fn to_nodespan(self,span:Span) -> Spanned<Node> {
                    Spanned::new(Node::$name(self),span)
                }

            }
        )*
    }
}
nodes_from!(Element);
pub struct ElementBuilder {
    name: String,
    props: HashMap<String, Spanned<Value>>,
    children: Vec<Spanned<Node>>,
    start_tag_span: Span,
    end_tag_span: Option<Span>,
}
impl ElementBuilder {
    pub fn new(name: impl AsRef<str>, start_tag_span: Span) -> Self {
        ElementBuilder {
            name: name.as_ref().to_owned(),
            props: HashMap::new(),
            children: vec![],
            start_tag_span,
            end_tag_span: None,
        }
    }
    pub fn with_props(self, props: HashMap<String, Spanned<Value>>) -> Self {
        Self { props, ..self }
    }
    pub fn with_children(self, children: Vec<Spanned<Node>>) -> Self {
        Self { children, ..self }
    }

    pub fn with_end_tag_span(self, end_tag_span: Span) -> Self {
        Self {
            end_tag_span: Some(end_tag_span),
            ..self
        }
    }

    pub fn finish(self) -> Element {
        Element {
            name: self.name,
            props: self.props,
            children: self.children,
            start_tag_span: self.start_tag_span,
            end_tag_span: self.end_tag_span,
        }
    }
    pub fn finish_node(self, span: Span) -> Spanned<Node> {
        self.finish().to_nodespan(span)
    }
}
