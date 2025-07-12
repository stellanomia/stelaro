use crate::stelaro_ast::ast;
use crate::stelaro_ast_lowering::LoweringContext;
use crate::stelaro_common::{ensure_sufficient_stack, Ident};
use crate::stelaro_sir::{sir, sir_id::SirId, def::Res};



impl<'a, 'sir> LoweringContext<'a, 'sir> {
    pub fn lower_pat(&mut self, pat: &ast::Pat) -> &'sir sir::Pat {
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
            res @ (None | Some(Res::Local(_))) => {
                let binding_id = match res {
                    Some(Res::Local(id)) => {
                        if id == pat.id {
                            self.ident_to_local_id.insert(id, sir_id.local_id);
                            sir_id
                        } else {
                            SirId {
                                owner: self.current_sir_id_owner,
                                local_id: self.ident_to_local_id[&id],
                            }
                        }
                    },
                    _ => {
                        self.ident_to_local_id.insert(pat.id, sir_id.local_id);
                        sir_id
                    },
                };
                sir::PatKind::Binding(binding_id, ident)
            },
            Some(_) => {
                unimplemented!("Pattern は Path をとることはできない");
            }
        }
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
