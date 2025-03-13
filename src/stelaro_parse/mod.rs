use std::path::Path;

use parser::Parser;

use crate::{stelaro_diagnostic::diag::ErrorEmitted, stelaro_lexer::Lexer, stelaro_session::Session};

pub mod parser;
mod diagnostics;
mod expr;

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

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{stelaro_common::source_map::SourceMap, stelaro_diagnostic::DiagCtxt};

    use super::*;

    #[test]
    fn test_parse_expr() {
        let src = Rc::new(
            "x = (1 + 2) * 3 == 4 and 5 == 6 or 7 != 8 or 9 == 10 and true".to_string()
        );
        let dcx = DiagCtxt::new(Rc::clone(&src));
        let source_map = Rc::new(SourceMap::new());
        let sess = &Session::new(dcx, source_map);
        let mut lexer = Lexer::new(sess, src.as_str());
        let ts = lexer.lex().unwrap();
        let mut parser = Parser::new(sess, ts);
        parser.parse_expr().unwrap();
    }
}