#![feature(associated_type_defaults)]
#![feature(never_type)]
#![feature(min_specialization)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::type_complexity)]

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

use stelaro_lexer::Lexer;
use stelaro_parse::parser::Parser;
use stelaro_session::{config::Input, session::{build_session, CompilerPaths}, Session};

pub fn temp(src: String) {
    let src = Rc::new(src.to_string());

    let paths = CompilerPaths {
        input: Input::Str {
            name: "temp".to_string(),
            input: "".to_string()
        },
        output_dir: None,
        output_file: None,
        temps_dir: None,
    };

    let sess = build_session(paths);
    let mut lexer = Lexer::new(&sess.psess, &src);
    let Ok(ts) = lexer.lex() else {
        unimplemented!()
    };
    let mut parser = Parser::new(&sess.psess, ts);

    let Ok(stelo) = parser.parse_stelo() else {
        unimplemented!()
    };

    dbg!(stelo);
}


pub fn run() {
}