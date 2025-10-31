use std::convert::Infallible;

use crate::{stelaro_context::TyCtxt, stelaro_ty::{Ty, visit::TypeVisitable}};

/// このトレイトは、フォールド可能なすべての型に実装され、
/// トラバーサルの骨格を提供します。
///
/// このトレイトは `TypeVisitable` のサブトレイトです。これは、多くの `TypeFolder` が
/// フォールディング中に `TypeVisitableExt` のヘルパーメソッド (e.g.,`has_infer_ty`) を
/// 使って、不要な走査をスキップする最適化を行うためです。
/// そのため、実際にはほとんどすべてのフォールド可能な型は、同時に訪問可能 (visitable)
/// である必要があります。
pub trait TypeFoldable<'tcx>: TypeVisitable<'tcx> + Clone {
    /// 失敗する可能性のあるフォールディングのエントリーポイントです。
    /// フォルダー`f`を使って値`t`をフォールドするには、`t.try_fold_with(f)`を呼び出します。
    ///
    /// ほとんどの型では、このメソッドは単に値を走査し、各フィールド/要素に対して`try_fold_with`を
    /// 呼び出すだけです。
    ///
    /// `Ty` のような関心のある型の場合、このメソッドの実装はその型専用のフォルダーメソッド
    /// （`F::try_fold_ty`など）を呼び出します。ここで制御が `TypeFoldable` から
    /// `FallibleTypeFolder` へと移ります。
    fn try_fold_with<F: FallibleTypeFolder<'tcx>>(self, folder: &mut F) -> Result<Self, F::Error>;

    /// 失敗しないフォールディングのエントリーポイントです。
    /// フォルダー`f`を使って値`t`をフォールドするには、`t.fold_with(f)`を呼び出します。
    ///
    /// `try_fold_with` と同じですが、失敗する可能性がありません。
    /// 両関数の振る舞いが同期していることを確認する必要があります。
    fn fold_with<F: TypeFolder<'tcx>>(self, folder: &mut F) -> Self;
}

/// このトレイトは、関心のある型 (e.g., `Ty`) にのみ実装されます。
pub trait TypeSuperFoldable<'tcx>: TypeFoldable<'tcx> {
    /// 対象となる再帰的な型に対して、デフォルトのフォールド処理を提供します。
    ///
    /// このメソッドは `FallibleTypeFolder` や `TypeFolder` のメソッド内において、
    /// 渡された対象の型に対して独自の走査ではなく、デフォルトの再帰的な走査を
    /// 行いたい場合にのみ呼び出すべきです。
    ///
    /// 例えば、`MyFolder::try_fold_ty(ty)`の中では`ty.try_super_fold_with(self)`を
    /// 呼び出すのは有効です。
    fn try_super_fold_with<F: FallibleTypeFolder<'tcx>>(self, folder: &mut F) -> Result<Self, F::Error>;

    /// 失敗しないフォルダーで使うための、`try_super_fold_with`の便利な代替です。
    /// `try_super_fold_with`との一貫性を保つため、このメソッドはオーバーライドしないでください。
    fn super_fold_with<F: TypeFolder<'tcx>>(self, folder: &mut F) -> Self;
}

/// このトレイトは、失敗しないすべての folding トラバーサルのために実装されます。
/// 対象となる型ごとに `fold` メソッドが定義されており、各メソッドはデフォルトで
/// 何も変更しない操作を行います。
pub trait TypeFolder<'tcx>: Sized {
    fn cx(&self) -> TyCtxt<'tcx>;

    fn fold_ty(&mut self, t: Ty<'tcx>) -> Ty<'tcx> {
        t.super_fold_with(self)
    }
}

pub trait FallibleTypeFolder<'tcx>: Sized {
    type Error;

    fn cx(&self) -> TyCtxt<'tcx>;

    fn try_fold_ty(&mut self, t: Ty<'tcx>) -> Result<Ty<'tcx>, Self::Error> {
        t.try_super_fold_with(self)
    }
}
