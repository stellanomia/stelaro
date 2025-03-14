use crate::stelaro_ast::{ast::Stmt, token::TokenKind};

use super::{parser::Parser, PResult};


impl<'sess> Parser<'sess> {
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
        todo!()
    }
}