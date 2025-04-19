pub mod def;

use crate::stelaro_common::Symbol;
use crate::stelaro_ty::Ty;

#[derive(Debug)]
pub enum Definition<'tcx> {
    Function { name: Symbol, params: &'tcx [Ty<'tcx>], return_ty: Ty<'tcx> },
    // Struct { name: Symbol, fields: &'tcx [FieldDef<'tcx>] },
}