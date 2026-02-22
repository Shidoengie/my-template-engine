use ariadne::{Color, Label, Report};

use crate::lang_errors::{LangError, MsgBuilder};
use crate::lexemes::tokens::TokenType;
use crate::spans::{Span, Spanned};
#[derive(Clone, Debug)]
pub enum ParseError {
    Unspecified(String),
    InvalidToken(TokenType, TokenType),
    UnexpectedToken(TokenType),
    UnmatchedTag {
        start_tag: Spanned<String>,
        end_tag: Spanned<String>,
    },
    UnexpectedStreamEnd,
}

impl LangError for Spanned<ParseError> {
    fn msg(&'_ self) -> Report<'_, Span> {
        use ParseError as Pe;
        match &self.item {
            Pe::InvalidToken(expected, got) => {
                MsgBuilder::build_err(format!("Invalid Token '{got:?}'"), self.span)
                    .with_err_label(format!("Expected this token to be {expected:?}."))
                    .finish()
            }
            Pe::UnmatchedTag { start_tag, end_tag } => {
                let start_tag_name = &start_tag.item;
                let end_tag_name = &end_tag.item;
                MsgBuilder::build_err(
                    format!(
                        "The end tag '{end_tag_name}' does not match the start tag '{end_tag_name}'",
                        
                    ),
                    end_tag.span,
                )
                .get_inner()
                .with_label(
                    Label::new(start_tag.span)
                        .with_color(Color::Red)
                        .with_message("This tag"),
                )
                .with_label(
                    Label::new(end_tag.span)
                        .with_color(Color::Red)
                        .with_message("And this tag"),
                )
                .with_label(
                    Label::new(self.span)
                        .with_color(Color::Red)
                        .with_message("These tags should match."),
                )
                .with_help(format!("Rename '{start_tag_name}' to '{end_tag_name}'."))
                .finish()
            }
            Pe::UnexpectedToken(got) => {
                MsgBuilder::build_err(format!("Unexpected token '{got:?}'"), self.span)
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
