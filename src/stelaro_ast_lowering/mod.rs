mod index;
mod item;

use std::collections::HashMap;

use crate::stelaro_ast::{ast, visit, NodeId};
use crate::stelaro_ast_lowering::index::index_sir;
use crate::stelaro_common::{Arena, IndexVec, LocalDefId, SortedMap, Span, STELO_DEF_ID};
use crate::stelaro_context::TyCtxt;
use crate::stelaro_diagnostics::DiagCtxtHandle;
use crate::stelaro_sir::{sir, sir_id::{ItemLocalId, OwnerId, STELO_OWNER_ID}};
use crate::stelaro_ty::ResolverAstLowering;


struct LoweringContext<'a, 'sir> {
    pub tcx: TyCtxt<'sir>,
    pub resolver: &'a mut ResolverAstLowering,
    pub arena: &'sir Arena,

    /// ローワリング対象の所有ノードの中にあるボディ（関数本体など）。
    pub bodies: Vec<(ItemLocalId, &'sir sir::Body<'sir>)>,
    /// 現在の `owner` を Lowering することで生成されたアイテムを収集する。
    pub children: Vec<(LocalDefId, sir::MaybeOwner<'sir>)>,

    current_sir_id_owner: OwnerId,
    item_local_id_counter: ItemLocalId,

    /// 現在の SIR 所有ノード内でローワリングされるNodeId。
    /// 重複ローワリングの検査にのみ使用される。
    #[cfg(debug_assertions)]
    node_id_to_local_id: HashMap<NodeId, ItemLocalId>,
    current_item: Option<Span>,

    /// 現在の SIR 所有ノード内でローワリングされる、パターン識別子の NodeId。
    ident_to_local_id: HashMap<NodeId, ItemLocalId>,
}

impl<'a, 'sir> LoweringContext<'a, 'sir> {
    fn new(tcx: TyCtxt<'sir>, resolver: &'a mut ResolverAstLowering) -> Self {
        Self {
            tcx,
            resolver,
            arena: tcx.sir_arena,
            children: Vec::new(),
            bodies: Vec::new(),
            current_sir_id_owner: STELO_OWNER_ID,
            current_item: None,
            item_local_id_counter: ItemLocalId::ZERO,
            node_id_to_local_id: HashMap::new(),
            ident_to_local_id: HashMap::new(),
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
    /// AST内のあるノードのIDが与えられたときに、それに対応する `LocalDefId` を名前解決器から (存在すれば) 取得する。
    fn opt_local_def_id(&self, node: NodeId) -> Option<LocalDefId> {
        self.resolver.node_id_to_def_id.get(&node).copied()
    }

    fn local_def_id(&self, node: NodeId) -> LocalDefId {
        self.opt_local_def_id(node).unwrap_or_else(|| panic!("ノードID `{node:?}` に対応するエントリが存在しません"))
    }

    /// AST内の所有ノードのIDが与えられたときに、それに対応する `OwnerId` を返す。
    fn owner_id(&self, node: NodeId) -> OwnerId {
        OwnerId { def_id: self.local_def_id(node) }
    }

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
        let owner_id = self.owner_id(owner);

        let current_bodies = std::mem::take(&mut self.bodies);
        let current_ident_to_local_id =
            std::mem::take(&mut self.ident_to_local_id);

        let current_node_id_to_local_id = std::mem::take(&mut self.node_id_to_local_id);
        let current_owner = std::mem::replace(&mut self.current_sir_id_owner, owner_id);
        let current_local_counter =
            std::mem::replace(&mut self.item_local_id_counter, ItemLocalId::new(1));


        // `next_node_id` と `node_id_to_def_id` はリセットしない：
        // 呼び出し側が作成した `LocalDefId` を `f` が参照できるようにするため。
        // また、呼び出し側が一部のサブ定義ノードの `LocalDefId` を参照できるようにするため。

        // オーナー自身に対して、最初の `SirId` は常に割り当てる。
        #[cfg(debug_assertions)]
        {
            let _old = self.node_id_to_local_id.insert(owner, ItemLocalId::ZERO);
            debug_assert_eq!(_old, None);
        }

        let item = f(self);
        assert_eq!(owner_id, item.def_id());
        let info = self.make_owner_info(item);

        self.bodies = current_bodies;
        self.ident_to_local_id = current_ident_to_local_id;

        #[cfg(debug_assertions)]
        {
            self.node_id_to_local_id = current_node_id_to_local_id;
        }
        self.current_sir_id_owner = current_owner;
        self.item_local_id_counter = current_local_counter;

        debug_assert!(!self.children.iter().any(|(id, _)| id == &owner_id.def_id));
        self.children.push((owner_id.def_id, sir::MaybeOwner::Owner(info)));
    }

    fn make_owner_info(&mut self, node: sir::OwnerNode<'sir>) -> &'sir sir::OwnerInfo<'sir> {
        let mut bodies = std::mem::take(&mut self.bodies);

        bodies.sort_by_key(|(k, _)| *k);
        let bodies = SortedMap::from_presorted_elements(bodies);

        let num_nodes = self.item_local_id_counter.as_usize();
        let (nodes, parenting) = index_sir(self.tcx, node, &bodies, num_nodes);
        let nodes = sir::OwnerNodes { nodes, bodies };

        self.arena.alloc(sir::OwnerInfo { nodes, parenting })
    }

    fn with_new_scopes<T>(&mut self, scope_span: Span, f: impl FnOnce(&mut Self) -> T) -> T {
        let current_item = self.current_item;
        self.current_item = Some(scope_span);

        let ret = f(self);

        self.current_item = current_item;

        ret
    }
}
