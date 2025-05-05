use std::{mem, ops::ControlFlow};

use crate::{stelaro_ast::{ast::*, visit::{walk_item, walk_param, Visitor}}, stelaro_common::{LocalDefId, Span, Symbol}, stelaro_sir::def::DefKind};

use super::Resolver;

// TODO: 定義を集め、node_id_to_def_id に定義を登録しつつ、create_def() していく
struct DefCollector<'a, 'ra, 'tcx> {
    resolver: &'a mut Resolver<'ra, 'tcx>,
    parent_def: LocalDefId,
}

impl<'a, 'ra, 'tcx> DefCollector<'a, 'ra, 'tcx> {
    fn create_def(
        &mut self,
        node_id: NodeId,
        name: Option<Symbol>,
        def_kind: DefKind,
    ) -> LocalDefId {
        let parent_def = self.parent_def;
        self.resolver
            .create_def(
                parent_def,
                node_id,
                name,
                def_kind,
            )
    }

    fn with_parent<F: FnOnce(&mut Self)>(&mut self, parent_def: LocalDefId, f: F) {
        let orig_parent_def = mem::replace(&mut self.parent_def, parent_def);
        f(self);
        self.parent_def = orig_parent_def;
    }
}

impl<'a, 'ra, 'tcx> Visitor<'a> for DefCollector<'a, 'ra, 'tcx> {
    fn visit_item(&mut self, item: &'a Item) -> ControlFlow<Self::BreakTy> {
        let def_kind = match &item.kind {
            ItemKind::Fn(..) => DefKind::Fn,
            ItemKind::Mod(..) => DefKind::Mod,
        };

        let _ = self.create_def(item.id, Some(item.ident.name), def_kind);

        walk_item(self, item)
    }

    fn visit_fn_decl(&mut self, f: &'a Function) -> ControlFlow<Self::BreakTy> {
        let FnSig { params, ret_ty, span } = &f.sig;

        for param in params {
            self.visit_param(param)?;
        }

        todo!()
    }

    fn visit_param(&mut self, p: &'a Param) -> std::ops::ControlFlow<!> {
        walk_param(self, p)
    }
}