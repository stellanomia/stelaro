use stelaro_lexer::Lexer;
use stelaro_parse::parser::Parser;

pub mod stelaro_lexer;
pub mod stelaro_parse;
pub mod stelaro_ast;
pub mod stelaro_common;

pub fn eval(src: &str) {
    let mut lexer = Lexer::new(src);
    let token_stream = match lexer.lex() {
        Ok(ts) => {
            ts
        },
        Err(errors) => {
            dbg!(errors);
            todo!()
        }
    };

    let parser = Parser::new(token_stream);
    

}
