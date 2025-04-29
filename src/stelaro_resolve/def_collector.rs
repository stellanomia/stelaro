use crate::stelaro_ast::{ast::Block, visit::Visitor};

use super::Resolver;


struct DefCollector<'r, 'ra, 'tcx> {
    r: &'r mut Resolver<'ra, 'tcx>,
}

impl<'r, 'ra, 'tcx> Visitor<'r> for DefCollector<'r, 'ra, 'tcx> {

}