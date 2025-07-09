use crate::stelaro_ast::ast;
use crate::stelaro_ast_lowering::LoweringContext;
use crate::stelaro_common::{ensure_sufficient_stack, Ident};
use crate::stelaro_sir::{sir, sir_id::SirId, def::Res};



impl<'a, 'sir> LoweringContext<'a, 'sir> {
    pub(crate) fn lower_pat(&mut self, pat: &ast::Pat) -> &'sir sir::Pat {
        self.arena.alloc(self.lower_pat_mut(pat))
    }

    fn lower_pat_mut(&mut self, pat: &ast::Pat) -> sir::Pat {
        ensure_sufficient_stack(|| {
            let pat_sir_id = self.lower_node_id(pat.id);
            let node = match pat.kind {
                ast::PatKind::WildCard => sir::PatKind::WildCard,
                ast::PatKind::Ident(ident) => {
                    self.lower_pat_ident(pat, ident, pat_sir_id)
                },
            };

            self.pat_with_node_id_of(pat, node, pat_sir_id)
        })
    }


    fn lower_pat_ident(
        &mut self,
        pat: &ast::Pat,
        ident: Ident,
        sir_id: SirId,
    ) -> sir::PatKind {
        match self.get_res(pat.id) {
            None | Some(Res::Local(_)) => todo!(),
            Some(_) => {
                todo!()
            }
        }
        todo!()
    }

    fn pat_with_node_id_of(
        &mut self,
        p: &ast::Pat,
        kind: sir::PatKind,
        sir_id: SirId,
    ) -> sir::Pat {
        sir::Pat { sir_id, kind, span: p.span }
    }
}
