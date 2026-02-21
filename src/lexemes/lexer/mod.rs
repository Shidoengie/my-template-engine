use super::tokens::*;
use crate::charvec::CharVec;
use crate::lang_errors::{LangError, LangResult};
use crate::lexemes::*;
use crate::spans::{FileID, IntoSpanned, Span, Spanned};
use std::fmt::Write;
use std::str::Chars;
mod error;

pub use error::*;
pub type Result<T = Token> = LangResult<T>;
#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    file_id: FileID,
    chars: Chars<'a>,
    source: String,
    pub(crate) index: usize,
}
impl<'a> Lexer<'a> {
    pub(crate) fn peek_char(&self) -> Option<char> {
        self.chars.clone().next()
    }
    fn peek_advance(&mut self) -> Option<char> {
        self.advance();
        self.peek_char()
    }
    fn make_error<T>(&self, err: LexError, start: usize, stop: usize) -> Result<T> {
        let inner = err.to_spanned(self.new_span(start, stop));
        Err(Box::new(inner) as Box<_>)
    }
    fn peek_next_char(&mut self) -> Option<char> {
        let mut cur_chars = self.chars.clone();
        cur_chars.next();
        cur_chars.next()
    }
    fn consume_char(&mut self, expected: char) -> Result<char> {
        let start = self.index;
        let Some(advanced) = self.advance() else {
            return self.make_error(LexError::UnexpectedStreamEnd, start, start);
        };
        if advanced != expected {
            return self.make_error(LexError::UnexpectedChar(advanced), start, start + 1);
        }
        return Ok(advanced);
    }
    pub(crate) fn advance(&mut self) -> Option<char> {
        self.index += 1;
        self.chars.next()
    }
    fn current_is(&mut self, expected: char) -> bool {
        self.peek_char() == Some(expected)
    }
    fn new_span(&self, start: usize, end: usize) -> Span {
        Span::new(self.file_id, start, end)
    }

    fn lex_number(&mut self) -> Result {
        let mut dot_count: u16 = 0;
        let start = self.index;

        let mut current = self.peek_char();
        if current.is_some_and(|x| x == '-') {
            current = self.advance();
        }
        while let Some(value) = current {
            if !value.is_numeric() && value != '.' && value != '_' {
                break;
            }

            if value == '.' {
                let Some(next) = self.peek_next_char() else {
                    return self.make_error(LexError::InvalidNumber, start, self.index);
                };
                if !next.is_ascii_digit() {
                    break;
                }
                dot_count += 1;
            }
            current = self.peek_advance();
        }
        if dot_count > 1 {
            return self.make_error(LexError::InvalidNumber, start, self.index);
        }
        let is_float = dot_count != 0;
        if is_float {
            return Ok(Token::new(
                TokenType::Float,
                self.new_span(start - 1, self.index),
            ));
        }
        Ok(Token::new(
            TokenType::Int,
            self.new_span(start - 1, self.index),
        ))
    }
    fn lex_identifier(&mut self) -> Result {
        let start = self.index - 1;
        let mut current = self.peek_char();
        while let Some(value) = current {
            if value.is_alphanumeric() || value == '_' {
                current = self.peek_advance();
                continue;
            }

            break;
        }
        let stop = self.index;
        let Some(span) = self.source.get(start..stop) else {
            return self.make_error(LexError::InvalidIdent, start, stop);
        };
        let kind = tokens::map_keyword(span).unwrap_or(TokenType::Word);
        Ok(Token::new(kind, self.new_span(start, stop)))
    }
    fn lex_string(&mut self, quote: char) -> Result {
        let start = self.index;
        let mut last = self.advance();
        let mut escaped = false;
        let mut buffer: Vec<char> = vec![];
        loop {
            let Some(unwrapped) = last else {
                return self.make_error(LexError::UnterminatedStr(quote), start, start + 1);
            };
            match (escaped, unwrapped) {
                (false, '\\') => escaped = true,
                (false, q) => {
                    if q == quote {
                        break;
                    }
                    buffer.push(q);
                    let bytecount = q.len_utf8();
                    self.index += bytecount.saturating_sub(1);
                }
                (true, ch) => {
                    let escape_map = match ch {
                        'n' => '\n',
                        't' => '\t',
                        '\\' => '\\',
                        '0' => '\0',
                        '"' => '\"',
                        '\'' => '\'',

                        _ => return self.make_error(LexError::InvalidEscape, start, self.index),
                    };
                    buffer.push(escape_map);
                    escaped = false;
                }
            }

            last = self.advance();
        }

        Ok(TokenType::Str(CharVec(buffer)).to_token(self.new_span(start - 1, self.index)))
    }
    fn make_eof_token(&self) -> Result {
        Ok(Token::new(
            TokenType::Eof,
            self.new_span(self.index - 1, self.index),
        ))
    }
    fn push_advance(&mut self, kind: TokenType, range: Span) -> Token {
        self.advance();
        Token::new(kind, range)
    }
    fn multi_char_token(
        &mut self,
        expected: char,
        short_token: TokenType,
        long_token: TokenType,
        range_start: usize,
    ) -> Result {
        if self.current_is(expected) {
            return Ok(self.push_advance(long_token, self.new_span(range_start, self.index)));
        }
        Ok(Token::new(
            short_token,
            self.new_span(range_start, range_start + 1),
        ))
    }

    fn ident_or_num(&mut self, expected: char) -> Result {
        let start = self.index;
        if expected.is_ascii_digit() {
            return self.lex_number();
        }
        if expected.is_alphanumeric() || expected == '_' {
            return self.lex_identifier();
        }
        self.make_error(LexError::UnexpectedChar(expected), start - 1, start)
    }

    fn multi_comment(&mut self) -> Result {
        self.consume_char('*')?;
        let mut nest = 1;
        let start = self.index;
        let mut end = self.index;
        while nest >= 1 {
            let Some(peeked) = self.peek_char() else {
                break;
            };
            match peeked {
                '*' => {
                    let Some(peeked) = self.peek_next_char() else {
                        self.advance();
                        continue;
                    };
                    if peeked != '>' {
                        continue;
                    }

                    nest -= 1;

                    let _ = self.advance();
                    let _ = self.advance();
                }
                '<' => {
                    let Some(peeked) = self.peek_next_char() else {
                        self.advance();
                        continue;
                    };
                    if peeked != '*' {
                        continue;
                    }
                    nest += 1;
                    let _ = self.advance();
                    let _ = self.advance();
                }
                _ => {
                    end = self.index + 1;
                    self.advance();
                }
            }
        }
        return Ok(Token::new(TokenType::Comment, self.new_span(start, end)));
    }

    pub fn new(src: &'a str, file_id: FileID) -> Self {
        Self {
            file_id,
            chars: src.chars(),
            source: String::from(src),
            index: 0,
        }
    }

    fn token_from_char(&mut self, ch: char, start: usize) -> Result {
        use TokenType as T;

        let range = self.new_span(start, start + 1);
        let just = |tk: TokenType| -> Result { Ok(Token::new(tk, range)) };
        match ch {
            '.' => just(T::Dot),
            ',' => just(T::Comma),
            '{' => just(T::LBrace),
            '}' => just(T::RBrace),
            '(' => just(T::LParen),
            ')' => just(T::RParen),
            '[' => just(T::LBracket),
            ']' => just(T::RBracket),
            '%' => just(T::Percent),
            ':' => just(T::Colon),
            '$' => just(T::Dollar),
            '@' => just(T::At),
            '|' => just(T::Pipe),
            '&' => just(T::Ampersand),
            '"' => self.lex_string('"'),
            '\'' => self.lex_string('\''),
            '?' => just(T::Question),
            '!' => just(T::Bang),

            '\r' => self.multi_char_token('\n', T::Space, T::NewLine, start),
            '\n' => just(T::NewLine),
            ' ' | '\t' => just(T::Space),
            '*' => just(T::Star),
            '-' => {
                let Some(peeked) = self.peek_char() else {
                    return Ok(Token::new(TokenType::Minus, range));
                };
                if peeked.is_alphanumeric() {
                    return self.lex_number();
                }
                return Ok(Token::new(TokenType::Minus, range));
            }
            '>' => just(T::Greater),
            '/' => just(T::Slash),
            '=' => just(T::Equal),
            '<' => self.lex_lesser_token(range),
            last => self.ident_or_num(last),
        }
    }
    fn lex_end_token(&mut self, range: Span) -> Result {
        let Some(peeked) = self.peek_next_char() else {
            return Ok(Token::new(TokenType::Lesser, range));
        };

        if peeked != '>' {
            return Ok(Token::new(TokenType::Lesser, range));
        }
        self.advance();
        self.advance();
        let mut span = range;
        span.end = self.index;
        return Ok(Token::new(TokenType::End, span));
    }
    fn lex_lesser_token(&mut self, range: Span) -> Result {
        let Some(peeked) = self.peek_char() else {
            return Ok(Token::new(TokenType::Lesser, range));
        };
        match peeked {
            '*' => self.multi_comment(),
            '/' => self.lex_end_token(range),
            _ => Ok(Token::new(TokenType::Lesser, range)),
        }
    }
    pub fn peek_next(&mut self) -> Result {
        let old_chars = self.chars.clone();
        let old_index = self.index;
        self.next()?;
        let token = self.next()?;
        self.chars = old_chars;
        self.index = old_index;
        Ok(token)
    }
    pub fn peek(&mut self) -> Result {
        let old_chars = self.chars.clone();
        let old_index = self.index;
        let token = self.next()?;
        self.chars = old_chars;
        self.index = old_index;
        Ok(token)
    }
    pub fn next(&mut self) -> Result {
        let start = self.index;
        let Some(last) = self.advance() else {
            return self.make_eof_token();
        };

        self.token_from_char(last, start)
    }
}
