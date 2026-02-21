use std::{clone, collections::HashMap};

use crate::{
    ast::nodes::*,
    lang_errors::LangError,
    lexemes::{lexer::Lexer, tokens::*},
    spans::{FileID, IntoSpanned, Spanned},
};

mod error;
use chumsky::{container::Container, prelude::todo};
use error::*;
use rayon::vec;
#[derive(Clone, Debug)]
pub struct Parser<'input> {
    file_id: FileID,
    input: &'input str,
    tokens: Lexer<'input>,
}
pub type Result<T = Spanned<Node>> = std::result::Result<T, Box<dyn LangError>>;
fn err<T>(value: impl LangError + 'static) -> Result<T> {
    Err(Box::new(value))
}
impl<'input> Parser<'input> {
    pub fn new(input: &'input str, file_id: FileID) -> Self {
        Parser {
            file_id,
            input,
            tokens: Lexer::new(input, file_id),
        }
    }
    /// converts token spans into text
    fn text(&mut self, token: &Token) -> String {
        self.input[token.span.start..token.span.end].to_string()
    }

    fn parse_int(&mut self, token: &Token) -> Value {
        let mut text = self.input[token.span.start..token.span.end].to_string();
        let idk: Vec<_> = text.chars().filter(|c| c != &'_').collect();
        text = String::from_iter(idk);
        Value::Int(text.parse().unwrap())
    }

    fn parse_float(&mut self, token: &Token) -> Value {
        let mut text = self.input[token.span.start..token.span.end].to_string();
        let idk: Vec<_> = text.chars().filter(|c| c != &'_').collect();
        text = String::from_iter(idk);
        Value::Float(text.parse().unwrap())
    }
    /// peeks the current token
    fn peek(&mut self) -> Result<Token> {
        self.tokens.peek()
    }
    /// peeks the current token, and, if theres any token that is not [`TokenType::Eof`] it will return [`Some`] else [`None`]
    fn peek_opt(&mut self) -> Result<Option<Token>> {
        let ok = self.tokens.peek()?;
        if ok.is(&TokenType::Eof) {
            return Ok(None);
        }
        Ok(Some(ok))
    }
    /// peeks the current token and if none was found it prints and returns an error
    /// this is used for expressions that require the existence of a current token
    fn peek_some(&mut self) -> Result<Token> {
        let peeked = self.peek()?;
        if peeked.is(&TokenType::Eof) {
            return err(ParseError::UnexpectedStreamEnd.to_spanned(peeked.span));
        }
        Ok(peeked)
    }
    /// advances to the next token
    fn next_significant(&mut self) -> Result<Token> {
        let mut tok = self.tokens.next()?;
        while tok.is(&TokenType::Comment) {
            tok = self.tokens.next()?;
        }
        return Ok(tok);
    }
    fn skip_comment(&mut self) -> Result<Option<Token>> {
        if self.peek()?.is(&TokenType::Comment) {
            return Ok(Some(self.next()?));
        }
        return Ok(None);
    }
    fn next(&mut self) -> Result<Token> {
        self.tokens.next()
    }
    fn expect_next(&mut self) -> Result<Token> {
        let token = self.peek_some()?;
        self.next()?;
        Ok(token)
    }

    /// checks if a token is the expected token and if it isnt returns an error
    /// this is used for checking if certain expressions are valid
    fn check_valid(&mut self, expected: TokenType, token: Token) -> Result<()> {
        if token.is(&expected) {
            return Ok(());
        }
        err(ParseError::InvalidToken(expected, token.kind).to_spanned(token.span))
    }
    /// peeks the current token and checks if it is the same as the expected token returning an error if it isnt
    /// this is also used for validating expressions
    fn expect(&mut self, expected: TokenType) -> Result<Token> {
        let token = self.peek_some()?;
        self.check_valid(expected, token.clone())?;
        Ok(token)
    }
    fn is_expected(&mut self, expected: TokenType) -> Result<Option<Token>> {
        let token = self.peek()?;
        if token.is(&expected) {
            return Ok(Some(token));
        }
        Ok(None)
    }
    fn consume(&mut self, expected: TokenType) -> Result<Token> {
        let token = self.expect(expected)?;
        self.next()?;
        Ok(token)
    }

    fn consume_ident(&mut self) -> Result<String> {
        let token = self.expect(TokenType::Identifier)?;
        self.next()?;
        Ok(self.text(&token))
    }
    pub(crate) fn parse_text(&mut self) -> Result {
        let mut buffer: Vec<String> = Vec::new();
        let start_span = self.peek()?.span;
        let mut end_span = start_span.clone();
        while let Some(tok) = self.peek_opt()? {
            if !tok.exists() {
                break;
            }
            if tok.is(&TokenType::Comment) {
                continue;
            }
            let text = self.text(&tok);
            buffer.push(text);
            let advanced = self.next()?;
            end_span = advanced.span;
            if self
                .peek()?
                .is_any(&[TokenType::Lesser, TokenType::Greater])
            {
                break;
            }
        }
        let text = buffer.join(" ");
        return Ok(Node::Text(text).to_spanned(start_span + end_span));
    }
    /// Parses element properties like `a = 1`
    fn parse_props(&mut self) -> Result<HashMap<String, Spanned<Value>>> {
        let mut props: HashMap<String, Spanned<Value>> = HashMap::new();

        while let Some(tok) = self.peek_opt()? {
            if tok.is(&TokenType::Lesser) {
                self.next()?;
                break;
            }
            let prop_name = self.consume_ident()?;

            self.skip_comment()?;
            let sign = self.peek()?;
            if sign.isnt(&TokenType::Equal) {
                props.insert(prop_name, Value::Bool(true).to_spanned(sign.span));
                continue;
            }
            self.next_significant()?;

            let value_token = self.peek()?;
            let value = self.parse_value(&value_token)?;
            props.insert(prop_name, value);
        }
        return Ok(props);
    }
    fn parse_content(&mut self) -> Result<Vec<Spanned<Node>>> {
        let mut children: Vec<Spanned<Node>> = vec![];
        while let Some(tok) = self.peek_opt()? {
            if tok.is(&TokenType::End) || !tok.exists() {
                break;
            }
            let parsed = self.parse_expr()?;
            children.push(parsed);
        }
        Ok(children)
    }
    fn parse_element(&mut self) -> Result {
        let start = self.next()?;

        let tag_name = self.consume_ident()?;
        let props = self.parse_props()?;
        let children = self.parse_content()?;
        let end = self.next()?;
        let element = Element {
            name: tag_name,
            props,
            children,
        };

        return Ok(Node::Element(element).to_spanned(start.span + end.span));
    }
    fn parse_value(&mut self, token: &Token) -> Result<Spanned<Value>> {
        let value = match &token.kind {
            TokenType::False => Ok(Value::Bool(false)),
            TokenType::True => Ok(Value::Bool(true)),
            TokenType::Str(txt) => Ok(Value::String(txt.to_string())),
            _ => todo!(),
        }?;
        Ok(value.to_spanned(token.span))
    }
    fn parse_expr(&mut self) -> Result {
        let peeked = self.peek()?;
        match peeked.kind {
            TokenType::Lesser => self.parse_element(),

            TokenType::Comment => {
                let text = self.text(&peeked);
                self.next()?;
                return Ok(Node::Comment(text).to_spanned(peeked.span));
            }
            _ => self.parse_text(),
        }
    }
}
impl<'input> Parser<'input> {
    pub fn parse(input: &'input str, file_id: FileID) -> Result<Vec<Spanned<Node>>> {
        let mut parser = Parser {
            file_id,
            input,
            tokens: Lexer::new(input, file_id),
        };
        parser.parse_content()
    }
}
