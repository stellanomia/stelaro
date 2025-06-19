use std::ops::Deref;
use std::{cell::RefCell, collections::HashMap};

use crate::stelaro_common::{DefId, Arena, IndexVec, LocalDefId, Span, StableSteloId, Symbol, STELO_DEF_ID};
use crate::stelaro_diagnostics::DiagCtxtHandle;
use crate::stelaro_sir::{def::DefKind, definitions::Definitions};
use crate::stelaro_ty::{Ty, TyKind};

use super::Session;

#[derive(Clone, Copy)]
pub struct TyCtxt<'tcx> {
    gcx: &'tcx GlobalCtxt<'tcx>,
}

pub struct GlobalCtxt<'tcx> {
    pub arena: &'tcx Arena,
    pub sir_arena: &'tcx Arena,
    pub sess: &'tcx Session,

    /// DefId から実際の定義へのマップ
    pub definitions: RefCell<Definitions>,

    /// 同一のTyKind<'ctx>に対して同一の参照を保持させるためのインターナー
    pub types_interner: RefCell<HashMap<TyKind<'tcx>, Ty<'tcx>>>,

    /// 定義がもつ `Span` への参照
    pub source_span: RefCell<IndexVec<LocalDefId, Span>>,

    /// 定義がもつ `DefKind` への参照
    pub def_kind_table: RefCell<IndexVec<LocalDefId, DefKind>>,

    // std, core 実装時など、複数のStelo解析の際に使われる
    // /// インターンされた [StableSteloId] のマップ
    // pub stable_stelo_ids: IndexMap<StableSteloId, SteloNum, BuildHasherDefault<Unhasher>>,
}

impl<'tcx> Deref for TyCtxt<'tcx> {
    type Target = &'tcx GlobalCtxt<'tcx>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.gcx
    }
}


impl<'tcx> TyCtxt<'tcx> {
    pub fn dcx(self) -> DiagCtxtHandle<'tcx> {
        self.sess.dcx()
    }

    pub fn create_def(
        self,
        parent: LocalDefId,
        name: Option<Symbol>,
        def_kind: DefKind,
    ) -> LocalDefId {
        let data = def_kind.def_path_data(name);

        self.definitions.borrow_mut().create_def(parent, data)
    }

    pub fn create_local_stelo_def_id(self, stelo_span: Span) -> LocalDefId {
        let key_span = self.source_span.borrow_mut().push(stelo_span);
        let key_def_kind = self.def_kind_table.borrow_mut().push(DefKind::Mod);

        assert_eq!(key_span, STELO_DEF_ID);
        assert_eq!(key_def_kind, STELO_DEF_ID);
        STELO_DEF_ID
    }

    pub fn local_def_kind(&self, local_def_id: LocalDefId) -> DefKind {
        // LocalDefId が生成されるとき、同時に DefKind は必ず登録される
        self.def_kind_table
            .borrow()
            .get(local_def_id)
            .copied()
            .expect("bug: LocalDefId とともに DefKind が登録されていない")
    }

    pub fn def_kind(&self, def_id: DefId) -> DefKind {
        if let Some(local_def_id) = def_id.as_local() {
            self.local_def_kind(local_def_id)
        } else {
            // 外部ステロに対する読み込みはまだ実装されていない
            // FIXME: 外部ステロに対しても def_kind を返せるようにする
            unimplemented!()
        }
    }
}


impl<'tcx> TyCtxt<'tcx> {
    pub fn new(gcx: &'tcx GlobalCtxt<'tcx>) -> Self {
        Self { gcx }
    }

    pub fn create_global_ctxt(
        sess: &'tcx Session,
        stable_stelo_id: StableSteloId,
        arena: &'tcx Arena,
        sir_arena: &'tcx Arena,
    ) -> GlobalCtxt<'tcx> {
        GlobalCtxt {
            arena,
            sir_arena,
            sess,
            definitions: RefCell::new(Definitions::new(stable_stelo_id)),
            types_interner: RefCell::new(HashMap::new()),
            source_span: RefCell::new(IndexVec::new()),
            def_kind_table: RefCell::new(IndexVec::new()),
        }
    }
}
