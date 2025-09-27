use std::marker::PhantomData;

use ena::unify::{NoError, UnificationTableStorage, UnifyKey, UnifyValue};

use crate::stelaro_common::{DefId, IndexVec, Span};
use crate::stelaro_context::TyCtxt;
use crate::stelaro_ty::Ty;
use crate::stelaro_ty::ty::TyVid;

pub struct InferCtxt<'tcx> {
    pub tcx: TyCtxt<'tcx>,
}

#[derive(Clone, Default)]
pub(crate) struct TypeVariableStorage<'tcx> {
    /// 各型変数の発生源などを記録する。
    values: IndexVec<TyVid, TypeVariableData>,

    /// 型変数の等価関係と、具体的な型への束縛を管理するUnion-Findテーブル。
    eq_relations: UnificationTableStorage<TyVidEqKey<'tcx>>,

    // sub_unification_table: UnificationTableStorage<TyVidSubKey>,
}

#[derive(Clone)]
pub(crate) struct TypeVariableData {
    origin: TypeVariableOrigin,
}

#[derive(Copy, Clone, Debug)]
pub struct TypeVariableOrigin {
    pub span: Span,
    /// この型パラメータがインスタンス化された場合の `DefId`。
    ///
    /// このフィールドは診断情報の表示時のみ使用されるべきである。
    pub param_def_id: Option<DefId>,
}

/// これらの構造体（新しい型として定義された TyVid）は、`eq_relations` の統一キーとして使用されます。
/// 各構造体は `TypeVariableValue` を保持しています。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct TyVidEqKey<'tcx> {
    vid: TyVid,

    // テーブル内では、各型変数 ID を以下のいずれかの値にマッピングします:
    phantom: PhantomData<TypeVariableValue<'tcx>>,
}

impl<'tcx> From<TyVid> for TyVidEqKey<'tcx> {
    #[inline]
    fn from(vid: TyVid) -> Self {
        TyVidEqKey { vid, phantom: PhantomData }
    }
}

impl<'tcx> UnifyKey for TyVidEqKey<'tcx> {
    type Value = TypeVariableValue<'tcx>;
    #[inline(always)]
    fn index(&self) -> u32 {
        self.vid.as_u32()
    }
    #[inline]
    fn from_index(i: u32) -> Self {
        TyVidEqKey::from(TyVid::from_u32(i))
    }
    fn tag() -> &'static str {
        "TyVidEqKey"
    }
    fn order_roots(a: Self, _: &Self::Value, b: Self, _: &Self::Value) -> Option<(Self, Self)> {
        if a.vid.as_u32() < b.vid.as_u32() { Some((a, b)) } else { Some((b, a)) }
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum TypeVariableValue<'tcx> {
    Known { value: Ty<'tcx> },
    Unknown,
}


impl<'tcx> TypeVariableValue<'tcx> {
    /// この値が既知の場合、その型を返し、未知の場合は `None` を返す。
    pub(crate) fn known(&self) -> Option<Ty<'tcx>> {
        match *self {
            TypeVariableValue::Unknown => None,
            TypeVariableValue::Known { value } => Some(value),
        }
    }

    pub(crate) fn is_unknown(&self) -> bool {
        match *self {
            TypeVariableValue::Unknown => true,
            TypeVariableValue::Known { .. } => false,
        }
    }
}


impl<'tcx> UnifyValue for TypeVariableValue<'tcx> {
    // このユニフィケーション自体は失敗しないことを示す。
    // 型の不一致エラーは、`unify_values` が呼ばれる前の、より高いレベルで
    // ハンドルされるべきである。
    type Error = NoError;

    /// 2つの型変数の「値」を統合するロジック。
    /// これは、2つの型変数グループが統合されるときに `ena` によって呼び出される。
    fn unify_values(value1: &Self, value2: &Self) -> Result<Self, NoError> {
        match (value1, value2) {
            // 両方の変数が既に具体的な型に束縛されることは、通常は発生すべきではない。なぜなら、
            // `unify(ty1, ty2)` のロジックは、変数を解決してから処理を行うため、
            // `unify(Known(t1), Known(t2))` は `unify(t1, t2)` として扱われるはず。
            // もしこのアームが実行された場合、それは型チェッカのロジックのバグを示唆する。
            (&TypeVariableValue::Known {..}, &TypeVariableValue::Known {..}) => {
                panic!("unify_values: 2つの既知の型変数を統一することはできません。")
            }

            // 片方が Known、もう片方が Unknown。
            // この場合、統合後の値は Known の方になります。
            (TypeVariableValue::Known {..}, TypeVariableValue::Unknown) => Ok(value1.clone()),
            (TypeVariableValue::Unknown, TypeVariableValue::Known {..}) => Ok(value2.clone()),

            // 両方とも Unknown。
            // 統合後の値も Unknown のままです。universe のような追加情報がないため、
            // これ以上行うことはありません。
            (TypeVariableValue::Unknown, TypeVariableValue::Unknown) => {
                Ok(TypeVariableValue::Unknown)
            }
        }
    }
}
