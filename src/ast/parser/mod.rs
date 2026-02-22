use std::{clone, collections::HashMap};

use crate::{
    ast::nodes::*,
    lang_errors::LangError,
    lexemes::{lexer::Lexer, tokens::*},
    spans::{FileID, IntoSpanned, Span, Spanned},
};

mod error;
use chumsky::{container::Container, prelude::todo};
use error::*;
use rayon::vec;
macro_rules! str_vec {
    ($($word:ident),*) => {
        vec![
            $(
                stringify!($word).to_owned(),
            )*
        ]
    };
}
#[derive(Clone, Debug)]
pub struct Parser<'input> {
    file_id: FileID,
    input: &'input str,
    tokens: Lexer<'input>,
    pub raw_tags: Vec<String>,
}
pub type Result<T = Spanned<Node>> = std::result::Result<T, Box<dyn LangError>>;
fn err<T>(value: impl LangError + 'static) -> Result<T> {
    Err(Box::new(value))
}
impl<'input> Parser<'input> {
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
    /// peeks the next token
    fn peek_next(&mut self) -> Result<Token> {
        self.tokens.peek_next()
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
        if !peeked.exists() {
            return err(ParseError::UnexpectedStreamEnd.to_spanned(peeked.span));
        }
        Ok(peeked)
    }
    /// advances to the next meaningful token
    ///
    fn next_significant(&mut self) -> Result<Token> {
        self.skip_unsignificant()?;
        return Ok(self.next()?);
    }
    fn skip_unsignificant(&mut self) -> Result<Vec<Token>> {
        let mut buffer: Vec<Token> = vec![];
        if self.peek()?.is_significant() {
            return Ok(buffer);
        }
        while let Some(token) = self.peek_opt()? {
            if token.is_significant() {
                return Ok(buffer);
            }
            buffer.push(self.next()?);
        }
        Ok(buffer)
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

    fn consume_word(&mut self) -> Result<String> {
        let token = self.expect(TokenType::Word)?;
        self.next()?;
        Ok(self.text(&token))
    }
    pub fn parse_raw_text(&mut self) -> Result {
        let mut buffer = String::new();
        let start_index = self.tokens.index;
        while let Some(ch) = self.tokens.peek_char() {
            if ch == '<' {
                break;
            }
            buffer.push(ch);
            self.tokens.advance();
        }
        let end_index = self.tokens.index;
        let span = Span::new(self.file_id, start_index, end_index);
        let node = Node::Text(buffer).to_spanned(span);

        Ok(node)
    }
    pub fn parse_text(&mut self) -> Result {
        let mut buffer = String::new();
        let start_span = self.peek()?.span;
        let mut end_span = start_span.clone();
        while let Some(token) = self.peek_opt()? {
            if !token.exists() {
                break;
            }
            if token.is(&TokenType::Comment) {
                continue;
            }
            let text = self.text(&token);
            buffer += &text;
            let advanced = self.next()?;
            end_span = advanced.span;
            if self
                .peek()?
                .is_any(&[TokenType::Lesser, TokenType::End, TokenType::LCloser])
            {
                break;
            }
        }
        buffer = buffer.trim().to_owned();
        return Ok(Node::Text(buffer).to_spanned(start_span + end_span));
    }
    /// Parses element properties like `a = 1`
    fn parse_props(&mut self) -> Result<HashMap<String, Spanned<Value>>> {
        let mut props: HashMap<String, Spanned<Value>> = HashMap::new();

        while let Some(_) = self.peek_opt()? {
            self.skip_unsignificant()?;
            let token = self.peek()?;
            if token.is(TokenType::RCloser) {
                break;
            }
            if token.is(&TokenType::Greater) {
                break;
            }

            let prop_name = self.consume_word()?;
            self.skip_unsignificant()?;
            let sign = self.peek()?;
            if sign.isnt(&TokenType::Equal) {
                props.insert(prop_name, Value::Bool(true).to_spanned(sign.span));
                continue;
            }
            self.next()?;
            self.skip_unsignificant()?;
            let value_token = self.peek()?;
            let value = self.parse_value(&value_token)?;
            self.next()?;
            props.insert(prop_name, value);
        }
        return Ok(props);
    }
    fn parse_content(&mut self, raw: bool) -> Result<Vec<Spanned<Node>>> {
        let mut children: Vec<Spanned<Node>> = vec![];
        while let Some(token) = self.peek_opt()? {
            if token.is_any([TokenType::End, TokenType::LCloser]) || !token.exists() {
                break;
            }

            let parsed = self.parse_expr(raw)?;
            if let Node::Text(ref text) = parsed.item {
                if text == "" {
                    continue;
                }
            }
            children.push(parsed);
        }
        Ok(children)
    }
    fn parse_element(&mut self) -> Result {
        let next = self.peek_next()?;
        if !next.exists() || next.is(&TokenType::Greater) {
            let token = self.next()?;
            let text = self.text(&token);
            let node = Node::Text(text).to_spanned(token.span);
            return Ok(node);
        }

        let start = self.next()?;
        let tag_name = self.consume_word()?;

        let parse_raw = self.raw_tags.contains(&tag_name);
        self.skip_unsignificant()?;
        if self.peek()?.is(TokenType::RCloser) {
            let end = self.next()?;
            let element = Element {
                name: tag_name,
                props: HashMap::new(),
                children: vec![],
                start_tag_span: start.span + end.span,
                end_tag_span: None,
            };
            return Ok(Node::Element(element).to_spanned(start.span + end.span));
        }
        let props = self.parse_props()?;
        if let Some(end) = self.peek()?.matches(TokenType::RCloser) {
            self.next()?;
            let element = Element {
                name: tag_name,
                props,
                children: vec![],
                start_tag_span: start.span + end.span,
                end_tag_span: None,
            };

            return Ok(Node::Element(element).to_spanned(start.span + end.span));
        }
        let start_tag_span = {
            let token = self.next()?;
            start.span + token.span
        };
        let children = self.parse_content(parse_raw)?;
        let end_start = self.next()?;
        if end_start.is(TokenType::LCloser) {
            self.skip_unsignificant()?;
            let end_tagname = self.consume_word()?;
            self.skip_unsignificant()?;
            let end = self.consume(TokenType::Greater)?;
            let end_tag_span = end_start.span + end.span;
            if end_tagname != tag_name {
                let error = ParseError::UnmatchedTag {
                    start_tag: tag_name.to_spanned(start_tag_span),
                    end_tag: end_tagname.to_spanned(end_tag_span),
                };
                return err(error.to_spanned(start.span + end.span));
            }
        }
        let element = Element {
            name: tag_name,
            props,
            children,
            start_tag_span,
            end_tag_span: Some(end_start.span),
        };

        return Ok(Node::Element(element).to_spanned(start.span + end_start.span));
    }
    fn parse_value(&mut self, token: &Token) -> Result<Spanned<Value>> {
        let value = match &token.kind {
            TokenType::False => Ok(Value::Bool(false)),
            TokenType::True => Ok(Value::Bool(true)),
            TokenType::Null => Ok(Value::Null),
            TokenType::Float => Ok(self.parse_float(token)),
            TokenType::Int => Ok(self.parse_int(token)),
            TokenType::Str(txt) => Ok(Value::String(txt.to_string())),
            foo => {
                dbg!(foo);
                todo!();
            }
        }?;
        Ok(value.to_spanned(token.span))
    }
    fn parse_expr(&mut self, raw: bool) -> Result {
        let peeked = self.peek()?;
        match peeked.kind {
            TokenType::Lesser => self.parse_element(),
            TokenType::Comment => {
                let text = self.text(&peeked);
                self.next()?;
                return Ok(Node::Comment(text).to_spanned(peeked.span));
            }

            _ if raw => self.parse_raw_text(),
            _ => self.parse_text(),
        }
    }
}
impl<'input> Parser<'input> {
    pub fn make(input: &'input str, file_id: FileID, raw_tags: Vec<String>) -> Self {
        Parser {
            file_id,
            input,
            tokens: Lexer::new(input, file_id),
            raw_tags,
        }
    }
    pub fn new(input: &'input str, file_id: FileID) -> Self {
        let raw_tags = str_vec!(pre, raw, script, style);
        Parser {
            file_id,
            input,
            raw_tags,
            tokens: Lexer::new(input, file_id),
        }
    }
    pub fn parse(input: &'input str, file_id: FileID) -> Result<Vec<Spanned<Node>>> {
        let mut parser = Self::new(input, file_id);
        parser.parse_content(false)
    }
}
