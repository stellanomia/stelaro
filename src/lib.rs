#![feature(associated_type_defaults)]
#![feature(never_type)]
#![feature(min_specialization)]
#![allow(clippy::should_implement_trait)]

pub mod stelaro_ast;
pub mod stelaro_common;
pub mod stelaro_context;
pub mod stelaro_diagnostic;
pub mod stelaro_interface;
pub mod stelaro_lexer;
pub mod stelaro_parse;
pub mod stelaro_resolve;
pub mod stelaro_session;
pub mod stelaro_sir;
pub mod stelaro_ty;

use std::rc::Rc;

use stelaro_common::source_map::SourceMap;
use stelaro_diagnostic::DiagCtxt;
use stelaro_lexer::Lexer;
use stelaro_parse::parser::Parser;
use stelaro_session::{session::default_emitter, Session};

pub fn temp(src: String) {
    let src = Rc::new(src.to_string());
    let source_map = Rc::new(SourceMap::new());
    let emitter = default_emitter(Rc::clone(&source_map));
    let dcx = DiagCtxt::new(emitter);
    let sess = Session::new(dcx, source_map);
    let mut lexer = Lexer::new(&sess, &src);
    let Ok(ts) = lexer.lex() else {
        unimplemented!()
    };
    let mut parser = Parser::new(&sess, ts);

    let Ok(stelo) = parser.parse_stelo() else {
        unimplemented!()
    };

    dbg!(stelo);
}


pub fn run() {
}