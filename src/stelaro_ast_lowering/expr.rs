use crate::stelaro_ast::{ast, token};
use crate::stelaro_ast_lowering::LoweringContext;
use crate::stelaro_common::{ensure_sufficient_stack, Span, Spanned};
use crate::stelaro_sir::sir::{self, LitKind};



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
                ExprKind::Lit(token_lit) => sir::ExprKind::Lit(self.lower_lit(token_lit, e.span)),
                ExprKind::If(cond, then, else_opt) => self.lower_expr_if(cond, then, else_opt.as_deref()),
                ExprKind::Block(block) => {
                    let block_sir_id = self.lower_node_id(block.id);
                    let sir_block = self.arena.alloc(
                        self.lower_block_noalloc(block_sir_id, block)
                    );
                    sir::ExprKind::Block(sir_block)
                },
                ExprKind::Assign(lhs, rhs, span) => {
                    let lhs = self.lower_expr(lhs);
                    let rhs = self.lower_expr(rhs);
                    sir::ExprKind::Assign(lhs, rhs, *span)
                },
                ExprKind::Path(path) => {
                    sir::ExprKind::Path(self.lower_path(e.id, path))
                },
                ExprKind::Paren(_) => unreachable!(),
            };

            sir::Expr { sir_id, kind, span: e.span }
        })
    }

    pub fn lower_lit(
        &mut self,
        token_lit: &token::Lit,
        span: Span,
    ) -> &'sir Spanned<LitKind> {
        let lit_kind = match LitKind::from_token_lit(*token_lit) {
            Ok(lit_kind) => lit_kind,
            Err(err) => {
                // let guar = report_lit_error(&self.tcx.sess.psess, err, *token_lit, span);
                // LitKind::Err(guar)
                todo!()
            }
        };
        self.arena.alloc(Spanned { node: lit_kind, span })
    }

    fn lower_expr_if(
        &mut self,
        cond: &ast::Expr,
        then: &ast::Block,
        else_opt: Option<&ast::Expr>,
    ) -> sir::ExprKind<'sir> {
        let lowered_cond = self.lower_expr(cond);
        let then_expr = self.lower_block_expr(then);
        if let Some(rslt) = else_opt {
            sir::ExprKind::If(
                lowered_cond,
                self.arena.alloc(then_expr),
                Some(self.lower_expr(rslt)),
            )
        } else {
            sir::ExprKind::If(lowered_cond, self.arena.alloc(then_expr), None)
        }
    }

    pub fn expr(&mut self, span: Span, kind: sir::ExprKind<'sir>) -> sir::Expr<'sir> {
        let sir_id = self.next_id();
        sir::Expr { sir_id, kind, span }
    }

    pub fn expr_block(&mut self, b: &'sir sir::Block<'sir>) -> sir::Expr<'sir> {
        self.expr(b.span, sir::ExprKind::Block(b))
    }
}