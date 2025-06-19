pub mod ty;

use std::collections::HashMap;

pub use ty::{Ty, TyKind};

use crate::stelaro_ast::NodeId;
use crate::stelaro_common::{LocalDefId, Span};
use crate::stelaro_sir::def::Res;

pub struct ResolverOutputs {
    pub ast_lowering: ResolverAstLowering,
}

#[derive(Debug)]
pub struct ResolverAstLowering {
    pub node_id_to_def_id: HashMap<NodeId, LocalDefId>,
    pub main_def: Option<MainDefinition>,
}

#[derive(Debug, Clone, Copy)]
pub struct MainDefinition {
    pub res: Res<NodeId>,
    pub span: Span,
}
