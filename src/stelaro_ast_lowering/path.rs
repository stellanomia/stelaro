use crate::stelaro_ast::{ast, NodeId};
use crate::stelaro_ast_lowering::LoweringContext;
use crate::stelaro_sir::{sir, Res};


impl<'sir> LoweringContext<'_, 'sir> {
    pub fn lower_path(
        &mut self,
        id: NodeId,
        path: &ast::Path,
    ) -> sir::Path<'sir> {
        let res = self.get_res(id).unwrap_or(Res::Err);
        let res = self.lower_res(res);
        sir::Path {
            span: path.span,
            res,
            segments: self.arena.alloc_from_iter(
                path.segments.iter().map(|segment| {
                    self.lower_path_segment(segment)
                })
            ),
        }
    }

    pub fn lower_path_segment(
        &mut self,
        segment: &ast::PathSegment,
    ) -> sir::PathSegment {
        let res = self.expect_res(segment.id);
        let sir_id = self.lower_node_id(segment.id);

        sir::PathSegment {
            ident: segment.ident,
            sir_id,
            res: self.lower_res(res),
        }
    }
}
