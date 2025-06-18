use crate::stelaro_ast::{ast::{Path, PathSegment}, token::TokenKind};

use super::{parser::Parser, PResult};


impl<'sess> Parser<'sess> {
    pub fn parse_path(&mut self) -> PResult<Path> {
        let mut segments = vec![];
        let start_span = self.token.span;

        segments.push(
            PathSegment {
                ident: self.parse_ident()?,
                id: self.next_node_id(),
            }
        );

        while self.token.kind == TokenKind::PathSep {
            self.bump();

            segments.push(
                PathSegment {
                    ident: self.parse_ident()?,
                    id: self.next_node_id(),
                }
            );
        }


        Ok(
            Path {
                span: start_span.merge(&self.prev_token.span),
                segments
            }
        )
    }
}
