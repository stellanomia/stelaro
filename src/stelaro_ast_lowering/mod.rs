mod block;
mod expr;
mod index;
mod item;
mod pat;
mod path;

use std::{collections::HashMap, thread};

use crate::stelaro_ast::{ast, visit, NodeId, ty::{Ty, TyKind}};
use crate::stelaro_ast_lowering::index::index_sir;
use crate::stelaro_common::{Arena, Idx, IndexVec, LocalDefId, SortedMap, Span, STELO_DEF_ID};
use crate::stelaro_context::TyCtxt;
use crate::stelaro_sir::{sir, def::Res, sir_id::{ItemLocalId, OwnerId, SirId, STELO_OWNER_ID}};
use crate::stelaro_ty::ResolverAstLowering;


struct LoweringContext<'a, 'sir> {
    pub tcx: TyCtxt<'sir>,
    pub resolver: &'a mut ResolverAstLowering,
    pub arena: &'sir Arena,

    /// ローワリング対象の所有ノードの中にあるボディ (関数本体など)。
    pub bodies: Vec<(ItemLocalId, &'sir sir::Body<'sir>)>,
    /// 現在の `owner` を Lowering することで生成されたアイテムを収集する。
    pub children: Vec<(LocalDefId, sir::MaybeOwner<'sir>)>,

    current_sir_id_owner: OwnerId,
    item_local_id_counter: ItemLocalId,

    /// Break, Continue が正しい文脈で使用されているか判定するのに使う。
    loop_scope: Option<SirId>,
    is_in_loop_condition: bool,

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
            loop_scope: None,
            is_in_loop_condition: false,
            item_local_id_counter: ItemLocalId::ZERO,
            #[cfg(debug_assertions)]
            node_id_to_local_id: HashMap::new(),
            ident_to_local_id: HashMap::new(),
        }
    }
}


#[derive(Debug, Clone, Copy)]
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

    drop(ast_index);

    thread::spawn(|| {
        drop(stelo);
    });

    sir::Stelo { owners }
}

impl<'a, 'sir> LoweringContext<'a, 'sir> {
    fn get_res(&self, id: NodeId) -> Option<Res<NodeId>> {
        self.resolver.res_map.get(&id).copied()
    }

    fn expect_res(&self, id: NodeId) -> Res<NodeId> {
        self.get_res(id).unwrap_or(Res::Err)
    }

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

        #[cfg(debug_assertions)]
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

    /// このメソッドは、与えられた `NodeId` に対して新しい `SirId` を割り当てます。
    /// 結果として得られる `SirId` が実際に SIR で使用されない場合は、このメソッドを呼び出さないようにする必要があります。
    /// 同じ `NodeId` でこのメソッドを2回呼び出すこともまた禁止されています。
    #[track_caller]
    fn lower_node_id(&mut self, ast_node_id: NodeId) -> SirId {
        let owner = self.current_sir_id_owner;
        let local_id = self.item_local_id_counter;
        assert_ne!(local_id, ItemLocalId::ZERO);
        self.item_local_id_counter.increment_by(1);
        let sir_id = SirId { owner, local_id };

        if let Some(def_id) = self.opt_local_def_id(ast_node_id) {
            self.children.push((def_id, sir::MaybeOwner::NonOwner(sir_id)));
        }

        // 同じ `NodeId` が複数回 lowering されていないかチェックします。
        #[cfg(debug_assertions)]
        {
            let old = self.node_id_to_local_id.insert(ast_node_id, local_id);
            assert_eq!(old, None);
        }

        sir_id
    }

    /// `NodeId` にを用いることなく、新しい `SirId` を生成します。
    fn next_id(&mut self) -> SirId {
        let owner = self.current_sir_id_owner;
        let local_id = self.item_local_id_counter;
        assert_ne!(local_id, ItemLocalId::ZERO);
        self.item_local_id_counter.increment_by(1);
        SirId { owner, local_id }
    }

    fn with_new_scopes<T>(&mut self, scope_span: Span, f: impl FnOnce(&mut Self) -> T) -> T {
        let current_item = self.current_item;
        self.current_item = Some(scope_span);

        let ret = f(self);

        self.current_item = current_item;

        ret
    }

    fn lower_res(&mut self, res: Res<NodeId>) -> Res {
        let res: Result<Res, ()> = res.apply_id(|id| {
            let owner = self.current_sir_id_owner;
            let local_id = self.ident_to_local_id.get(&id).copied().ok_or(())?;
            Ok(SirId { owner, local_id })
        });

        // Resが外側のSIR ownerのLocalを指している場合、SirIdを見つけられないことがあります。
        // これは、以下のような誤ったコードで戻り値の型 `x` をlower化しようとするときに起こり得ます。
        //   fn foo(x: i32) -> x {}
        res.unwrap_or(Res::Err)
    }

    fn lower_ty(&mut self, t: &Ty) -> &'sir sir::Ty<'sir> {
        self.arena.alloc(self.lower_ty_direct(t))
    }

    fn lower_path_ty(
        &mut self,
        t: &Ty,
        path: &ast::Path,
    ) -> sir::Ty<'sir> {
        sir::Ty {
            kind: sir::TyKind::Path(
                self.lower_path(t.id, path)
            ),
            span: t.span,
            sir_id: self.lower_node_id(t.id)
        }
    }

    fn lower_ty_direct(&mut self, t: &Ty) -> sir::Ty<'sir> {
        let kind = match &t.kind {
            TyKind::Path(path) => {
                return self.lower_path_ty(t, path);
            },
            TyKind::Infer => sir::TyKind::Infer,
            TyKind::Unit => sir::TyKind::Unit,
        };

        sir::Ty {
            sir_id: self.lower_node_id(t.id),
            span: t.span,
            kind
        }
    }

    fn lower_block_expr(&mut self, b: &ast::Block) -> sir::Expr<'sir> {
        let block = self.lower_block(b);
        self.expr_block(block)
    }

    fn block_expr(&mut self, expr: &'sir sir::Expr<'sir>) -> &'sir sir::Block<'sir> {
        self.block_all(expr.span, &[], Some(expr))
    }

    fn block_all(
        &mut self,
        span: Span,
        stmts: &'sir [sir::Stmt<'sir>],
        expr: Option<&'sir sir::Expr<'sir>>,
    ) -> &'sir sir::Block<'sir> {
        let block = sir::Block {
            stmts,
            expr,
            sir_id: self.next_id(),
            span,
        };

        self.arena.alloc(block)
    }
}
