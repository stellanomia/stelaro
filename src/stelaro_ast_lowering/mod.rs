use crate::{stelaro_context::TyCtxt, stelaro_ty::ResolverAstLowering};


struct LoweringContext<'a, 'sir> {
    tcx: TyCtxt<'sir>,
    resolver: &'a mut ResolverAstLowering,
}