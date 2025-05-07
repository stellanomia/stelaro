use std::ops::Deref;
use std::{cell::RefCell, collections::HashMap};

use crate::stelaro_ast::NodeId;
use crate::stelaro_common::{IndexVec, LocalDefId, Span, Symbol, STELO_DEF_ID};
use crate::stelaro_common::{Arena, def_id::DefId};
use crate::stelaro_sir::def::DefKind;
use crate::stelaro_sir::definitions::Definitions;
use crate::stelaro_ty::{Ty, TyKind};

use super::Session;

#[derive(Clone, Copy)]
pub struct TyCtxt<'tcx> {
    gcx: &'tcx GlobalCtxt<'tcx>,
}

pub struct GlobalCtxt<'tcx> {
    arena: &'tcx Arena,
    sess: &'tcx Session,

    /// ASTのNodeIdから解決されたDefIdへのマップ
    pub resolution_map: RefCell<HashMap<NodeId, DefId>>,

    /// DefId から実際の定義へのマップ
    pub definitions: RefCell<Definitions>,

    /// 同一のTyKind<'ctx>に対して同一の参照を保持させるためのインターナー
    pub types_interner: RefCell<HashMap<TyKind<'tcx>, Ty<'tcx>>>,

    /// 定義がもつ `Span` への参照
    pub source_span: RefCell<IndexVec<LocalDefId, Span>>,

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
    #[inline(always)]
    fn sess(&self) -> &Session {
        self.sess
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
        let key = self.source_span.borrow_mut().push(stelo_span);
        assert_eq!(key, STELO_DEF_ID);
        key
    }

    pub fn def_kind(&self, def_id: DefId) -> DefKind {
        todo!()
    }
}
