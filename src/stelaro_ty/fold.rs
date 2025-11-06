use std::convert::Infallible;

use crate::{
    stelaro_context::TyCtxt,
    stelaro_diagnostics::ErrorEmitted,
    stelaro_ty::{
        Ty, TyKind,
        visit::TypeVisitable,
    },
};

/// このトレイトは、フォールド可能なすべての型に実装され、
/// 走査の骨格を提供します。
///
/// このトレイトは `TypeVisitable` のサブトレイトです。これは、多くの `TypeFolder` が
/// フォールディング中に `TypeVisitableExt` のヘルパーメソッド（例: `has_infer_ty`）を
/// 使って、不要な走査をスキップする最適化を行うためです。
/// そのため、実際にはほとんどすべてのフォールド可能な型は、同時に訪問可能（visitable）
/// である必要があります。
pub trait TypeFoldable<'tcx>: TypeVisitable<'tcx> + Clone {
    /// 失敗する可能性のあるフォールディングのエントリーポイントです。
    /// フォルダー`f`を使って値`t`をフォールドするには、`t.try_fold_with(f)`を呼び出します。
    ///
    /// ほとんどの型では、このメソッドは単に値を走査し、各フィールド/要素に対して`try_fold_with`を
    /// 呼び出すだけです。
    ///
    /// `Ty` のような「興味のある型」の場合、このメソッドの実装はその型専用のフォルダーメソッド
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

/// このトレイトは、「興味のある型」(e.g., `Ty`)にのみ実装されます。
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

/// 失敗する可能性のあるフォールディング走査のための、基本となるトレイト。
pub trait FallibleTypeFolder<'tcx>: Sized {
    type Error;

    fn tcx(&self) -> TyCtxt<'tcx>;

    fn try_fold_ty(&mut self, t: Ty<'tcx>) -> Result<Ty<'tcx>, Self::Error> {
        t.try_super_fold_with(self)
    }

    // 将来、Constなどの「興味のある型」が増えたら、ここに対応するメソッドを追加します。
    // fn try_fold_const(&mut self, c: Const<'tcx>) -> Result<Const<'tcx>, Self::Error> { ... }
}

/// 失敗しないフォールディング走査のためのトレイト。
///
/// 実質的には、`FallibleTypeFolder` の `Error` 型が `Infallible` であることを
/// 示すためのマーカー/ラッパートレイトです。
pub trait TypeFolder<'tcx>: FallibleTypeFolder<'tcx, Error = Infallible> {
    /// 失敗しないバージョンの `fold_ty` を提供します。
    /// `try_fold_ty` の結果を `unwrap` することで実装されますが、
    /// `Error = Infallible` のため、この `unwrap` は決してパニックしません。
    fn fold_ty(&mut self, t: Ty<'tcx>) -> Ty<'tcx> {
        self.try_fold_ty(t).unwrap()
    }
}

/// `Error = Infallible` であるすべての `FallibleTypeFolder` は、自動的に `TypeFolder` になります。
/// これにより、`TypeFolder` トレイト境界を持つ `fold_with` メソッドに渡せるようになります。
impl<'tcx, F> TypeFolder<'tcx> for F where F: FallibleTypeFolder<'tcx, Error = Infallible> {}

impl<'tcx> TypeFoldable<'tcx> for Ty<'tcx> {
    fn try_fold_with<F: FallibleTypeFolder<'tcx>>(self, folder: &mut F) -> Result<Self, F::Error> {
        folder.try_fold_ty(self)
    }

    fn fold_with<F: TypeFolder<'tcx>>(self, folder: &mut F) -> Self {
        folder.fold_ty(self)
    }
}

impl<'tcx> TypeSuperFoldable<'tcx> for Ty<'tcx> {
    fn try_super_fold_with<F: FallibleTypeFolder<'tcx>>(self, _folder: &mut F) -> Result<Self, F::Error> {
        let _ = match *self.kind() {
            // 再帰的にフォールドが必要なバリアント
            TyKind::Tuple(_) => {
                unimplemented!("Folding for Tuple is not yet implemented");
            }

            // 末端の型、あるいは内部にフォールドすべき `Ty` を持たない型。
            TyKind::Bool
            | TyKind::Char
            | TyKind::Str
            | TyKind::Int(_)
            | TyKind::Uint(_)
            | TyKind::Float(_)
            | TyKind::FnDef(_)
            | TyKind::Infer(_)
            | TyKind::Unit
            | TyKind::Never
            | TyKind::Error(_) => return Ok(self),
        };

        // もし `kind` が変更されていたら、新しい `Ty` を生成する。
        // （まだ変更されるケースはないが、将来のためにこの構造を維持する）
        // let tcx = folder.tcx();
        // Ok(tcx.mk_ty(kind))
    }

    fn super_fold_with<F: TypeFolder<'tcx>>(self, folder: &mut F) -> Self {
        match self.try_super_fold_with(folder) {
            Ok(t) => t,
            Err(e) => match e {}, // `e` is of type `Infallible`
        }
    }
}

impl<'tcx> TypeFoldable<'tcx> for ErrorEmitted {
    fn try_fold_with<F: FallibleTypeFolder<'tcx>>(self, _folder: &mut F) -> Result<Self, F::Error> {
        Ok(self)
    }
    fn fold_with<F: TypeFolder<'tcx>>(self, _folder: &mut F) -> Self {
        self
    }
}

impl<'tcx, T: TypeFoldable<'tcx>> TypeFoldable<'tcx> for Vec<T> {
    fn try_fold_with<F: FallibleTypeFolder<'tcx>>(self, folder: &mut F) -> Result<Self, F::Error> {
        self.into_iter().map(|t| t.try_fold_with(folder)).collect()
    }

    fn fold_with<F: TypeFolder<'tcx>>(self, folder: &mut F) -> Self {
        self.into_iter().map(|t| t.fold_with(folder)).collect()
    }
}

impl<'tcx, T: TypeFoldable<'tcx>> TypeFoldable<'tcx> for Option<T> {
    fn try_fold_with<F: FallibleTypeFolder<'tcx>>(self, folder: &mut F) -> Result<Self, F::Error> {
        self.map(|v| v.try_fold_with(folder)).transpose()
    }

    fn fold_with<F: TypeFolder<'tcx>>(self, folder: &mut F) -> Self {
        self.map(|v| v.fold_with(folder))
    }
}
