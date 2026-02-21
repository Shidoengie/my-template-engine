use std::fmt::Debug;

use crate::{charvec::CharVec, spans::*};
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    Str(CharVec),
    At,
    Dollar,
    Float,
    Int,
    Word,
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
    Space,
    NewLine,
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
    /// Determines if the token is of kind in `kind`.
    fn is(&self, kind: &TokenType) -> bool;
    /// Determines if the token matches any of kinds in `matches`.
    fn is_any(&self, matches: impl AsRef<[TokenType]>) -> bool;
    /// Determines if the token is significant.
    ///
    /// Tokens that are of kind [`TokenType::Space`], [`TokenType::Space`], [`TokenType::NewLine`] arent significant.
    fn is_significant(&self) -> bool {
        self.isnt_any(&[TokenType::Comment, TokenType::Space, TokenType::NewLine])
    }
    /// Inverse of [`Self::is`].
    fn isnt(&self, kind: &TokenType) -> bool {
        return !self.is(kind);
    }
    /// Inverse of [`Self::is_significant`].
    fn isnt_significant(&self) -> bool {
        !self.is_significant()
    }
    /// Inverse of [`Self::is_any`].
    fn isnt_any(&self, matches: impl AsRef<[TokenType]>) -> bool {
        return !self.is_any(matches);
    }
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
    fn is_any(&self, matches: impl AsRef<[TokenType]>) -> bool {
        matches.as_ref().contains(&self.kind)
    }
    fn is(&self, kind: &TokenType) -> bool {
        &self.kind == kind
    }
}

impl TokenEq for Option<Token> {
    fn is_any(&self, matches: impl AsRef<[TokenType]>) -> bool {
        let Some(kind) = self else {
            return false;
        };
        kind.is_any(matches.as_ref())
    }
    fn is(&self, kind: &TokenType) -> bool {
        match self.clone() {
            Some(tok) => &tok.kind == kind,
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
