use crate::stelaro_common::LocalDefId;
use crate::stelaro_sir::sir_id::ItemLocalId;
use crate::stelaro_ty::ty::Ty;
use std::collections::HashMap;

/// 型チェックの成果物を集約する構造体。
/// SIRのオーナー（関数など）ごとに1つ生成される。
#[derive(Debug, Default)]
pub struct TypeckResults<'tcx> {
    /// この結果がどのオーナーに属するかを示すID。
    pub owner_id: LocalDefId,

    /// 各SIRノード（式、パターンなど）の型を格納するマップ。
    /// これがcodegenで最も重要になる情報。
    node_types: HashMap<ItemLocalId, Ty<'tcx>>,

    /// 型チェック中にエラーが発生したかどうか。
    /// エラーがあった場合、後続のフェーズ（codegenなど）をスキップできる。
    pub tainted_by_errors: bool,
}

impl<'tcx> TypeckResults<'tcx> {
    pub fn new(owner_id: LocalDefId) -> Self {
        Self {
            owner_id,
            node_types: HashMap::new(),
            tainted_by_errors: false,
        }
    }

    /// ノードの型を記録する。
    pub fn record_type(&mut self, id: ItemLocalId, ty: Ty<'tcx>) {
        self.node_types.insert(id, ty);
    }

    /// ノードの型を取得する。
    pub fn node_type(&self, id: ItemLocalId) -> Option<Ty<'tcx>> {
        self.node_types.get(&id).copied()
    }
}