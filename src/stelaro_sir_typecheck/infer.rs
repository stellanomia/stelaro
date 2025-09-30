use std::cell::RefCell;
use std::marker::PhantomData;

use ena::unify::{NoError, InPlaceUnificationTable, UnifyKey, UnifyValue};

use crate::stelaro_common::{DefId, IndexVec, Span};
use crate::stelaro_context::TyCtxt;
use crate::stelaro_ty::Ty;
use crate::stelaro_ty::ty::TyVid;

pub struct InferCtxt<'tcx> {
    pub tcx: TyCtxt<'tcx>,
    pub inner: RefCell<InferCtxtInner<'tcx>>,
}

#[derive(Clone)]
pub struct InferCtxtInner<'tcx> {
    type_variable_storage: TypeVariableStorage<'tcx>,
}

#[derive(Clone, Default)]
pub(crate) struct TypeVariableStorage<'tcx> {
    /// 各型変数の発生源などを記録する。
    values: IndexVec<TyVid, TypeVariableData>,

    /// 型変数の等価関係と、具体的な型への束縛を管理するUnion-Findテーブル。
    eq_relations: InPlaceUnificationTable<TyVidEqKey<'tcx>>,

    // sub_unification_table: UnificationTableStorage<TyVidSubKey>,
}

pub struct TypeVariableTable<'a, 'tcx> {
    storage: &'a mut TypeVariableStorage<'tcx>,
    // undo_log: &'a mut InferCtxtUndoLogs<'tcx>,
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
    // この unification 自体は失敗しないことを示す。
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

impl<'a, 'tcx> TypeVariableTable<'a, 'tcx> {
    pub fn new(storage: &'a mut TypeVariableStorage<'tcx>) -> Self {
        Self { storage }
    }

    /// 指定された `vid` が作成されたときの発生源(origin)を返します。
    ///
    /// この関数は、`vid` が他の型変数と統合されたかどうかを考慮しないことに注意してください。
    pub fn var_origin(&self, vid: TyVid) -> &TypeVariableOrigin {
        &self.storage.values[vid].origin
    }

    /// `a == b` であることを記録します。
    ///
    /// 事前条件: `a` と `b` はどちらもまだ解決されていない(knownではない)必要があります。
    pub(crate) fn equate(&mut self, a: TyVid, b: TyVid) {
        debug_assert!(self.probe(a).is_unknown(), "`equate`の事前条件違反: `a` ({:?}) は既に解決済みです", a);
        debug_assert!(self.probe(b).is_unknown(), "`equate`の事前条件違反: `b` ({:?}) は既に解決済みです", b);
        self.eq_relations().union(a, b);
    }

    /// `vid` を型 `ty` でインスタンス化（束縛）します。
    ///
    /// 事前条件: `vid` はまだインスタンス化されていてはなりません。
    pub(crate) fn instantiate(&mut self, vid: TyVid, ty: Ty<'tcx>) {
        let vid = self.root_var(vid);
        debug_assert!(!ty.is_ty_var(), "型変数を別の型変数でインスタンス化しようとしました: {vid:?} を {ty:?} で");
        debug_assert!(self.probe(vid).is_unknown(), "既に解決済みの型変数をインスタンス化しようとしました: {:?}", vid);
        debug_assert!(
            self.eq_relations().probe_value(vid).is_unknown(),
            "型変数 `{vid:?}` を二重にインスタンス化しようとしました: 新しい値 = {ty:?}, 古い値 = {:?}",
            self.eq_relations().probe_value(vid)
        );
        self.eq_relations().union_value(vid, TypeVariableValue::Known { value: ty });
    }

    /// 新しい型変数を生成します。
    pub fn new_var(&mut self, origin: TypeVariableOrigin) -> TyVid {
        let vid = self.storage.values.push(TypeVariableData { origin });

        let key = self.storage.eq_relations.new_key(TypeVariableValue::Unknown);
        assert_eq!(key.vid, vid, "不変条件違反: eq_relationsとvaluesは常に同期している必要があります");

        vid
    }

    /// `eq_relations` 等価テーブルにおける `vid` のルート変数を返します。
    ///
    /// このテーブルが管理する関係は同値関係であり、`root_var(a) == root_var(b)` は
    /// `a` と `b` が同じ同値クラスに属することを意味します。この関係は推移律を
    /// 満たすため、間接的な等価関係も正しく判定できます。
    pub(crate) fn root_var(&mut self, vid: TyVid) -> TyVid {
        self.eq_relations().find(vid).vid
    }

    /// これまでに作成された型変数の総数を返します。
    pub(crate) fn num_vars(&self) -> usize {
        self.storage.values.len()
    }

    /// `vid` がインスタンス化されている場合、その値(`TypeVariableValue`)を取得します。
    pub(crate) fn probe(&mut self, vid: TyVid) -> TypeVariableValue<'tcx> {
        self.inlined_probe(vid)
    }

    /// `eq_relations` テーブルへの可変参照を返します。
    #[inline]
    pub fn eq_relations(&mut self) -> &mut InPlaceUnificationTable<TyVidEqKey<'tcx>> {
        &mut self.storage.eq_relations
    }

    /// `probe` の常にインライン化されるバージョン。ホットな呼び出し元で使用されます。
    #[inline(always)]
    pub(crate) fn inlined_probe(&mut self, vid: TyVid) -> TypeVariableValue<'tcx> {
        self.eq_relations().inlined_probe_value(vid)
    }

    /// まだ解決されていない（Unknown状態の）すべての型変数のリストを返します。
    /// 型チェックの最後に呼び出し、もしリストが空でなければ型エラーを報告するために使います。
    pub(crate) fn unresolved_variables(&mut self) -> Vec<TyVid> {
        (0..self.num_vars())
            .filter_map(|i| {
                let vid = TyVid::from_usize(i);
                match self.probe(vid) {
                    TypeVariableValue::Unknown => Some(vid),
                    TypeVariableValue::Known { .. } => None,
                }
            })
            .collect()
    }
}
