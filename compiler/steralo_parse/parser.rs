use crate::stelalo_ast::token::{Token, TokenStream};



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
}