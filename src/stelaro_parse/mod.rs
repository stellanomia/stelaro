use std::path::Path;

use parser::Parser;

use crate::{stelaro_diagnostic::diag::ErrorEmitted, stelaro_lexer::Lexer, stelaro_session::Session};

pub mod parser;

pub fn new_parser_from_file<'a>(sess: &'a Session, path: &Path) -> Result<Parser, ErrorEmitted>{
    let file = sess.source_map().load_file(path).unwrap_or_else(|e| {

        todo!()
    });

    let mut lexer = Lexer::new(file.src.as_ref(), sess);

    match lexer.lex() {
        Ok(ts) => Ok(Parser::new(ts)),
        Err(errs) => Err(errs)?,
    }
}
