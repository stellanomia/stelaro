pub mod context;
pub mod sir_map;

use std::ops::Deref;
use std::{cell::RefCell, collections::HashMap};

use crate::stelaro_common::{
    Arena, DefId, IndexVec, LocalDefId, STELO_DEF_ID, Span, StableSteloId, Symbol,
};
use crate::stelaro_context::context::CommonTypes;
use crate::stelaro_diagnostics::DiagCtxtHandle;
use crate::stelaro_session::Session;
use crate::stelaro_sir::{
    def::DefKind,
    definitions::{self, Definitions},
    sir,
};
use crate::stelaro_ty::{Ty, TyKind};

#[derive(Clone, Copy)]
pub struct TyCtxt<'tcx> {
    gcx: &'tcx GlobalCtxt<'tcx>,
}

pub struct GlobalCtxt<'tcx> {
    pub arena: &'tcx Arena,
    pub sir_arena: &'tcx Arena,
    pub sess: &'tcx Session,

    pub types: CommonTypes<'tcx>,

    /// DefId から実際の定義へのマップ
    pub definitions: RefCell<Definitions>,

    /// 同一のTyKind<'ctx>に対して同一の参照を保持させるためのインターナー
    pub types_interner: RefCell<HashMap<TyKind<'tcx>, Ty<'tcx>>>,

    /// 定義がもつ `Span` への参照
    pub source_span: RefCell<IndexVec<LocalDefId, Span>>,

    /// 定義がもつ `DefKind` への参照
    pub def_kind_table: RefCell<IndexVec<LocalDefId, DefKind>>,

    /// AST Lowering 後の Stelo
    pub sir_stelo: RefCell<Option<&'tcx sir::Stelo<'tcx>>>,
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

    pub fn def_key(self, id: DefId) -> definitions::DefKey {
        if let Some(id) = id.as_local() {
            self.definitions.borrow().def_key(id)
        } else {
            // 外部ステロに対する読み込みはまだ実装されていない
            // FIXME: 外部ステロに対しても def_kind を返せるようにする
            unimplemented!()
        }
    }

    #[inline]
    pub fn opt_parent(self, id: DefId) -> Option<DefId> {
        self.def_key(id).parent.map(|index| DefId { index, ..id })
    }

    #[inline]
    pub fn parent(self, id: DefId) -> DefId {
        match self.opt_parent(id) {
            Some(id) => id,
            None => panic!("{id:?} は親を持たない"),
        }
    }

    #[inline]
    pub fn opt_local_parent(self, id: LocalDefId) -> Option<LocalDefId> {
        self.opt_parent(id.to_def_id()).map(DefId::expect_local)
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
            sir_stelo: None.into(),
            types: CommonTypes::new(todo!(), sess),
        }
    }
}
