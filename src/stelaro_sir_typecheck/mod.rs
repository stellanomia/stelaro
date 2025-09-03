pub mod result;

use crate::stelaro_context::TyCtxt;


pub struct TypeckCtxt<'tcx> {
    pub tcx: TyCtxt<'tcx>,
}