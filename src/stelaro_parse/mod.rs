pub mod parser;
mod diagnostics;
mod expr;
mod stmt;
mod ty;

use crate::stelaro_diagnostic::diag::ErrorEmitted;
use crate::stelaro_lexer::Lexer;
use crate::stelaro_session::Session;

use std::path::Path;
use parser::Parser;


type PResult<T> = Result<T, ErrorEmitted>;

pub fn new_parser_from_file<'sess>(sess: &'sess Session, path: &Path) -> Result<Parser<'sess>, ErrorEmitted> {
    let file = sess.source_map().load_file(path).unwrap_or_else(|e| {
        sess.dcx().emit_fatal(format!("{e}"));
    });

    let mut lexer = Lexer::new(sess, file.src.as_ref(),);

    match lexer.lex() {
        Ok(ts) => Ok(Parser::new(sess, ts)),
        Err(errs) => Err(errs)?,
    }
}

pub fn new_parser_from_src(sess: &Session, src: String ) -> Result<Parser<'_>, ErrorEmitted> {
    let mut lexer = Lexer::new(sess, src.as_str());

    match lexer.lex() {
        Ok(ts) => Ok(
            Parser::new(sess, ts)
        ),
        Err(errs) => Err(errs)?,
    }
}

