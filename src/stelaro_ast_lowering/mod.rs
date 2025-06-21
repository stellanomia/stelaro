mod item;

use std::collections::HashMap;

use crate::stelaro_ast::{ast, visit, NodeId};
use crate::stelaro_common::{Arena, IndexVec, LocalDefId, STELO_DEF_ID};
use crate::stelaro_context::TyCtxt;
use crate::stelaro_diagnostics::DiagCtxtHandle;
use crate::stelaro_sir::sir;
use crate::stelaro_ty::ResolverAstLowering;


struct LoweringContext<'a, 'sir> {
    pub tcx: TyCtxt<'sir>,
    pub resolver: &'a mut ResolverAstLowering,
    pub arena: &'sir Arena,

    /// 現在の `owner` を Lowering することで生成されたアイテムを収集する。
    pub children: Vec<(LocalDefId, sir::MaybeOwner<'sir>)>,
}

impl<'a, 'sir> LoweringContext<'a, 'sir> {
    fn new(tcx: TyCtxt<'sir>, resolver: &'a mut ResolverAstLowering) -> Self {
        Self {
            tcx,
            resolver,
            arena: tcx.sir_arena,
            children: Vec::new(),
        }
    }

    pub fn dcx(&self) -> DiagCtxtHandle<'sir> {
        self.tcx.dcx()
    }
}


#[derive(Clone, Copy)]
enum AstOwner<'a> {
    NonOwner,
    Stelo(&'a ast::Stelo),
    Item(&'a ast::Item),
}

fn index_stelo<'a>(
    node_id_to_def_id: &HashMap<NodeId, LocalDefId>,
    stelo: &'a ast::Stelo,
) -> IndexVec<LocalDefId, AstOwner<'a>> {
    let mut indexer = Indexer { node_id_to_def_id, index: IndexVec::new() };
    *indexer.index
    .ensure_contains_elem(STELO_DEF_ID, || AstOwner::NonOwner) = AstOwner::Stelo(stelo);

    visit::walk_stelo(&mut indexer, stelo);
    return indexer.index;

    struct Indexer<'s, 'a> {
        node_id_to_def_id: &'s HashMap<NodeId, LocalDefId>,
        index: IndexVec<LocalDefId, AstOwner<'a>>,
    }

    impl<'a> visit::Visitor<'a> for Indexer<'_, 'a> {
        fn visit_item(&mut self, item: &'a ast::Item) {
            let def_id = self.node_id_to_def_id[&item.id];
            *self.index.ensure_contains_elem(def_id, || AstOwner::NonOwner) = AstOwner::Item(item);
            visit::walk_item(self, item)
        }
    }
}

pub fn lower_to_sir(
    tcx: TyCtxt<'_>,
    mut resolver: ResolverAstLowering,
    stelo: ast::Stelo,
) -> sir::Stelo<'_> {
    let ast_index = index_stelo(&resolver.node_id_to_def_id, &stelo);

    let mut owners = IndexVec::from_fn_n(
        |_| sir::MaybeOwner::Phantom,
        tcx.definitions.borrow().def_index_count(),
    );

    let mut lowerer = item::ItemLowerer {
        tcx,
        resolver: &mut resolver,
        ast_index: &ast_index,
        owners: &mut owners,
    };

    for def_id in ast_index.indices() {
        lowerer.lower_node(def_id);
    }

    todo!()
}

impl<'a, 'sir> LoweringContext<'a, 'sir> {
    /// `LoweringContext` をリフレッシュし、ネストしたアイテムを `lower` 化する準備を整えます。
    /// `lower` 化されたアイテムは `self.children` に登録されます。
    ///
    /// この関数は `SirId` の `lower` 化のための基盤をセットアップし、
    /// 共有の可変状態を退避させることで、クロージャによる状態の汚染を防ぎます。
    fn with_sir_id_owner(
        &mut self,
        owner: NodeId,
        f: impl FnOnce(&mut Self) -> sir::OwnerNode<'sir>,
    ) {
        todo!()
    }
}
