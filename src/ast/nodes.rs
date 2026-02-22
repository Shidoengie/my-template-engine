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
