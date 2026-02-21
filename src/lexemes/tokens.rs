use std::fmt::Debug;

use crate::{charvec::CharVec, spans::*};
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    Str(CharVec),
    At,
    Dollar,
    Float,
    Int,
    Identifier,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Equal,
    Dot,
    Lesser,
    Greater,
    Comma,
    Colon,
    Bang,
    Percent,
    False,
    True,
    Ampersand,
    Pipe,
    Null,
    Question,
    Eof,
    Comment,
    End,
}

impl TokenType {
    pub fn to_token(self, span: Span) -> Token {
        Token::new(self, span)
    }
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub kind: TokenType,
    pub span: Span,
}
impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Token::{kind:?}[{span:?}]",
            kind = self.kind,
            span = self.span
        )
    }
}

pub trait TokenEq {
    fn is(&self, kind: &TokenType) -> bool;
    fn is_any(&self, matches: &[TokenType]) -> bool;
    fn isnt(&self, kind: &TokenType) -> bool;
}
impl Token {
    pub fn new(kind: TokenType, span: Span) -> Self {
        Token { kind, span }
    }
    pub fn exists(&self) -> bool {
        return self.isnt(&TokenType::Eof);
    }
}
impl TokenEq for Token {
    fn is_any(&self, matches: &[TokenType]) -> bool {
        matches.contains(&self.kind)
    }
    fn is(&self, kind: &TokenType) -> bool {
        &self.kind == kind
    }
    fn isnt(&self, kind: &TokenType) -> bool {
        &self.kind != kind
    }
}

impl TokenEq for Option<Token> {
    fn is_any(&self, matches: &[TokenType]) -> bool {
        let Some(kind) = self else {
            return false;
        };
        kind.is_any(matches)
    }
    fn is(&self, kind: &TokenType) -> bool {
        match self.clone() {
            Some(tok) => &tok.kind == kind,
            None => false,
        }
    }
    fn isnt(&self, kind: &TokenType) -> bool {
        match self.clone() {
            Some(tok) => &tok.kind != kind,
            None => false,
        }
    }
}
pub fn map_keyword(text: &str) -> Option<TokenType> {
    let res = match text {
        "true" => TokenType::True,
        "false" => TokenType::False,
        "null" => TokenType::Null,

        _ => return None,
    };
    Some(res)
}
