use crate::stelaro_ast::{ast::{Block, Stmt}, token::TokenKind};

use super::{parser::Parser, PResult};


impl Parser<'_> {
    pub fn parse_stmt(&mut self) -> PResult<Stmt> {
        match self.token.kind {
            TokenKind::Let => {
                self.parse_let()
            }
            TokenKind::If => {
                self.parse_if()
            }
            _ => {
                todo!()
            }
        }
    }

    pub fn parse_block(&mut self) -> PResult<Block> {
        todo!()
    }

    pub fn parse_let(&mut self) -> PResult<Stmt> {
        self.eat(TokenKind::Let, self.token.span)?;

        todo!()
    }

    pub fn parse_if(&mut self) -> PResult<Stmt> {
        self.eat(TokenKind::If, self.token.span)?;

        todo!()
    }
}