pub mod ast;
mod charvec;
mod filestore;
pub mod lang_errors;
pub mod lexemes;
pub mod spans;
use std::path::Path;

use filestore::FileStore;
use lexemes::lexer::Lexer;
use lexemes::tokens::*;

use crate::ast::nodes::Node;
use crate::ast::parser::Parser;
use crate::ast::schema::{self, ElementSchema, SchemaLoadError};
use crate::lang_errors::{LangMessage, LangResult};
use crate::spans::Spanned;

pub struct Compiler {
    pub file_store: FileStore,
    silent: bool,
}
impl Compiler {
    pub fn take_filestore(self) -> FileStore {
        self.file_store
    }
    pub fn get_filestore(&self) -> &FileStore {
        &self.file_store
    }
    pub fn make(file_store: FileStore, silent: bool) -> Self {
        Self { file_store, silent }
    }
    pub fn new() -> Self {
        Self {
            silent: false,

            file_store: FileStore::new(),
        }
    }
    pub fn lex(&mut self, input: &str) -> LangResult<Vec<Token>> {
        let file_id = self.file_store.add(input.to_owned());
        let mut lexer = Lexer::new(input, file_id);
        let mut buf = vec![];
        loop {
            let tok = lexer.next().inspect_err(|err| {
                if !self.silent {
                    err.msg()
                        .eprint(self.file_store.clone())
                        .expect("Could not print error.");
                }
            })?;
            if tok.is(&TokenType::Eof) {
                break;
            }
            buf.push(tok);
        }
        Ok(buf)
    }
    pub fn parse(
        &mut self,
        input: &str,
        schema_path: impl AsRef<Path>,
    ) -> LangResult<Vec<Spanned<Node>>> {
        let file_id = self.file_store.add(input.to_owned());

        Parser::new(input, file_id)
            .with_schema_file(schema_path)
            .inspect_err(|err| {
                if !self.silent {
                    err.msg()
                        .eprint(self.file_store.clone())
                        .expect("Could not print error.");
                }
            })
    }
    pub fn print_langerr(&self, err: &dyn LangMessage) -> std::io::Result<()> {
        err.msg().eprint(self.file_store.clone())
    }
}
