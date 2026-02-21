use ariadne::Report;

use crate::lang_errors::{LangError, MsgBuilder};
use crate::lexemes::tokens::TokenType;
use crate::spans::{Span, Spanned};
#[derive(Clone, Debug)]
pub enum ParseError {
    Unspecified(String),
    InvalidToken(TokenType, TokenType),
    UnexpectedToken(TokenType),
    UnexpectedStreamEnd,
}

impl LangError for Spanned<ParseError> {
    fn msg(&'_ self) -> Report<'_, Span> {
        use ParseError as Pe;
        match &self.item {
            Pe::InvalidToken(expected, got) => {
                MsgBuilder::build_err(format!("Invalid Token {got:?}"), self.span)
                    .with_err_label(format!("Expected this token to be {expected:?}."))
                    .finish()
            }

            Pe::UnexpectedToken(got) => {
                MsgBuilder::build_err(format!("Unexpected token {got:?}"), self.span)
                    .with_err_label("This should not be here.")
                    .finish()
            }
            Pe::UnexpectedStreamEnd => {
                MsgBuilder::build_err("Unexpected end of token stream", self.span)
                    .with_err_label("Expected more tokens here.")
                    .finish()
            }
            Pe::Unspecified(err) => MsgBuilder::build_unspecified_err(err.to_string(), self.span),
        }
    }
}
