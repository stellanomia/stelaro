use crate::stelaro_ast::ast::{Pat, PatKind};

use super::{parser::Parser, PResult};



impl<'sess> Parser<'sess> {
    pub fn parse_pat_before_ty(&mut self) -> PResult<Pat> {
        let ident = self.parse_ident()?;

        let kind = if ident.is_underscore() {
            PatKind::WildCard
        } else {
            PatKind::Ident(ident)
        };

        Ok(
            Pat {
                id: self.next_node_id(),
                kind,
                span: ident.span
            }
        )
    }
}