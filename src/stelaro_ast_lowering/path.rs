use crate::stelaro_ast::{ast, NodeId};
use crate::stelaro_ast_lowering::LoweringContext;
use crate::stelaro_common::Span;
use crate::stelaro_sir::sir;


impl<'sir> LoweringContext<'_, 'sir> {
    pub fn lower_path(
        &mut self,
        id: NodeId,
        path: &ast::Path,
    ) -> sir::Path<'sir> {
        todo!()
    }

    pub fn lower_path_segment(
        &mut self,
        path_span: Span,
        segment: &ast::PathSegment,
    ) -> sir::PathSegment {
        todo!()
    }
}