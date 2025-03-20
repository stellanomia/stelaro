use crate::stelaro_ast::{ast::{Block, Stmt}, token::TokenKind};

use super::{parser::Parser, PResult};


impl Parser<'_> {
    pub fn parse_stmt(&mut self) -> PResult<Stmt> {
        match self.token.kind {
            TokenKind::Let => {
                self.bump();
                todo!()
            }
            _ => {
                todo!()
            }
        }
    }

    pub fn parse_block(&mut self) -> PResult<Block> {
        todo!()
    }
}