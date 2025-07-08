use crate::stelaro_ast::ast::{self, StmtKind};
use crate::stelaro_ast_lowering::LoweringContext;
use crate::stelaro_sir::{sir, sir_id::SirId};


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
            match &s.kind {
                StmtKind::Let(local) => {
                    let sir_id = self.lower_node_id(s.id);
                    let local = self.lower_local(local);
                    let kind = sir::StmtKind::Let(local);
                    let span = s.span;
                    stmts.push(sir::Stmt { sir_id, kind, span });
                }
                StmtKind::Expr(e) => {
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
                StmtKind::Semi(e) => {
                    let e = self.lower_expr(e);
                    let sir_id = self.lower_node_id(s.id);
                    let kind = sir::StmtKind::Semi(e);
                    let span = s.span;
                    stmts.push(sir::Stmt { sir_id, kind, span });
                }
                StmtKind::Return(e) => {
                    let e = e.as_ref().map(|e| self.lower_expr(e));
                    let sir_id = self.lower_node_id(s.id);
                    let kind = sir::StmtKind::Return(e);
                    let span = s.span;
                    stmts.push(sir::Stmt { sir_id, kind, span });
                }
                StmtKind::While(e, b) => {
                    let e = self.lower_expr(e);
                    let b = self.lower_block(b);
                    let sir_id = self.lower_node_id(s.id);
                    let kind = sir::StmtKind::While(e, b);
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
}
