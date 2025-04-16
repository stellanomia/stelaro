use crate::stelaro_common::Symbol;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum PrimTy {
    Bool,
    Char,
    Int(IntTy),
    Uint(UintTy),
    Float(FloatTy),
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


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParamTy {
    pub index: u32,
    pub name: Symbol,
}
