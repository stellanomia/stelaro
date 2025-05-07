use std::mem;

use crate::{stelaro_ast::{ast::*, visit::{walk_item, Visitor}, NodeId}, stelaro_common::STELO_DEF_ID};
use crate::stelaro_common::{LocalDefId, Symbol};
use crate::stelaro_sir::def::DefKind;

use super::Resolver;

pub fn collect_definitions(
    resolver: &mut Resolver<'_, '_>,
    stelo: &Stelo,
) {
    let parent_def = STELO_DEF_ID;
    let mut visitor = DefCollector { resolver, parent_def};
    visitor.visit_stelo(stelo);
}

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
    fn visit_item(&mut self, item: &'a Item) {
        let def_kind = match &item.kind {
            ItemKind::Fn(..) => DefKind::Fn,
            ItemKind::Mod(..) => DefKind::Mod,
        };

        let def_id = self.create_def(item.id, Some(item.ident.name), def_kind);

        self.with_parent(def_id, |this| {
            walk_item(this, item);
        });
    }
}