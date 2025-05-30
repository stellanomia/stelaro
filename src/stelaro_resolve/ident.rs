use crate::stelaro_sir::def::{Namespace, PerNS};

use super::{late::{Scope, Segment}, Module, NameBinding, Resolver};


impl<'ra, 'tcx> Resolver<'ra, 'tcx> {
    pub fn resolve_path_with_scopes(
        &mut self,
        path: &[Segment],
        opt_ns: Option<Namespace>,
        parent_scope: &Module<'ra>,
        scopes: Option<&PerNS<Vec<Scope<'ra>>>>,
        ignore_binding: Option<NameBinding<'ra>>,
    ) {

    }
}