pub mod session;
pub mod stelaro_lexer;
pub mod stelaro_parse;
pub mod stelaro_ast;
pub mod stelaro_common;

use stelaro_lexer::Lexer;
use stelaro_parse::parser::Parser;

pub fn 仮(src: &str) {
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

    let parser = Parser::new(token_stream);
    

}
