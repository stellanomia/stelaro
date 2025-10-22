use std::fmt;

use crate::stelaro_common::VisitorResult;
use crate::stelaro_diagnostics::ErrorEmitted;
use crate::stelaro_ty::Ty;

/// このトレイトは、訪問可能なすべての型に実装され、
/// トラバーサルの骨格を提供します。
pub trait TypeVisitable<'tcx>: fmt::Debug {
    /// トラバーサルを開始するためのエントリーポイントです。ビジター `v` を使って値 `t` を巡回するには、
    /// `t.visit_with(v)` を呼び出します。
    ///
    /// 複合型 (構造体やenumなど) では、内部の各フィールドに対して再帰的にこのメソッドを呼び出し、巡回を継続します。
    ///
    /// 一方で、`Ty` のような巡回の基本単位となる型に到達すると、`visitor.visit_ty(self)` のように
    /// `TypeVisitor` 側のメソッドを呼び出し、その型の値に対する具体的な `TypeVisitor` による処理が実行されます。
    fn visit_with<V: TypeVisitor<'tcx>>(&self, visitor: &mut V) -> V::Result;
}

// このトレイトは、巡回の対象となる型に対して実装されます。
pub trait TypeSuperVisitable<'tcx>: TypeVisitable<'tcx> {
    /// 対象となる再帰的な型に対して、デフォルトの巡回処理を提供します。
    ///
    /// このメソッドは `TypeVisitor` のメソッド内でのみ呼び出すべきです。
    /// 具体的には、そのメソッドに渡された対象の型の値に対して、独自の巡回ではない
    /// デフォルトの巡回を行いたい場合に使用します。
    ///
    /// 例えば、`MyVisitor::visit_ty(ty)` の中では `ty.super_visit_with(self)` を
    /// 呼び出すのは有効ですが、それ以外の (フィールドなどに対する) 巡回は
    /// `xyz.visit_with(self)` で行うべきです。
    fn super_visit_with<V: TypeVisitor<'tcx>>(&self, visitor: &mut V) -> V::Result;
}

/// このトレイトは、あらゆる巡回処理に対して実装されます。
/// 対象となる型ごとに visit メソッドが定義されており、それぞれのメソッドは
/// デフォルトでその型のフィールドを標準的な方法で再帰的に巡回します。
pub trait TypeVisitor<'tcx>: Sized {
    type Result: VisitorResult = ();

    fn visit_ty(&mut self, t: Ty<'tcx>) -> Self::Result {
        t.super_visit_with(self)
    }

    fn visit_error(&mut self, _guar: ErrorEmitted) -> Self::Result {
        Self::Result::output()
    }
}

impl<'tcx> TypeSuperVisitable<'tcx> for Ty<'tcx> {
    fn super_visit_with<V: TypeVisitor<'tcx>>(&self, visitor: &mut V) -> V::Result {
        use super::TyKind;

        match self.kind() {
            TyKind::Tuple(_) => unimplemented!(),
            TyKind::Error(error_emitted) => error_emitted.visit_with(visitor),

            TyKind::Bool |
            TyKind::Char |
            TyKind::Int(_) |
            TyKind::Uint(_) |
            TyKind::Float(_) |
            TyKind::Infer(_) |
            TyKind::FnDef(_) |
            TyKind::Never |
            TyKind::Unit |
            TyKind::Str => V::Result::output(),
        }
    }
}

impl<'tcx> TypeVisitable<'tcx> for Ty<'tcx> {
    fn visit_with<V: TypeVisitor<'tcx>>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_ty(*self)
    }
}

impl<'tcx> TypeVisitable<'tcx> for ErrorEmitted {
    fn visit_with<V: TypeVisitor<'tcx>>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_error(*self)
    }
}
