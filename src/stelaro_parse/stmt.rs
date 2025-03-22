use crate::stelaro_ast::ast::{Local, LocalKind};
use crate::stelaro_ast::{ast::{NodeId, Stmt, StmtKind}, token::TokenKind};

use super::diagnostics::DiagsParser;
use super::{parser::Parser, PResult};


impl Parser<'_> {
    pub fn parse_stmt(&mut self) -> PResult<Stmt> {
        match self.token.kind {
            TokenKind::Let => {
                self.parse_let()
            },
            TokenKind::While => {
                self.parse_while()
            },
            _ => {
                let expr = self.parse_expr()?;

                match self.token.kind {
                    TokenKind::Semicolon => {
                        self.bump();

                        Ok(
                            self.mk_stmt(
                                expr.span.merge(&self.prev_token.span),
                                StmtKind::Expr(Box::new(expr))
                            )
                        )
                    },
                    // 次が } でブロックの終端にある式である
                    TokenKind::RBrace => {
                        Ok(
                            self.mk_stmt(
                                expr.span,
                                StmtKind::ReturnableExpr(Box::new(expr))
                            )
                        )
                    },
                    _ => {
                        Err(
                            DiagsParser::missing_semicolon(
                                self.dcx(),
                                self.token.span
                            ).emit()
                        )
                    }
                }
            }
        }
    }

    pub fn parse_let(&mut self) -> PResult<Stmt> {
        self.eat(TokenKind::Let, self.token.span)?;
        let start = self.prev_token.span;

        let ident = self.parse_ident()?;

        let ty = if self.token.kind == TokenKind::Colon {
            self.bump();
            Some(self.parse_ty()?)
        } else {
            None
        };

        let kind = if self.token.kind == TokenKind::Semicolon {
            self.bump();
            LocalKind::Decl
        } else {
            self.eat(TokenKind::Equal, self.token.span)?;
            let expr = self.parse_expr()?;
            self.eat(TokenKind::Semicolon, self.token.span)?;

            LocalKind::Init(Box::new(expr))
        };

        Ok(
            self.mk_stmt(
                start.merge(&self.prev_token.span),
                StmtKind::Let(
                    Box::new(
                        Local {
                            id: NodeId::dummy(),
                            ident,
                            kind,
                            ty,
                            span: start.merge(&self.prev_token.span)
                        }
                    )
                )
            )
        )
    }

    pub fn parse_while(&mut self) -> PResult<Stmt> {
        self.eat(TokenKind::While, self.token.span)?;
        let start = self.prev_token.span;

        let cond = self.parse_expr()?;

        let block = self.parse_block()?;

        Ok(
            self.mk_stmt(
                start.merge(&self.prev_token.span),
                StmtKind::While (
                    Box::new(cond),
                    Box::new(block)
                )
            )
        )
    }
}