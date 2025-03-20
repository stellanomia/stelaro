use crate::stelaro_ast::token::{Token, TokenKind, TokenStream};
use crate::stelaro_common::span::Span;
use crate::stelaro_diagnostic::diag::{DiagCtxtHandle, ErrorEmitted};
use crate::stelaro_session::Session;

use super::diagnostics::DiagsParser;


pub struct Parser<'sess> {
    sess: &'sess Session,
    pub token_stream: TokenStream,
    pub token: Token,
    pub prev_token: Token,
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

    #[inline]
    pub fn dcx(&self) -> DiagCtxtHandle<'_> {
        self.sess.dcx()
    }

    pub fn bump(&mut self) {
        self.prev_token = self.token;

        self.token = self.token_stream.next().expect("bug: TokenStreamの範囲外アクセス");
    }

    pub fn eat(&mut self, expected: TokenKind, span: Span) -> Result<(), ErrorEmitted>{
        if self.token.kind == expected {
            self.bump();
            Ok(())
        }else {
            Err(
                DiagsParser::unexpected_token_with_expected(
                    self.dcx(),
                    self.token.kind,
                    expected,
                    span,
                ).emit()
            )
        }
    }

}