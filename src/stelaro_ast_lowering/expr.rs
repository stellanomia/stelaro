use crate::stelaro_ast::ast;
use crate::stelaro_ast_lowering::LoweringContext;
use crate::stelaro_common::Span;
use crate::stelaro_sir::sir;



impl<'sir> LoweringContext<'_, 'sir> {

    pub fn lower_expr(&mut self, e: &ast::Expr) -> &'sir sir::Expr<'sir> {
        self.arena.alloc(self.lower_expr_mut(e))
    }


    pub fn lower_expr_mut(&mut self, e: &ast::Expr) -> sir::Expr<'sir> {
        todo!()
    }

    pub fn expr(&mut self, span: Span, kind: sir::ExprKind<'sir>) -> sir::Expr<'sir> {
        let sir_id = self.next_id();
        sir::Expr { sir_id, kind, span }
    }

    pub fn expr_block(&mut self, b: &'sir sir::Block<'sir>) -> sir::Expr<'sir> {
        self.expr(b.span, sir::ExprKind::Block(b))
    }
}