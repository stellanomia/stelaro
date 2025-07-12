use crate::stelaro_common::{DefId, DefPathHash, LocalDefId};
use crate::stelaro_context::TyCtxt;
use crate::stelaro_sir::{
    sir::{Body, BodyId, Item, ItemId, Node, OwnerNode, OwnerNodes, Mod},
    sir_id::{SirId, OwnerId, ItemLocalId, STELO_OWNER_ID},
    visit::SirTyCtxt,
    DefKey, DefPath,
};


impl<'tcx> TyCtxt<'tcx> {
    #[inline]
    fn expect_sir_owner_nodes(self, def_id: LocalDefId) -> &'tcx OwnerNodes<'tcx> {
        self.opt_sir_owner_nodes(def_id)
            .unwrap_or_else(|| panic!("bug: {def_id:?} is not an owner"))
    }

    #[inline]
    pub fn sir_owner_nodes(self, owner_id: OwnerId) -> &'tcx OwnerNodes<'tcx> {
        self.expect_sir_owner_nodes(owner_id.def_id)
    }

    #[inline]
    pub fn expect_sir_owner_node(self, def_id: LocalDefId) -> OwnerNode<'tcx> {
        self.expect_sir_owner_nodes(def_id).node()
    }

    #[inline]
    pub fn sir_owner_node(self, owner_id: OwnerId) -> OwnerNode<'tcx> {
        self.sir_owner_nodes(owner_id).node()
    }

    /// `id` に対応する `sir::Node` を取得します。
    pub fn sir_node(self, id: SirId) -> Node<'tcx> {
        self.sir_owner_nodes(id.owner).nodes[id.local_id].node
    }

    /// `id` に対応する `sir::Node` を取得します。
    #[inline]
    pub fn sir_node_by_def_id(self, id: LocalDefId) -> Node<'tcx> {
        self.sir_node(self.local_def_id_to_sir_id(id))
    }

    /// この `sir_id` を持つノードの親HIRノードの `SirId` を返します。
    /// `sir_id == CRATE_HIR_ID` の場合に限り、同じ `sir_id` を返します。
    ///
    /// 繰り返し呼び出して親をイテレートする場合は、[`TyCtxt::sir_parent_iter`] の使用を推奨します。
    pub fn parent_sir_id(self, sir_id: SirId) -> SirId {
        let SirId { owner, local_id } = sir_id;
        if local_id == ItemLocalId::ZERO {
            self.sir_owner_parent(owner)
        } else {
            let parent_local_id = self.sir_owner_nodes(owner).nodes[local_id].parent;
            // HIRのインデックス付けでチェックされているはずです。
            debug_assert_ne!(parent_local_id, local_id);
            SirId { owner, local_id: parent_local_id }
        }
    }

    /// この `sir_id` を持つノードの親HIRノードを返します。
    /// `sir_id == CRATE_HIR_ID` の場合に限り、同じ `sir_id` のHIRノードを返します。
    pub fn parent_sir_node(self, sir_id: SirId) -> Node<'tcx> {
        self.sir_node(self.parent_sir_id(sir_id))
    }

    #[inline]
    pub fn sir_root_module(self) -> &'tcx Mod<'tcx> {
        match self.sir_owner_node(STELO_OWNER_ID) {
            OwnerNode::Stelo(item) => item,
            _ => unreachable!(),
        }
    }

    pub fn sir_def_key(self, def_id: LocalDefId) -> DefKey {
        // DefKey へのアクセスは DefPathHash の一部であるため、問題ない
        self.definitions.borrow().def_key(def_id)
    }

    pub fn sir_def_path(self, def_id: LocalDefId) -> DefPath {
        // DefKey へのアクセスは DefPathHash の一部であるため、問題ない
        self.definitions.borrow().def_path(def_id)
    }

    #[inline]
    pub fn sir_def_path_hash(self, def_id: LocalDefId) -> DefPathHash {
        self.definitions.borrow().def_path_hash(def_id)
    }

    pub fn sir_get_if_local(self, id: DefId) -> Option<Node<'tcx>> {
        id.as_local().map(|id| self.sir_node_by_def_id(id))
    }

    pub fn sir_item(self, id: ItemId) -> &'tcx Item<'tcx> {
        match self.sir_owner_node(id.owner_id) {
            OwnerNode::Item(item) => item,
            _ => panic!("bug: Item ではありません"),
        }
    }

    pub fn sir_body(self, id: BodyId) -> &'tcx Body<'tcx> {
        self.sir_owner_nodes(id.sir_id.owner).bodies[&id.sir_id.local_id]
    }

    pub fn sir_body_owner_def_id(self, BodyId { sir_id }: BodyId) -> LocalDefId {
        self.parent_sir_node(sir_id).associated_body().unwrap().0
    }
}

impl<'tcx> SirTyCtxt<'tcx> for TyCtxt<'tcx> {
    fn sir_node(&self, sir_id: SirId) -> Node<'tcx> {
        (*self).sir_node(sir_id)
    }

    fn sir_body(&self, id: BodyId) -> &'tcx Body<'tcx> {
        (*self).sir_body(id)
    }

    fn sir_item(&self, id: ItemId) -> &'tcx Item<'tcx> {
        (*self).sir_item(id)
    }
}
