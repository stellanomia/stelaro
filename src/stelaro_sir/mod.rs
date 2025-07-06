pub mod def;
pub mod definitions;
pub mod sir;
pub mod sir_id;
pub mod visit;

use crate::stelaro_common::LocalDefId;
use crate::stelaro_context::TyCtxt;
use crate::stelaro_sir::sir_id::{ItemLocalId, OwnerId, STELO_SIR_ID};
use crate::stelaro_sir::{sir::OwnerNodes, sir_id::SirId};


impl<'tcx> TyCtxt<'tcx> {
    #[inline]
    pub fn opt_sir_owner_nodes(self, def_id: LocalDefId) -> Option<&'tcx OwnerNodes<'tcx>> {
        self.sir_stelo?.owners.get(def_id)?.as_owner().map(|i| &i.nodes)
    }

    #[inline]
    pub fn local_def_id_to_sir_id(self, def_id: LocalDefId) -> SirId {
        self.sir_stelo.map(|stelo| {
            match stelo.owners[def_id] {
                sir::MaybeOwner::Owner(_) => SirId::make_owner(def_id),
                sir::MaybeOwner::NonOwner(sir_id) => sir_id,
                sir::MaybeOwner::Phantom => panic!("bug: {:?} に SirId はない", def_id),
            }
        }).unwrap()
    }

    #[inline]
    pub fn sir_owner_parent(self, owner_id: OwnerId) -> SirId {
        self.opt_local_parent(owner_id.def_id).map_or(STELO_SIR_ID, |parent_def_id| {
            let parent_owner_id = self.local_def_id_to_sir_id(parent_def_id).owner;
            SirId {
                owner: parent_owner_id,
                local_id: self.sir_stelo.unwrap().owners[parent_owner_id.def_id]
                    .unwrap()
                    .parenting
                    .get(&owner_id.def_id)
                    .copied()
                    .unwrap_or(ItemLocalId::ZERO),
            }
        })
    }
}
