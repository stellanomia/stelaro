use crate::stelaro_ast::{ast::*, token::TokenKind};

use super::{parser::Parser, PResult};


impl Parser<'_> {
    pub fn parse_stmt(&mut self) -> PResult<Option<Stmt>> {
        match self.token.kind {
            TokenKind::Let => {
                self.parse_stmt_let().map(Some)
            },
            TokenKind::Loop => {
                self.parse_stmt_loop().map(Some)
            }
            TokenKind::While => {
                self.parse_stmt_while().map(Some)
            },
            TokenKind::Break => {
                self.parse_stmt_break().map(Some)
            },
            TokenKind::Continue => {
                self.eat(TokenKind::Continue, self.token.span)?;
                Ok(Some(self.mk_stmt(self.prev_token.span, StmtKind::Continue)))
            }
            TokenKind::Return => {
                self.parse_stmt_return().map(Some)
            },
            TokenKind::If => {
                let expr_if = self.parse_expr_if()?;

                Ok(Some(
                    self.mk_stmt(
                        expr_if.span,
                        StmtKind::Expr(Box::new(expr_if))
                    )
                ))
            },
            _ => {
                let expr = self.parse_expr()?;

                match self.token.kind {
                    TokenKind::Semicolon => {
                        self.bump();

                        Ok(Some(
                            self.mk_stmt(
                                expr.span.merge(&self.prev_token.span),
                                StmtKind::Semi(Box::new(expr))
                            )
                        ))
                    },
                    // 次が } でブロックの終端にある式である
                    TokenKind::RBrace => {
                        Ok(Some(
                            self.mk_stmt(
                                expr.span,
                                StmtKind::Expr(Box::new(expr))
                            )
                        ))
                    },
                    _ => {
                        Ok(None)
                    }
                }
            }
        }
    }

    pub fn parse_stmt_let(&mut self) -> PResult<Stmt> {
        self.eat(TokenKind::Let, self.token.span)?;
        let start = self.prev_token.span;

        let pat = self.parse_pat_before_ty()?;

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

        let id = self.next_node_id();

        Ok(
            self.mk_stmt(
                start.merge(&self.prev_token.span),
                StmtKind::Let(
                    Box::new(
                        Local {
                            id,
                            pat: Box::new(pat),
                            kind,
                            ty,
                            span: start.merge(&self.prev_token.span)
                        }
                    )
                )
            )
        )
    }

    pub fn parse_stmt_loop(&mut self) -> PResult<Stmt> {
        self.eat(TokenKind::Loop, self.token.span)?;
        let start = self.prev_token.span;
        let block = self.parse_block()?;

        Ok(
            self.mk_stmt(
                start.merge(&self.prev_token.span),
                StmtKind::Loop (
                    Box::new(block)
                )
            )
        )
    }

    pub fn parse_stmt_while(&mut self) -> PResult<Stmt> {
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

    pub fn parse_stmt_break(&mut self) -> PResult<Stmt> {
        self.eat(TokenKind::Break, self.token.span)?;
        let start = self.prev_token.span;

        let expr = if self.token.kind == TokenKind::Semicolon {
            None
        } else {
            Some(
                Box::new(self.parse_expr()?)
            )
        };

        self.eat(TokenKind::Semicolon, self.token.span)?;

        Ok(
            self.mk_stmt(
                start.merge(&self.prev_token.span),
                StmtKind::Break (
                    expr,
                )
            )
        )
    }

    pub fn parse_stmt_return(&mut self) -> PResult<Stmt> {
        self.eat(TokenKind::Return, self.token.span)?;
        let start = self.prev_token.span;

        let expr = if self.token.kind == TokenKind::Semicolon {
            None
        } else {
            Some(
                Box::new(self.parse_expr()?)
            )
        };

        self.eat(TokenKind::Semicolon, self.token.span)?;

        Ok(
            self.mk_stmt(
                start.merge(&self.prev_token.span),
                StmtKind::Return (
                    expr,
                )
            )
        )
    }
}
