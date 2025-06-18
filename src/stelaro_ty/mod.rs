pub mod ty;

pub use ty::{Ty, TyKind};

use crate::stelaro_ast::NodeId;
use crate::stelaro_sir::def::Res;
use crate::stelaro_common::Span;

pub struct ResolverOutputs {
    pub ast_lowering: ResolverAstLowering,
}

#[derive(Debug)]
pub struct ResolverAstLowering {

}

#[derive(Debug, Clone, Copy)]
pub struct MainDefinition {
    pub res: Res<NodeId>,
    pub span: Span,
}

