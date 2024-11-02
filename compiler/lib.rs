use common::symbol::Interner;
use steralo_lexer::Lexer;

pub mod steralo_lexer;
pub mod steralo_parse;
pub mod common;

pub fn eval(src: &str) {
    let literals = Interner::new();
    let indents = Interner::new();
    let mut lexer = Lexer::new(src, &literals, &indents);
    match lexer.lex() {
        Ok(ts) => {
            dbg!(ts);
        },
        Err(errors) => {
            dbg!(errors);
        }
    }

}
