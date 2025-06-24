use crate::stelaro_ast::ast;
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
        ast_stmts: &[ast::Stmt],
    ) -> (&'sir [sir::Stmt<'sir>], Option<&'sir sir::Expr<'sir>>) {
        todo!()
    }
}