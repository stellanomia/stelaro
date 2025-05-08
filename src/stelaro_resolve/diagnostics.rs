use crate::stelaro_common::Ident;
use crate::stelaro_diagnostic::diag::DiagCtxtHandle;
use crate::stelaro_sir::def::Namespace;

use super::{Module, ModuleKind, NameBinding, Resolver};


impl<'ra, 'tcx> Resolver<'ra, 'tcx> {
    pub fn dcx(&self) -> DiagCtxtHandle<'tcx> {
        self.tcx.dcx()
    }

    pub(crate) fn report_conflict(
        &mut self,
        parent: Module<'_>,
        ident: Ident,
        ns: Namespace,
        new_binding: NameBinding<'ra>,
        old_binding: NameBinding<'ra>,
    ) {
        // 名前衝突が起こった、二個目の名前に対して診断する
        if old_binding.span.start > new_binding.span.start {
            // new_binding, old_binding を入れ替える
            self.report_conflict(parent, ident, ns, old_binding, new_binding);
        }

        let container = match parent.kind {
            ModuleKind::Def(kind, _, _) => kind.descr(parent.def_id()),
            ModuleKind::Block => "block",
        };


    }
}