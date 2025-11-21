use std::fmt;
use std::ops::ControlFlow;
use std::sync::Arc;

use crate::stelaro_common::{Idx, IndexVec, VisitorResult};
use crate::stelaro_diagnostics::ErrorEmitted;
use crate::stelaro_ty::{Ty, TyKind};
use crate::{try_visit, walk_visitable_list};

use bitflags::bitflags;

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

impl<'tcx, T: TypeVisitable<'tcx>, U: TypeVisitable<'tcx>> TypeVisitable<'tcx> for (T, U) {
    fn visit_with<V: TypeVisitor<'tcx>>(&self, visitor: &mut V) -> V::Result {
        try_visit!(self.0.visit_with(visitor));
        self.1.visit_with(visitor)
    }
}

impl<'tcx, A: TypeVisitable<'tcx>, B: TypeVisitable<'tcx>, C: TypeVisitable<'tcx>> TypeVisitable<'tcx>
    for (A, B, C)
{
    fn visit_with<V: TypeVisitor<'tcx>>(&self, visitor: &mut V) -> V::Result {
        try_visit!(self.0.visit_with(visitor));
        try_visit!(self.1.visit_with(visitor));
        self.2.visit_with(visitor)
    }
}

impl<'tcx, T: TypeVisitable<'tcx>> TypeVisitable<'tcx> for Option<T> {
    fn visit_with<V: TypeVisitor<'tcx>>(&self, visitor: &mut V) -> V::Result {
        match self {
            Some(v) => v.visit_with(visitor),
            None => V::Result::output(),
        }
    }
}

impl<'tcx, T: TypeVisitable<'tcx>, E: TypeVisitable<'tcx>> TypeVisitable<'tcx> for Result<T, E> {
    fn visit_with<V: TypeVisitor<'tcx>>(&self, visitor: &mut V) -> V::Result {
        match self {
            Ok(v) => v.visit_with(visitor),
            Err(e) => e.visit_with(visitor),
        }
    }
}

impl<'tcx, T: TypeVisitable<'tcx>> TypeVisitable<'tcx> for Arc<T> {
    fn visit_with<V: TypeVisitor<'tcx>>(&self, visitor: &mut V) -> V::Result {
        (**self).visit_with(visitor)
    }
}

impl<'tcx, T: TypeVisitable<'tcx>> TypeVisitable<'tcx> for Box<T> {
    fn visit_with<V: TypeVisitor<'tcx>>(&self, visitor: &mut V) -> V::Result {
        (**self).visit_with(visitor)
    }
}

impl<'tcx, T: TypeVisitable<'tcx>> TypeVisitable<'tcx> for Vec<T> {
    fn visit_with<V: TypeVisitor<'tcx>>(&self, visitor: &mut V) -> V::Result {
        walk_visitable_list!(visitor, self.iter());
        V::Result::output()
    }
}

// `TypeFoldable` は `&[T]` に対しては実装されていません。
// 一般的なケースでは新しいスライスを返すことができないため、これは意味をなしません。
// ただし、他の場所には特定のスライス型に対する `TypeFoldable` の自明な実装がいくつか存在します。
impl<'tcx, T: TypeVisitable<'tcx>> TypeVisitable<'tcx> for &[T] {
    fn visit_with<V: TypeVisitor<'tcx>>(&self, visitor: &mut V) -> V::Result {
        walk_visitable_list!(visitor, self.iter());
        V::Result::output()
    }
}

impl<'tcx, T: TypeVisitable<'tcx>> TypeVisitable<'tcx> for Box<[T]> {
    fn visit_with<V: TypeVisitor<'tcx>>(&self, visitor: &mut V) -> V::Result {
        walk_visitable_list!(visitor, self.iter());
        V::Result::output()
    }
}

impl<'tcx, T: TypeVisitable<'tcx>, Ix: Idx> TypeVisitable<'tcx> for IndexVec<Ix, T> {
    fn visit_with<V: TypeVisitor<'tcx>>(&self, visitor: &mut V) -> V::Result {
        walk_visitable_list!(visitor, self.iter());
        V::Result::output()
    }
}

impl<'tcx, T: TypeVisitable<'tcx>, S> TypeVisitable<'tcx> for indexmap::IndexSet<T, S> {
    fn visit_with<V: TypeVisitor<'tcx>>(&self, visitor: &mut V) -> V::Result {
        walk_visitable_list!(visitor, self.iter());
        V::Result::output()
    }
}


bitflags! {
    /// 型が内部にどのような要素を含んでいるかを要約するフラグ。
    /// これにより、高コストな再帰的走査を避け、高速なチェックが可能になる。
    #[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
    pub struct TypeFlags: u8 {
        /// この型に `TyKind::Infer` が含まれている。
        const HAS_TY_INFER = 1 << 0;
        /// この型に `TyKind::Error` が含まれている。
        const HAS_ERROR    = 1 << 1;
    }
}

#[derive(Debug)]
pub struct FlagComputation {
    pub flags: TypeFlags,
}

impl<'tcx> FlagComputation {
    fn new() -> FlagComputation {
        FlagComputation { flags: TypeFlags::empty() }
    }

    fn add_flags(&mut self, flags: TypeFlags) {
        self.flags |= flags;
    }

    pub fn for_kind(kind: &TyKind<'tcx>) -> FlagComputation {
        let mut result = FlagComputation::new();
        result.add_kind(kind);
        result
    }

    fn add_kind(&mut self, kind: &TyKind<'tcx>) {
        match *kind {
            TyKind::Bool
            | TyKind::Char
            | TyKind::Str
            | TyKind::Int(_)
            | TyKind::Uint(_)
            | TyKind::Float(_)
            | TyKind::FnDef(_)
            | TyKind::Unit
            | TyKind::Never => {}

            TyKind::Infer(_) => self.add_flags(TypeFlags::HAS_TY_INFER),
            TyKind::Error(_) => self.add_flags(TypeFlags::HAS_ERROR),

            TyKind::Tuple(_) => unreachable!(),
        }
    }
}

pub trait Flags {
    fn flags(&self) -> TypeFlags;
}

pub trait TypeVisitableExt<'tcx>: TypeVisitable<'tcx> {
    fn has_type_flags(&self, flags: TypeFlags) -> bool;

    fn references_error(&self) -> bool {
        self.has_type_flags(TypeFlags::HAS_ERROR)
    }

    fn error_reported(&self) -> Result<(), ErrorEmitted>;

    fn has_infer_types(&self) -> bool {
        self.has_type_flags(TypeFlags::HAS_TY_INFER)
    }
}

impl<'tcx, T: TypeVisitable<'tcx>> TypeVisitableExt<'tcx> for T {
    fn has_type_flags(&self, flags: TypeFlags) -> bool {
        let res =
            self.visit_with(&mut HasTypeFlagsVisitor { flags }) == ControlFlow::Break(FoundFlags);
        res
    }

    fn error_reported(&self) -> Result<(), ErrorEmitted> {
        if self.references_error() {
            if let ControlFlow::Break(guar) = self.visit_with(&mut HasErrorVisitor) {
                Err(guar)
            } else {
                panic!("type flags said there was an error, but now there is not")
            }
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct FoundFlags;

struct HasTypeFlagsVisitor {
    flags: TypeFlags,
}

impl std::fmt::Debug for HasTypeFlagsVisitor {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.flags.fmt(fmt)
    }
}

impl<'tcx> TypeVisitor<'tcx> for HasTypeFlagsVisitor {
    type Result = ControlFlow<FoundFlags>;

    /// `visit_ty` は、型に遭遇するたびに呼び出されます。
    /// これがこのビジターの主要なロジックです。
    #[inline]
    fn visit_ty(&mut self, t: Ty<'tcx>) -> Self::Result {
        // `Ty` が持つ計算済みの `flags` と、探している `flags` を比較します。
        if t.flags().intersects(self.flags) {
            // もし共通のフラグがあれば、目的は達成された
            ControlFlow::Break(FoundFlags)
        } else {
            ControlFlow::Continue(())
        }
    }

    /// `visit_error` は、エラー型に遭遇したときに呼び出されます。
    #[inline]
    fn visit_error(&mut self, _guar: ErrorEmitted) -> Self::Result {
        // もし探しているフラグに `HAS_ERROR` が含まれていれば、
        // エラーが見つかったので走査を終了させます。
        if self.flags.contains(TypeFlags::HAS_ERROR) {
            ControlFlow::Break(FoundFlags)
        } else {
            // `HAS_ERROR` を探していない場合は、走査を続行します。
            ControlFlow::Continue(())
        }
    }
}

struct HasErrorVisitor;

impl<'tcx> TypeVisitor<'tcx> for HasErrorVisitor {
    type Result = ControlFlow<ErrorEmitted>;

    fn visit_error(&mut self, guar: ErrorEmitted) -> Self::Result {
        ControlFlow::Break(guar)
    }
}
