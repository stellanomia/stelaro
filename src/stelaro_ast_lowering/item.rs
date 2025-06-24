use crate::stelaro_ast::{ast::{self, ModSpan}, NodeId, STELO_NODE_ID};
use crate::stelaro_ast_lowering::{AstOwner, LoweringContext};
use crate::stelaro_context::TyCtxt;
use crate::stelaro_common::{IndexSlice, IndexVec, LocalDefId, Span, STELO_DEF_ID};
use crate::stelaro_sir::{sir, sir_id::SirId};
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

    pub fn lower_item_ref(&mut self, item: &ast::Item) -> Vec<sir::ItemId> {
        let node_ids = vec![
            sir::ItemId { owner_id: self.owner_id(item.id) }
        ];
        // if let ItemKind::Use(use_tree) = &item.kind {
        //     self.lower_item_id_use_tree(use_tree, &mut node_ids);
        // }
        node_ids
    }

    pub fn lower_item(&mut self, item: &ast::Item) -> &'sir sir::Item<'sir> {
        let sir_id = SirId::make_owner(self.current_sir_id_owner.def_id);
        let kind = self.lower_item_kind(item.span, item.id, sir_id, &item.kind);
        let item = sir::Item {
            owner_id: sir_id.expect_owner(),
            kind,
            span: item.span,
        };
        self.arena.alloc(item)
    }

    fn lower_item_kind(
        &mut self,
        span: Span,
        id: NodeId,
        sir_id: SirId,
        i: &ast::ItemKind,
    ) -> sir::ItemKind<'sir> {
        use ast::ItemKind;

        match i {
            ItemKind::Fn(box ast::Function {
                span,
                ident,
                sig: ast::FnSig {
                    decl,
                    span: fn_sig_span,
                    ..
                },
                body,
                ..
            }) => {
                self.with_new_scopes(*fn_sig_span, |this| {
                    let body_id = this.lower_body(|this| {
                        todo!()
                    });
                    let decl = this.lower_fn_decl();

                    let sig = sir::FnSig {
                        decl: todo!(),
                        span: *fn_sig_span,
                    };

                    sir::ItemKind::Fn {
                        ident: *ident,
                        sig,
                        body: body_id,
                    }
                });
                todo!()
            },
            ItemKind::Mod(ident, module) => {
                match module {
                    ast::ModKind::Inline(
                        items,
                        mod_span
                    ) => sir::ItemKind::Mod(*ident, self.lower_mod(items, mod_span)),
                }
            },
        }
    }

    fn record_body(
        &mut self,
        params: &'sir [sir::Param<'sir>],
        value: sir::Expr<'sir>,
    ) -> sir::BodyId {
        let body = sir::Body { params, value: self.arena.alloc(value) };
        let id = body.id();
        assert_eq!(id.sir_id.owner, self.current_sir_id_owner);
        self.bodies.push((id.sir_id.local_id, self.arena.alloc(body)));
        id
    }

    pub fn lower_body(
        &mut self,
        f: impl FnOnce(&mut Self) -> (&'sir [sir::Param<'sir>], sir::Expr<'sir>),
    ) -> sir::BodyId {
        let (parameters, result) = f(self);
        self.record_body(parameters, result)
    }

    fn lower_fn_decl(
        &mut self,
    ) -> &'sir sir::FnDecl<'sir> {
        todo!()
    }
}