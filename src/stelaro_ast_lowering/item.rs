use crate::stelaro_ast::ast::{self, ModSpan};
use crate::stelaro_ast::{NodeId, STELO_NODE_ID};
use crate::stelaro_ast_lowering::{AstOwner, LoweringContext};
use crate::stelaro_context::TyCtxt;
use crate::stelaro_common::{IndexSlice, IndexVec, LocalDefId, STELO_DEF_ID};
use crate::stelaro_sir::sir;
use crate::stelaro_ty::ResolverAstLowering;


pub struct ItemLowerer<'a, 'sir> {
    pub tcx: TyCtxt<'sir>,
    pub resolver: &'a mut ResolverAstLowering,
    pub ast_index: &'a IndexSlice<LocalDefId, AstOwner<'a>>,
    pub owners: &'a mut IndexVec<LocalDefId, sir::MaybeOwner<'sir>>,
}


impl<'a, 'sir> ItemLowerer<'a, 'sir> {
    fn with_lctx(
        &mut self,
        owner: NodeId,
        f: impl FnOnce(&mut LoweringContext<'_, 'sir>) -> sir::OwnerNode<'sir>,
    ) {
        let mut lctx = LoweringContext::new(self.tcx, self.resolver);
        lctx.with_sir_id_owner(owner, |lctx| f(lctx));

        for (def_id, info) in lctx.children {
            let owner = self.owners.ensure_contains_elem(def_id, || sir::MaybeOwner::Phantom);
            assert!(
                matches!(owner, sir::MaybeOwner::Phantom),
                "lctx.children に {def_id:?} の重複コピーがあります"
            );
            *owner = info;
        }
    }

    pub fn lower_node(&mut self, def_id: LocalDefId) {
        let owner = self.owners.ensure_contains_elem(def_id, || sir::MaybeOwner::Phantom);
        if let sir::MaybeOwner::Phantom = owner {
            let node = self.ast_index[def_id];
            match node {
                AstOwner::NonOwner => {}
                AstOwner::Stelo(s) => {
                    assert_eq!(self.resolver.node_id_to_def_id[&STELO_NODE_ID], STELO_DEF_ID);
                    self.with_lctx(STELO_NODE_ID, |lctx| {
                        let module = lctx.lower_mod(&s.items, &s.span);
                        sir::OwnerNode::Stelo(module)
                    })
                }
                AstOwner::Item(item) => {
                    self.with_lctx(item.id, |lctx|
                        sir::OwnerNode::Item(lctx.lower_item(item))
                    )
                }
            }
        }
    }
}

impl<'sir> LoweringContext<'_, 'sir> {
    pub fn lower_mod(
        &mut self,
        items: &[Box<ast::Item>],
        span: &ModSpan,
    ) -> &'sir sir::Mod<'sir> {
        self.arena.alloc(sir::Mod {
            spans: sir::ModSpan { inner_span: span.inner_span },
            item_ids: self.arena.alloc_from_iter(
                items.iter().flat_map(|x| self.lower_item_ref(x))
            ),
        })
    }

    pub fn lower_item(&mut self, item: &ast::Item) -> &'sir sir::Item<'sir> {
        todo!()
    }

    pub fn lower_item_ref(&mut self, item: &ast::Item) -> Vec<sir::ItemId>{
        todo!()
    }
}