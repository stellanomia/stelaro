use crate::stelaro_ast::{ast::{Block, Expr, ExprKind, NodeId, Stelo, Stmt, StmtKind}, token::{Lit, LiteralKind, Token, TokenKind, TokenStream}};
use crate::stelaro_common::{span::Span, symbol::Ident};
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
            token_stream: token_stream.filter(|t| t.kind != TokenKind::LineComment).collect(),
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

        dbg!(self.token);

        match self.token_stream.next() {
            Some(t) => self.token = t,
            None => panic!("bug: TokenStreamの範囲外アクセス")
        }
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

    pub fn parse_stelo(&mut self) -> PResult<Stelo> {
        todo!()
    }

    pub fn parse_ident(&mut self) -> PResult<Ident> {

        if let TokenKind::Ident(symbol) = self.token.kind {
            self.bump();

            Ok(Ident::new(symbol, self.prev_token.span))
        } else if let TokenKind::Literal(lit @ Lit {
            kind: LiteralKind::Integer | LiteralKind::Float, ..
        }) = self.token.kind {
                let next = self.look_ahead(1);

                // 次のトークンが識別子ならSpanに含める
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

    pub fn parse_block(&mut self) -> PResult<Block> {
        self.eat(TokenKind::LBrace, self.token.span)?;
        let start = self.prev_token.span;
        let mut stmts = vec![];

        loop {
            match self.token.kind {
                TokenKind::RBrace => {
                    self.bump();
                    break;
                },
                TokenKind::Eof => {
                    break;
                }
                _ => {
                    stmts.push(self.parse_stmt()?);
                }
            }
        }

        Ok(
            Block {
                id: NodeId::dummy(),
                stmts,
                span: start.merge(&self.prev_token.span)
            }
        )
    }

    #[inline]
    pub fn mk_expr(&self, span: Span, kind: ExprKind) -> Expr {
        Expr { kind, span, id: NodeId::dummy() }
    }

    #[inline]
    pub fn mk_stmt(&self, span: Span, kind: StmtKind) -> Stmt {
        Stmt { id: NodeId::dummy(), kind, span }
    }
}