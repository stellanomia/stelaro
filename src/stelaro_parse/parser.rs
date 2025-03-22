use crate::stelaro_ast::token::{Lit, LiteralKind, Token, TokenKind, TokenStream};
use crate::stelaro_common::span::Span;
use crate::stelaro_common::symbol::Ident;
use crate::stelaro_diagnostic::diag::{DiagCtxtHandle, ErrorEmitted};
use crate::stelaro_session::Session;

use super::diagnostics::DiagsParser;
use super::PResult;


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

    pub fn look_ahead(&self, k: usize) -> Option<Token> {
        self.token_stream.peek_nth(k).cloned()
    }

    pub fn parse_ident(&mut self) -> PResult<Ident> {

        if let TokenKind::Ident(symbol) = self.token.kind {
            self.bump();

            Ok(Ident::new(symbol, self.prev_token.span))
        } else if let TokenKind::Literal(lit @ Lit {
            kind: LiteralKind::Integer | LiteralKind::Float, ..
        }) = self.token.kind {
                let next = self.look_ahead(1);
                let span = if let Some(TokenKind::Ident(_)) = next.map(|t| t.kind) {
                    self.token.span.merge(&next.unwrap().span)
                } else {
                    self.token.span
                };

                Err(
                    DiagsParser::unexpected_numeric_literal_for_identifier(
                        self.dcx(),
                        lit,
                        span,
                    ).emit()
                )?
        } else {
            Err(
                DiagsParser::unexpected_token_for_identifier(
                    self.dcx(),
                    self.token.span,
                ).emit()
            )?
        }
    }
}