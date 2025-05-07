use crate::stelaro_common::Ident;
use crate::stelaro_diagnostic::diag::DiagCtxtHandle;
use crate::stelaro_sir::def::Namespace;

use super::{Module, NameBinding, Resolver};


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
        
    }
}