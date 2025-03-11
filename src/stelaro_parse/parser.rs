use std::mem;

use crate::{stelaro_ast::token::{Token, TokenStream}, stelaro_session::Session};

pub struct Parser<'sess> {
    sess: &'sess Session,
    token_stream: TokenStream,
    token: Token,
    prev_token: Token,
}

impl<'sess> Parser<'sess> {
    pub fn new(
        sess: &'sess Session,
        token_stream: TokenStream,
    ) -> Self {
        let mut parser = Parser {
            sess,
            token_stream,
            token: Token::dummy(),
            prev_token: Token::dummy(),
        };

        parser.bump();

        parser
    }

    pub fn bump(&mut self) {
        self.prev_token = self.token;

        self.token = self.token_stream.next().expect("bug: TokenStreamの範囲外アクセス");
    }

}