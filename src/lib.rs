#![feature(associated_type_defaults)]
#![feature(never_type)]

pub mod stelaro_ast;
pub mod stelaro_common;
pub mod stelaro_interface;
pub mod stelaro_lexer;
pub mod stelaro_parse;
pub mod stelaro_session;
pub mod stelaro_diagnostic;

use stelaro_lexer::Lexer;
use stelaro_parse::parser::Parser;

pub fn temp(src: &str) {
    let mut lexer = Lexer::new(src);
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