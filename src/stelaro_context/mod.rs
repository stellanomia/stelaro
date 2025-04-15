use std::ops::Deref;
use std::{cell::RefCell, collections::HashMap};

use crate::stelaro_ast::ast::NodeId;
use crate::stelaro_common::{Arena, def_id::DefId};
use crate::stelaro_ty::{Definition, Ty, TyKind};

use super::Session;

pub struct TyCtxt<'tcx> {
    gcx: &'tcx GlobalCtxt<'tcx>,
}

pub struct GlobalCtxt<'tcx> {
    arena: &'tcx Arena,
    sess: &'tcx Session,

    // ASTのNodeIdから解決されたDefIdへのマップ
    pub resolution_map: RefCell<HashMap<NodeId, DefId>>,

    // DefId から実際の定義へのマップ
    pub definitions: RefCell<HashMap<DefId, &'tcx Definition<'tcx>>>,

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
}