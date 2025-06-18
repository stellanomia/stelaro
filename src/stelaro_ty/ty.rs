use crate::stelaro_common::{sym, DefId, Symbol};


#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Ty<'tcx>(&'tcx TyKind<'tcx>);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TyKind<'tcx> {
    Bool,
    Char,
    Int(IntTy),
    Uint(UintTy),
    Float(FloatTy),

    // 特定の関数定義を指す型
    FnDef(DefId),

    // 関数ポインタ
    // FnPtr(...),

    // 型パラメータ
    // Param(ParamTy),

    // 参照型
    // Ref(Ty<'tcx>),

    // Struct(AdtDef<'tcx>),

    // NOTE: タプルは未実装
    // 将来的なTyKindがもつべき'tcxのプレースホルダとして宣言
    Tuple(&'tcx [Ty<'tcx>]),

    // () 型。ボトム型として機能する
    // タプルが実装できた際に、これを削除し空のTupleがUnitを表すように変更する
    Unit,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrimTy {
    Bool,
    Char,
    Int(IntTy),
    Uint(UintTy),
    Float(FloatTy),
}

impl PrimTy {
    pub fn from_name(name: Symbol) -> Option<PrimTy> {
        let ty = match name {
            sym::BOOL => PrimTy::Bool,
            sym::CHAR => PrimTy::Char,
            sym::I32 => PrimTy::Int(IntTy::I32),
            sym::I64 => PrimTy::Int(IntTy::I64),
            _ => return None,
        };

        Some(ty)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IntTy {
    Isize,
    I8,
    I16,
    I32,
    I64,
    I128,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UintTy {
    Usize,
    U8,
    U16,
    U32,
    U64,
    U128,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FloatTy {
    F32,
    F64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParamTy {
    pub index: u32,
    pub name: Symbol,
}
