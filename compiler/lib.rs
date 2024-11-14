use steralo_lexer::Lexer;
use steralo_parse::parser::Parser;

pub mod steralo_lexer;
pub mod steralo_parse;
pub mod stelalo_ast;
pub mod common;

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
