use std::path::Path;

use errors::ParseError;
use parser::Parser;

use crate::{stelaro_lexer::{errors::LexerError, Lexer}, stelaro_session::Session};

pub mod parser;
pub mod errors;

pub fn new_parser_from_file<'a>(sess: &'a Session, path: &Path) -> Result<Parser, ParserErrors>{
    let file = sess.source_map().load_file(path).unwrap_or_else(|e| {
        // ここで別のエラー型を返したい
        todo!()
    });

    let mut lexer = Lexer::new(file.src.as_ref());

    match lexer.lex() {
        Ok(ts) => Ok(Parser::new(ts)),
        Err(errs) => Err(errs)?,
    }
}

#[derive(Debug)]
pub enum ParserErrors {
    ParseErrors(Vec<ParseError>),
    LexerErrors(Vec<LexerError>),
}

impl From<Vec<LexerError>> for ParserErrors {
    fn from(value: Vec<LexerError>) -> Self {
        ParserErrors::LexerErrors(value)
    }
}

