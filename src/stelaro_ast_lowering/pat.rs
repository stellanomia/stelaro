use crate::stelaro_ast::ast;
use crate::stelaro_ast_lowering::LoweringContext;
use crate::stelaro_sir::sir;



impl<'a, 'sir> LoweringContext<'a, 'sir> {
    pub(crate) fn lower_pat(&mut self, pattern: &ast::Pat) -> &'sir sir::Pat<'sir> {
        self.arena.alloc(self.lower_pat_mut(pattern))
    }

    fn lower_pat_mut(&mut self, mut pattern: &ast::Pat) -> sir::Pat<'sir> {
        todo!()
    }
}
