pub mod result;
mod expectation;
mod expr;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use crate::stelaro_common::{LocalDefId, Span};
use crate::stelaro_context::TyCtxt;
use crate::stelaro_diagnostics::DiagCtxtHandle;
use crate::stelaro_sir::sir_id::SirId;
use crate::stelaro_sir_typecheck::result::TypeckResults;
use crate::stelaro_ty::Ty;

pub struct TypeCheckCtxt<'tcx> {
    pub tcx: TyCtxt<'tcx>,

    /// 各オーナーの型チェック結果を格納するマップ。
    results_map: RefCell<HashMap<LocalDefId, TypeckResults<'tcx>>>,
}

/// 型チェック中にコードの発散（divergence）状態を追跡します。
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Diverges {
    /// 発散しない
    Maybe,
    /// 常に発散する (e.g., `return`, `break`)
    Always,
    /// 常に発散し、既に警告済み
    WarnedAlways,
}

/// `break`や`continue`が可能なスコープの情報を保持します。
#[derive(Debug, Clone, Copy)]
pub struct BreakableScope<'tcx> {
    pub loop_id: SirId,
    pub break_ty: Option<Ty<'tcx>>,
}

impl<'tcx> TypeCheckCtxt<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>) -> Self {
        Self {
            tcx,
            results_map: RefCell::new(HashMap::new()),
        }
    }

    /// 指定されたオーナーの`TypeckResults`への可変参照を取得し、存在しない場合は新しく作成する。
    pub(crate) fn results_for(&self, owner_id: LocalDefId) -> std::cell::RefMut<'_, TypeckResults<'tcx>> {
        let mut map = self.results_map.borrow_mut();
        map.entry(owner_id)
            .or_insert_with(||
                TypeckResults::new(owner_id)
        );
        std::cell::RefMut::map(map, |m| m.get_mut(&owner_id).unwrap())
    }

    /// 型チェック完了後に、すべての結果を消費して返却する
    pub fn take_results(self) -> HashMap<LocalDefId, TypeckResults<'tcx>> {
        self.results_map.into_inner()
    }
}


pub struct FnCtxt<'a, 'tcx> {
    pub tccx: &'a TypeCheckCtxt<'tcx>,
    pub owner_id: LocalDefId,

    /// この関数の期待される戻り値の型。
    pub return_ty: Ty<'tcx>,

    /// `return` が最初に現れる場所。
    pub return_span: Span,

    /// 現在のコードパスが発散しているかどうか。
    pub diverges: Cell<Diverges>,

    /// `break`/`continue`の対象となるループスコープのスタック。
    pub breakable_scopes: RefCell<Vec<BreakableScope<'tcx>>>,

    /// `loop`式から`break`で返される値の型を統一するための情報。
    pub loop_break_types: RefCell<HashMap<SirId, Ty<'tcx>>>,
}

impl<'a, 'tcx> FnCtxt<'a, 'tcx> {
    pub fn new(
        tccx: &'a TypeCheckCtxt<'tcx>,
        owner_id: LocalDefId,
        return_ty: Ty<'tcx>,
        return_span: Span,
    ) -> Self {
        Self {
            tccx,
            owner_id,
            return_ty,
            return_span,
            diverges: Cell::new(Diverges::Maybe),
            breakable_scopes: RefCell::new(Vec::new()),
            loop_break_types: RefCell::new(HashMap::new()),
        }
    }

    #[inline]
    pub fn tcx(&self) -> TyCtxt<'tcx> {
        self.tccx.tcx
    }

    #[inline]
    pub fn dcx(&self) -> DiagCtxtHandle<'_> {
        self.tccx.tcx.dcx()
    }

    pub fn record_type(&self, sir_id: SirId, ty: Ty<'tcx>) {
        if self.owner_id != sir_id.owner.def_id {
            panic!(
                "SirId owner mismatch: expected {:?}, found {:?}",
                self.owner_id, sir_id.owner.def_id
            );
        }
        let mut results = self.tccx.results_for(self.owner_id);
        results.record_type(sir_id.local_id, ty);
    }

    pub fn record_error(&self) {
        let mut results = self.tccx.results_for(self.owner_id);
        results.tainted_by_errors = true;
    }

    /// エラーを報告し、`tainted_by_errors`フラグを立てます。
    pub fn report_error(&self, message: &str, span: Span) {
        let mut diag = self.dcx().struct_err(span);
        diag.set_message(message.to_string());
        diag.emit();
        self.record_error();
    }
}
