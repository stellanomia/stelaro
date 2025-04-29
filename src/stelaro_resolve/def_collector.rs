use crate::stelaro_ast::visit::Visitor;

use super::Resolver;

// TODO: 定義を集め、node_id_to_def_id に定義を登録しつつ、create_def() していく
struct DefCollector<'r, 'ra, 'tcx> {
    r: &'r mut Resolver<'ra, 'tcx>,
}

impl<'r, 'ra, 'tcx> Visitor<'r> for DefCollector<'r, 'ra, 'tcx> {

}