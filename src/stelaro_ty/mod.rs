use crate::stelaro_common::{DefId, Symbol};


#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Ty<'ctx>(&'ctx TyKind<'ctx>);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TyKind<'tcx> {
    Bool,
    Char,
    Int(IntTy),
    Uint(UintTy),
    Float(FloatTy),
    FnDef(DefId),
    Param(ParamTy),
    // Ref(Ty<'tcx>),
    // Struct(DefId),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum IntTy {
    Isize,
    I8,
    I16,
    I32,
    I64,
    I128,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum UintTy {
    Usize,
    U8,
    U16,
    U32,
    U64,
    U128,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum FloatTy {
    F32,
    F64,
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