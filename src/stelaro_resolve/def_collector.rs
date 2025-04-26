use std::ops::ControlFlow;
use visit::Visitor;

use super::Resolver;

use crate::stelaro_ast::ty;
use crate::stelaro_ast::visit;
use crate::stelaro_ast::ast::*;


impl<'ra, 'tcx> Resolver<'ra, 'tcx> {
    pub fn define() {
        
    }
}

struct DefCollector<'r, 'ra, 'tcx> {
    resolver: &'r mut Resolver<'ra, 'tcx>,
}

impl<'r, 'ra, 'tcx> Visitor<'r> for DefCollector<'r, 'ra, 'tcx> {
    fn visit_stelo(&mut self, stelo: &'r Stelo) -> ControlFlow<Self::BreakTy> {
        visit::walk_stelo(self, stelo)
    }

    fn visit_item(&mut self, item: &'r Item) -> ControlFlow<Self::BreakTy> {
        visit::walk_item(self, item)
    }

    fn visit_fn_decl(&mut self, f: &'r Function) -> ControlFlow<Self::BreakTy> {
        visit::walk_fn_decl(self, f)
    }

    fn visit_ident(&mut self, _ident: &'r crate::stelaro_common::Ident) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }

    fn visit_block(&mut self, b: &'r Block) -> ControlFlow<Self::BreakTy> {
        visit::walk_block(self, b)
    }

    fn visit_param(&mut self, param: &'r Param) -> ControlFlow<Self::BreakTy> {
        visit::walk_param(self, param)
    }

    fn visit_fn_ret_ty(&mut self, ret_ty: &'r FnRetTy) -> ControlFlow<Self::BreakTy> {
        visit::walk_fn_ret_ty(self, ret_ty)
    }

    fn visit_stmt(&mut self, stmt: &'r Stmt) -> ControlFlow<Self::BreakTy> {
        visit::walk_stmt(self, stmt)
    }

    fn visit_ty(&mut self, ty: &'r ty::Ty) -> ControlFlow<Self::BreakTy> {
        visit::walk_ty(self, ty)
    }

    fn visit_local(&mut self, local: &'r Local) -> ControlFlow<Self::BreakTy> {
        visit::walk_local(self, local)
    }

    fn visit_path(&mut self, path: &'r Path) -> ControlFlow<Self::BreakTy> {
        visit::walk_path(self, path)
    }

    fn visit_path_segment(&mut self, path_segment: &'r PathSegment) -> ControlFlow<Self::BreakTy> {
        visit::walk_path_segment(self, path_segment)
    }

    fn visit_pat(&mut self, pat: &'r Pat) -> ControlFlow<Self::BreakTy> {
        visit::walk_pat(self, pat)
    }

    fn visit_expr(&mut self, expr: &'r Expr) -> ControlFlow<Self::BreakTy> {
        visit::walk_expr(self, expr)
    }
}