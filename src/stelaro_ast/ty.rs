use crate::stelaro_common::span::Span;

use super::ast::Path;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ty {
    pub kind: TyKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TyKind {
    Path(Path),
    Infer,

    // NOTE: タプル実装後、削除する
    Unit,
    // Tuple,
    // Ref,
    // Array,
}
