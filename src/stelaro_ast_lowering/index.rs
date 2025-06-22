use std::collections::HashMap;

use crate::stelaro_common::{IndexVec, LocalDefId, SortedMap};
use crate::stelaro_sir::{sir::{self, Body, ParentedNode}, sir_id::{ItemLocalId, OwnerId}};
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
    todo!()
}