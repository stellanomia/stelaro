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
    let mut lexer = Lexer::new(src.as_str(), &sess);
    let token_stream = match lexer.lex() {
        Ok(ts) => {
            dbg!(&ts);
            ts
        },
        Err(errors) => {
            dbg!(errors);
            todo!()
        }
    };

    let _parser = Parser::new(token_stream);
}


pub fn run() {
    let _args = &std::env::args().collect::<Vec<String>>()[1..];

    
}