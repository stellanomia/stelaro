use crate::stelaro_ast::NodeId;
use crate::stelaro_common::Span;

use super::ast::Path;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ty {
    pub id: NodeId,
    pub kind: TyKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TyKind {
    Path(Path),
    Infer,

    // () 型。ボトム型として機能する
    // NOTE: タプルの実装後、これを削除し
    // 空の Tuple が Unit を表すように変更する
    Unit,
    // Tuple,
    // Ref,
    // Array,
}
