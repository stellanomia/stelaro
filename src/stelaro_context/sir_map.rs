use crate::{stelaro_context::TyCtxt, stelaro_sir::visit::Visitor};



impl<'tcx> TyCtxt<'tcx> {
    pub fn sir_visit_all_item_likes_in_stelo<V>(self, visitor: &mut V) -> V::Result
    where
        V: Visitor<'tcx>,
    {
        todo!()
    }
}
