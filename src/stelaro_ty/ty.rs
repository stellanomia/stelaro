use crate::stelaro_common::{DefId, Symbol};
use crate::stelaro_diagnostics::ErrorEmitted;

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

    Infer(InferTy),

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

/// まだ型が確定していない「未推論の型」を表すためのプレースホルダ。
///
/// 例えば、空の配列 `[]` があったとき、その要素の型はすぐには分かりません。
/// このような場合に、一度"型変数"を割り当てておき、後で本来の型を推論します。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InferTy {
    /// 最も一般的な型変数。`?T` のように表現される。
    ///
    /// `let x = some_fn();` のように、戻り値の型が文脈から明らかでない場合に
    /// `x` の型として使われます。
    /// この変数は、`i32`, `bool`, 構造体など、どんな型にもなり得ます。
    TyVar(TyVid),

    /// 「何らかの整数型」を表す型変数。`{integer}` のように表現される。
    ///
    /// `let num = 100;` のように整数リテラルが現れたときに使われます。
    /// この変数は `i32` や `u64` などの整数型にしかなれません。
    /// これにより、`if 1 { ... }` のようなコードで早期に型エラーを検出できます。
    IntVar(IntVid),

    /// 「何らかの浮動小数点数型」を表す型変数。`{float}` のように表現される。
    ///
    /// `let pi = 3.14;` のように浮動小数点数リテラルが現れたときに使われます。
    /// この変数は `f32` または `f64` にしかなれません。
    FloatVar(FloatVid),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct TyVid(pub u32);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct IntVid(pub u32);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct FloatVid(pub u32);


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
