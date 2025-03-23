use crate::stelaro_common::{span::Span, symbol::Ident};

use super::ast::NodeId;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ty {
    kind: TyKind,
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum TyKind {
    Path(Path),
    Infer,
    // Tuple,
    // Ref,
    // Array,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Path {
    pub span: Span,
    pub segments: Vec<PathSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PathSegment {
    pub ident: Ident,
    pub id: NodeId,
}
