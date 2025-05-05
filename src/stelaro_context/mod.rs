use std::ops::Deref;
use std::{cell::RefCell, collections::HashMap};

use crate::stelaro_ast::NodeId;
use crate::stelaro_common::{LocalDefId, Symbol};
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

    // ASTのNodeIdから解決されたDefIdへのマップ
    pub resolution_map: RefCell<HashMap<NodeId, DefId>>,

    // DefId から実際の定義へのマップ
    pub definitions: RefCell<Definitions>,

    // 同一のTyKind<'ctx>に対して同一の参照を保持させるためのインターナー
    pub types_interner: RefCell<HashMap<TyKind<'tcx>, Ty<'tcx>>>,
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

    pub(crate) fn create_def(
        self,
        parent: LocalDefId,
        name: Option<Symbol>,
        def_kind: DefKind,
    ) -> LocalDefId {
        let data = def_kind.def_path_data(name);

        self.definitions.borrow_mut().create_def(parent, data)
    }
}
