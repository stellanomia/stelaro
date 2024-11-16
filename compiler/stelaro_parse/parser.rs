use crate::stelaro_ast::token::{Token, TokenStream};

use super::errors::ParseError;


pub struct Parser {
    token_stream: TokenStream,
    token: Token,
    prev_token: Token,
}

impl Parser {
    pub fn new(
        token_stream: TokenStream,
    ) -> Self {
        let mut parser = Parser {
            token_stream,
            token: Token::dummy(),
            prev_token: Token::dummy(),
        };

        parser.bump();

        parser
    }

    pub fn bump(&mut self) {

    }

    pub fn expect(&mut self, token: &Token) -> Result<(), ParseError> {
        todo!()
    }
}