use crate::stelaro_ast::{token::TokenKind, ty::{Ty, TyKind}};

use super::{diagnostics::DiagsParser, parser::Parser, PResult};


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
            // TODO: タプル型の実装
            TokenKind::LParen => {
                self.bump();

                self.eat(TokenKind::RParen, self.token.span)?;

                TyKind::Unit
            },
            _ => {
                let mut diag = DiagsParser::unexpected_token_for_type(
                    self.dcx(),
                    self.token.span
                );

                diag.set_label(
                    self.prev_token.span.between(&self.token.span),
                    "ここに型を記述してください".to_string()
                );

                Err(diag.emit())?
            }
        };

        Ok(
            Ty {
                id: self.next_node_id(),
                kind,
                span: start.merge(&self.prev_token.span),
            }
        )
    }
}
