use crate::stelaro_common::{DefId, Symbol};


#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Ty<'ctx>(&'ctx TyKind<'ctx>);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TyKind<'tcx> {
    // Struct(DefId),
    // Ref(Ty<'tcx>),
    FnDef(DefId, &'tcx [Ty<'tcx>], Ty<'tcx>),
    Param(ParamTy),
}

#[derive(Debug)]
pub enum Definition<'tcx> {
    Function { name: Symbol, params: &'tcx [Ty<'tcx>], return_ty: Ty<'tcx> },
    // Struct { name: Symbol, fields: &'tcx [FieldDef<'tcx>] },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParamTy {
    pub index: u32,
    pub name: Symbol,
}