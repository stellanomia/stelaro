use crate::stelaro_ast::ast;
use crate::stelaro_ast_lowering::LoweringContext;
use crate::stelaro_common::{ensure_sufficient_stack, Span};
use crate::stelaro_sir::sir;



impl<'sir> LoweringContext<'_, 'sir> {
    pub fn lower_expr(&mut self, e: &ast::Expr) -> &'sir sir::Expr<'sir> {
        self.arena.alloc(self.lower_expr_mut(e))
    }

    fn lower_exprs(&mut self, exprs: &[ast::Expr]) -> &'sir [sir::Expr<'sir>] {
        self.arena.alloc_from_iter(exprs.iter().map(|x| self.lower_expr_mut(x)))
    }

    pub fn lower_expr_mut(&mut self, e: &ast::Expr) -> sir::Expr<'sir> {
        use ast::ExprKind;

        ensure_sufficient_stack(|| {
            // 丸括弧式は SirId を持たず、特別に処理されます。
            if let ExprKind::Paren(expr) = &e.kind {
                let mut expr = self.lower_expr_mut(expr);
                if e.span.contains(&expr.span) {
                    expr.span = e.span;
                }
                return expr;
            }

            let sir_id = self.lower_node_id(e.id);

            let kind = match &e.kind {
                ExprKind::Call(f, args) => {
                    let f = self.lower_expr(f);
                    sir::ExprKind::Call(f, self.lower_exprs(args))
                },
                ExprKind::Binary(binop, lhs, rhs) => {
                    let lhs = self.lower_expr(lhs);
                    let rhs = self.lower_expr(rhs);
                    sir::ExprKind::Binary(*binop, lhs, rhs)
                },
                ExprKind::Unary(un_op, expr) => {
                    let expr = self.lower_expr(expr);
                    sir::ExprKind::Unary(*un_op, expr)
                },
                ExprKind::Lit(token_lit) => todo!(),
                ExprKind::If(cond, then, else_opt) => todo!(),
                ExprKind::Block(block) => todo!(),
                ExprKind::Assign(lhs, rhs) => todo!(),
                ExprKind::Path(path) => todo!(),
                ExprKind::Paren(_) => unreachable!(),
            };

            sir::Expr { sir_id, kind, span: e.span }
        })
    }

    pub fn expr(&mut self, span: Span, kind: sir::ExprKind<'sir>) -> sir::Expr<'sir> {
        let sir_id = self.next_id();
        sir::Expr { sir_id, kind, span }
    }

    pub fn expr_block(&mut self, b: &'sir sir::Block<'sir>) -> sir::Expr<'sir> {
        self.expr(b.span, sir::ExprKind::Block(b))
    }
}