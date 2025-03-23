use crate::stelaro_ast::{token::TokenKind, ty::{Ty, TyKind}};

use super::{parser::Parser, PResult};


impl Parser<'_> {
    pub fn parse_ty(&mut self) -> PResult<Ty> {
        let start = self.token.span;

        let kind = match self.token.kind {
            TokenKind::Ident(symbol) => {
                let path = self.parse_path()?;

                if symbol.as_str() == "_" {
                    TyKind::Infer
                } else {
                    TyKind::Path(path)
                }
            },
            _ => todo!()
        };

        Ok(
            Ty {
                kind,
                span: start.merge(&self.prev_token.span),
            }
        )
    }
}