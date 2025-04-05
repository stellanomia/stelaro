use crate::stelaro_context::TyCtxt;


struct Resolver<'tcx> {
    tcx: TyCtxt<'tcx>,
}