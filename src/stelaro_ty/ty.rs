use crate::{stelaro_common::{DefId, Symbol}, stelaro_diagnostics::ErrorEmitted};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Ty<'tcx>(pub &'tcx TyKind<'tcx>);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TyKind<'tcx> {
    Bool,
    Char,
    Str,
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

    // 発散型
    Never,
    Error(ErrorEmitted),
}

impl<'tcx> Ty<'tcx> {
    pub fn kind(&self) -> &'tcx TyKind<'tcx> {
        self.0
    }

    pub fn is_error(&self) -> bool {
        matches!(self.kind(), TyKind::Error(_))
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
