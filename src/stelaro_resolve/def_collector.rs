use super::Resolver;


struct DefCollector<'r, 'ra, 'tcx> {
    resolver: &'r mut Resolver<'ra, 'tcx>,
}
