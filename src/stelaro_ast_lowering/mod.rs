use crate::stelaro_common::Arena;
use crate::stelaro_context::TyCtxt;
use crate::stelaro_diagnostics::DiagCtxtHandle;
use crate::stelaro_sir::sir;
use crate::stelaro_ty::ResolverAstLowering;


struct LoweringContext<'a, 'sir> {
    pub tcx: TyCtxt<'sir>,
    pub resolver: &'a mut ResolverAstLowering,
    pub arena: &'sir Arena,
}

impl<'a, 'sir> LoweringContext<'a, 'sir> {
    fn new(tcx: TyCtxt<'sir>, resolver: &'a mut ResolverAstLowering) -> Self {
        Self {
            tcx,
            resolver,
            arena: tcx.sir_arena,
        }
    }

    pub fn dcx(&self) -> DiagCtxtHandle<'sir> {
        self.tcx.dcx()
    }
}


pub fn lower_to_sir(tcx: TyCtxt<'_>) -> sir::Crate {
    todo!()
}
