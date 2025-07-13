use crate::stelaro_ast::ast::{self, StmtKind};
use crate::stelaro_ast_lowering::LoweringContext;
use crate::stelaro_common::Span;
use crate::stelaro_sir::{sir::{self, LoopSource}, sir_id::SirId};


impl<'a, 'sir> LoweringContext<'a, 'sir> {
    pub fn lower_block(
        &mut self,
        b: &ast::Block,
    ) -> &'sir sir::Block<'sir> {
        let sir_id = self.lower_node_id(b.id);
        self.arena.alloc(self.lower_block_noalloc(sir_id, b))
    }

    pub fn lower_block_noalloc(
        &mut self,
        sir_id: SirId,
        b: &ast::Block,
    ) -> sir::Block<'sir> {
        let (stmts, expr) = self.lower_stmts(&b.stmts);
        sir::Block { sir_id, stmts, expr, span: b.span }
    }

    fn lower_stmts(
        &mut self,
        mut ast_stmts: &[ast::Stmt],
    ) -> (&'sir [sir::Stmt<'sir>], Option<&'sir sir::Expr<'sir>>) {
        let mut stmts: Vec<sir::Stmt> = vec![];
        let mut expr = None;

        while let [s, tail @ ..] = ast_stmts {
            match s.kind {
                StmtKind::Let(ref local) => {
                    let sir_id = self.lower_node_id(s.id);
                    let local = self.lower_local(local);
                    let kind = sir::StmtKind::Let(local);
                    let span = s.span;
                    stmts.push(sir::Stmt { sir_id, kind, span });
                }
                StmtKind::Expr(ref e) => {
                    let e = self.lower_expr(e);
                    if tail.is_empty() {
                        expr = Some(e);
                    } else {
                        let sir_id = self.lower_node_id(s.id);
                        let kind = sir::StmtKind::Expr(e);
                        let span = s.span;
                        stmts.push(sir::Stmt { sir_id, kind, span });
                    }
                }
                StmtKind::Semi(ref e) => {
                    let sir_id = self.lower_node_id(s.id);
                    let e = self.lower_expr(e);
                    let kind = sir::StmtKind::Semi(e);
                    let span = s.span;
                    stmts.push(sir::Stmt { sir_id, kind, span });
                }
                StmtKind::Loop(ref b) => {
                    let sir_id = self.lower_node_id(s.id);

                    self.with_loop_scope(sir_id,|this| {
                        let span = s.span;
                        let kind = sir::StmtKind::Loop(
                            this.lower_block(b),
                            LoopSource::Loop,
                            span
                        );
                        stmts.push(sir::Stmt {sir_id, kind, span});
                    })
                }
                StmtKind::While(ref cond, ref b) => {
                    let sir_id = self.lower_node_id(s.id);

                    self.with_loop_scope(sir_id, |this| {
                        let span = s.span.merge(&cond.span);
                        let kind = this.lower_expr_while_in_loop_scope(span, cond, b);
                        stmts.push(
                            sir::Stmt { sir_id, kind, span }
                        );
                    })
                }
                StmtKind::Break(ref e) => {
                    let sir_id = self.lower_node_id(s.id);
                    let opt_expr = e.as_ref().map(|expr| self.lower_expr(expr));
                    let kind = sir::StmtKind::Break(
                        self.lower_loop_destination(),
                        opt_expr,
                    );
                    stmts.push(
                        sir::Stmt { sir_id, kind, span: s.span }
                    );
                }
                StmtKind::Continue => {
                    let sir_id = self.lower_node_id(s.id);
                    let kind = sir::StmtKind::Continue(self.lower_loop_destination());
                    stmts.push(sir::Stmt { sir_id, kind, span: s.span });
                }
                StmtKind::Return(ref e) => {
                    let sir_id = self.lower_node_id(s.id);
                    let e = e.as_ref().map(|e| self.lower_expr(e));
                    let kind = sir::StmtKind::Return(e);
                    let span = s.span;
                    stmts.push(sir::Stmt { sir_id, kind, span });
                }
            }
            ast_stmts = tail;
        }
        (self.arena.alloc_from_iter(stmts), expr)
    }

    fn lower_local(&mut self, l: &ast::Local) -> &'sir sir::LetStmt<'sir> {
        let ty = l.ty.as_ref().map(|t| {
            self.lower_ty(t)
        });
        let init = l.kind.init().map(|init| self.lower_expr(init));
        let sir_id = self.lower_node_id(l.id);
        let pat = self.lower_pat(&l.pat);
        let span = l.span;
        self.arena.alloc(sir::LetStmt { sir_id, ty, pat, init, span })
    }

    fn with_loop_scope<T>(&mut self, loop_id: SirId, f: impl FnOnce(&mut Self) -> T) -> T {
        // 内側のループを処理するため、外側のループ条件式内にいるという状態を一時的に解除する
        let was_in_loop_condition = self.is_in_loop_condition;
        self.is_in_loop_condition = false;

        let old_scope = self.loop_scope.replace(loop_id);
        let result = f(self);
        self.loop_scope = old_scope;

        self.is_in_loop_condition = was_in_loop_condition;

        result
    }

    fn with_loop_condition_scope<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let was_in_loop_condition = self.is_in_loop_condition;
        self.is_in_loop_condition = true;

        let result = f(self);

        self.is_in_loop_condition = was_in_loop_condition;

        result
    }


    fn lower_loop_destination(&mut self) -> sir::Destination {
        let target_id = self.loop_scope
            .map(Ok)
            .unwrap_or(Err(sir::LoopIdError::OutsideLoopScope));
        sir::Destination { target_id }
    }

    fn lower_expr_while_in_loop_scope(
        &mut self,
        span: Span,
        cond: &ast::Expr,
        body: &ast::Block,
    ) -> sir::StmtKind<'sir> {
        let cond = self.with_loop_condition_scope(|t| t.lower_expr(cond));
        let then = self.lower_block_expr(body);
        let stmt_break = self.stmt_break(span);
        let else_block = self.block_all(span, self.arena.alloc([stmt_break]), None);
        let else_expr = self.arena.alloc(self.expr_block(else_block));
        let if_kind = sir::ExprKind::If(cond, self.arena.alloc(then), Some(else_expr));
        let if_expr = self.expr(span, if_kind);
        let block = self.block_expr(self.arena.alloc(if_expr));
        let span = span.merge(&cond.span);
        sir::StmtKind::Loop(block, sir::LoopSource::While, span)
    }

    pub fn stmt(&mut self, span: Span, kind: sir::StmtKind<'sir>) -> sir::Stmt<'sir> {
        let sir_id = self.next_id();
        sir::Stmt { sir_id, kind, span }
    }

    pub fn stmt_break(&mut self, span: Span) -> sir::Stmt<'sir> {
        let stmt_break = sir::StmtKind::Break(self.lower_loop_destination(), None);
        self.stmt(span, stmt_break)
    }
}
