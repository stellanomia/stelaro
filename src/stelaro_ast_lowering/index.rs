use std::collections::HashMap;

use crate::stelaro_common::{IndexVec, LocalDefId, SortedMap};
use crate::stelaro_sir::{
    sir::{self, Body, ParentedNode, Node},
    sir_id::{ItemLocalId, OwnerId, SirId},
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


    // match item {
    //     OwnerNode::Stelo(sitem) => {
    //         collector.visit_mod(sitem, sitem.spans.inner_span, STELO_SIR_ID)
    //     }
    //     OwnerNode::Item(item) => collector.visit_item(item),
    // };

    for (local_id, node) in collector.nodes.iter_enumerated() {
        if let Node::Err(_) = node.node {
            let sir_id = SirId { owner: item.def_id(), local_id };
            panic!("アイテムのSIRを走査する際に、ID {sir_id} に遭遇しませんでした");
        }
    }

    (collector.nodes, collector.parenting)
}
