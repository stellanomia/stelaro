use std::{path::Path, rc::Rc};

use parser::Parser;

use crate::{stelaro_common::error_core::ErrorCore, stelaro_session::Session, stelaro_lexer::Lexer};

pub mod parser;
pub mod errors;

pub fn new_parser_from_file<'a>(sess: &'a Session, path: &Path) -> Result<Parser, Vec<Rc<ErrorCore>>>{
    let file = sess.source_map().load_file(path).unwrap_or_else(|e| {
        todo!()
    });

    let mut lexer = Lexer::new(file.src.as_ref());

    match lexer.lex() {
        Ok(ts) => Ok(Parser::new(ts)),
        Err(errs) => Err(errs.into_iter().map(|e|e.core()).collect()),
    }
}

