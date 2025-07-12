use std::collections::HashMap;

use crate::stelaro_common::{IndexVec, LocalDefId, SortedMap, Span};
use crate::stelaro_sir::{
    sir::{self, *},
    sir_id::{ItemLocalId, OwnerId, SirId, STELO_SIR_ID},
    visit,
    Visitor,
};
use crate::stelaro_context::TyCtxt;


/// SIR を巡回して `Node` を収集し、SIR マップに格納するビジター。
struct NodeCollector<'a, 'sir> {
    tcx: TyCtxt<'sir>,

    /// 関数や定数などの本体 (Body) のマップへの参照
    bodies: &'a SortedMap<ItemLocalId, &'sir Body<'sir>>,

    /// 出力
    nodes: IndexVec<ItemLocalId, ParentedNode<'sir>>,
    parenting: HashMap<LocalDefId, ItemLocalId>,

    /// このノードの親
    parent_node: ItemLocalId,

    /// 所有者 (SIRオーナー)
    owner: OwnerId,
}

pub fn index_sir<'sir>(
    tcx: TyCtxt<'sir>,
    item: sir::OwnerNode<'sir>,
    bodies: &SortedMap<ItemLocalId, &'sir Body<'sir>>,
    num_nodes: usize,
) -> (IndexVec<ItemLocalId, ParentedNode<'sir>>, HashMap<LocalDefId, ItemLocalId>) {
    let err_node = ParentedNode { parent: ItemLocalId::ZERO, node: Node::Err(item.span()) };
    let mut nodes = IndexVec::from_elem_n(err_node, num_nodes);
    // このノードの親は決してアクセスされるべきではありません。
    // 使用された場合に内部コンパイラエラーを強制的に発生させるよう、無効な値に設定している。
    nodes[ItemLocalId::ZERO] = ParentedNode { parent: ItemLocalId::INVALID, node: item.into() };
    let mut collector = NodeCollector {
        tcx,
        owner: item.def_id(),
        parent_node: ItemLocalId::ZERO,
        nodes,
        bodies,
        parenting: Default::default(),
    };


    match item {
        OwnerNode::Stelo(sitem) => {
            collector.visit_mod(sitem, sitem.spans.inner_span, STELO_SIR_ID)
        }
        OwnerNode::Item(item) => collector.visit_item(item),
    };

    for (local_id, node) in collector.nodes.iter_enumerated() {
        if let Node::Err(_) = node.node {
            let sir_id = SirId { owner: item.def_id(), local_id };
            panic!("アイテムのSIRを走査する際に、ID {sir_id} に遭遇しませんでした");
        }
    }

    (collector.nodes, collector.parenting)
}

impl<'a, 'sir> NodeCollector<'a, 'sir> {
    fn insert(&mut self, span: Span, sir_id: SirId, node: Node<'sir>) {
       debug_assert_eq!(self.owner, sir_id.owner);
       debug_assert_ne!(sir_id.local_id.as_u32(), 0);
       debug_assert_ne!(sir_id.local_id, self.parent_node);

        // あるノードのDepNodeが、そのノードのSirId ownerと一致することを確認します。
        if cfg!(debug_assertions) && sir_id.owner != self.owner {
                panic!(
                   "{:?} において {node:?} に対する SirId が矛盾しています: \
                    current_dep_node_owner={} ({:?}), sir_id.owner={} ({:?})",
                   span,
                   self.tcx.definitions
                       .borrow()
                       .def_path(self.owner.def_id)
                       .to_string_no_stelo_verbose(),
                   self.owner,
                   self.tcx.definitions
                       .borrow()
                       .def_path(sir_id.owner.def_id)
                       .to_string_no_stelo_verbose(),
                   sir_id.owner,
               )
        }

        self.nodes[sir_id.local_id] = ParentedNode { parent: self.parent_node, node };
    }

    fn with_parent<F: FnOnce(&mut Self)>(&mut self, parent_node_id: SirId, f: F) {
        debug_assert_eq!(parent_node_id.owner, self.owner);
        let parent_node = self.parent_node;
        self.parent_node = parent_node_id.local_id;
        f(self);
        self.parent_node = parent_node;
    }

    fn insert_nested(&mut self, item: LocalDefId) {
        if self.parent_node != ItemLocalId::ZERO {
            self.parenting.insert(item, self.parent_node);
        }
    }
}

impl<'a, 'sir> Visitor<'sir> for NodeCollector<'a, 'sir> {
    fn visit_nested_item(&mut self, item: sir::ItemId) -> Self::Result {
        self.insert_nested(item.owner_id.def_id);
    }

    fn visit_nested_body(&mut self, id: sir::BodyId) -> Self::Result {
        debug_assert_eq!(id.sir_id.owner, self.owner);
        let body = self.bodies[&id.sir_id.local_id];
        self.visit_body(body)
    }

    fn visit_param(&mut self, param: &'sir Param) {
        let node = Node::Param(param);
        self.insert(param.ident.span, param.sir_id, node);
        self.with_parent(param.sir_id, |this| {
            visit::walk_param(this, param);
        });
    }

    fn visit_item(&mut self, i: &'sir Item<'sir>) -> Self::Result {
        debug_assert_eq!(i.owner_id, self.owner);
        self.with_parent(i.sir_id(), |this| {
            visit::walk_item(this, i);
        });
    }

    fn visit_pat(&mut self, pat: &'sir Pat) {
        self.insert(pat.span, pat.sir_id, Node::Pat(pat));

        self.with_parent(pat.sir_id, |this| {
            visit::walk_pat(this, pat);
        });
    }

    fn visit_expr(&mut self, expr: &'sir Expr<'sir>) {
        self.insert(expr.span, expr.sir_id, Node::Expr(expr));

        self.with_parent(expr.sir_id, |this| {
            visit::walk_expr(this, expr);
        });
    }

    fn visit_stmt(&mut self, stmt: &'sir Stmt<'sir>) {
        self.insert(stmt.span, stmt.sir_id, Node::Stmt(stmt));

        self.with_parent(stmt.sir_id, |this| {
            visit::walk_stmt(this, stmt);
        });
    }

    fn visit_path_segment(&mut self, path_segment: &'sir PathSegment) -> Self::Result {
        self.insert(path_segment.ident.span, path_segment.sir_id, Node::PathSegment(path_segment));

        self.with_parent(path_segment.sir_id, |this| {
            visit::walk_path_segment(this, path_segment);
        });
    }

    fn visit_ty(&mut self, ty: &'sir Ty<'sir>) -> Self::Result {
        self.insert(ty.span, ty.sir_id, Node::Ty(ty));

        self.with_parent(ty.sir_id, |this| {
            visit::walk_ty(this, ty);
        });
    }

    fn visit_infer(
        &mut self,
        inf_id: SirId,
        _inf_span: Span,
    ) -> Self::Result {
        self.visit_id(inf_id)
    }

    fn visit_block(&mut self, block: &'sir Block<'sir>) {
        self.insert(block.span, block.sir_id, Node::Block(block));
        self.with_parent(block.sir_id, |this| {
            visit::walk_block(this, block);
        });
    }

    fn visit_local(&mut self, l: &'sir LetStmt<'sir>) {
        self.insert(l.span, l.sir_id, Node::LetStmt(l));
        self.with_parent(l.sir_id, |this| {
            visit::walk_local(this, l);
        })
    }
}
