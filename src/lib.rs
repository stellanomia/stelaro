#![feature(associated_type_defaults)]
#![feature(never_type)]

pub mod stelaro_ast;
pub mod stelaro_common;
pub mod stelaro_interface;
pub mod stelaro_lexer;
pub mod stelaro_parse;
pub mod stelaro_session;
pub mod stelaro_diagnostic;

use std::rc::Rc;

use stelaro_common::source_map::SourceMap;
use stelaro_diagnostic::DiagCtxt;
use stelaro_lexer::Lexer;
use stelaro_parse::parser::Parser;
use stelaro_session::Session;

pub fn temp(src: String) {
    let src = Rc::new(src.to_string());
    let dcx = DiagCtxt::new(Rc::clone(&src));
    let source_map = Rc::new(SourceMap::new());
    let sess = Session::new(dcx, source_map);
    let mut lexer = Lexer::new(&sess, &src);
    if let Ok(ts) = lexer.lex() {
        let mut parser = Parser::new(&sess, ts);
        let expr = parser.parse_expr();

        match expr {
            Ok(expr) => {dbg!(&expr); expr},
            Err(_) => todo!(),
        };
    }

}


pub fn run() {
}