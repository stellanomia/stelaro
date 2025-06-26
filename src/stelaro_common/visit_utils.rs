use std::ops::ControlFlow;

#[macro_export]
macro_rules! try_visit {
    ($e:expr) => {
        match $crate::stelaro_common::VisitorResult::branch($e) {
            std::ops::ControlFlow::Continue(()) => (),
            #[allow(unreachable_code)]
            std::ops::ControlFlow::Break(r) => {
                return $crate::stelaro_common::VisitorResult::from_residual(r);
            }
        }
    };
}

#[macro_export]
macro_rules! walk_list {
    ($visitor: expr, $method: ident, $list: expr $(, $($extra_args: expr),+ )?) => {
        for elem in $list {
            try_visit!($visitor.$method(elem $(, $($extra_args),*)? ));
        }
    };
}

#[macro_export]
macro_rules! visit_opt {
    ($visitor: expr, $method: ident, $opt: expr $(, $($extra_args: expr),+ )?) => {
        if let Some(value) = $opt {
            try_visit!($visitor.$method(value $(, $($extra_args),*)? ));
        }
    };
}

/// Visitor パターンの走査結果を抽象化するトレイト。
///
/// このトレイトは、走査が中断せずに完了した (`Continue`) か、
/// 途中で中断された (`Break`) かを統一的に扱うための抽象です。
/// `Visitor::Result` のトレイト境界として使用され、主に `()` (中断しない) と
/// `ControlFlow<T>` (中断する可能性がある) の2つの型に対して実装されます。
pub trait VisitorResult {
    /// 走査が中断された場合に返される値の型。
    type Residual;

    /// 走査が中断せずに完了した場合に返されるデフォルトの「継続」を示す値。
    fn output() -> Self;

    /// 中断を示す `Residual` の値から `VisitorResult` 型の値を生成します。
    fn from_residual(residual: Self::Residual) -> Self;

    /// `ControlFlow<Self::Residual>` (中断の可能性を含むフロー) から `VisitorResult` 型の値を生成します。
    fn from_branch(b: ControlFlow<Self::Residual>) -> Self;

    /// `VisitorResult` の値から、中断 (`Break`) または継続 (`Continue`) を示す
    /// `ControlFlow<Self::Residual>` を得ます。
    fn branch(self) -> ControlFlow<Self::Residual>;
}

/// `Visitor` による走査が中断しないときの型。
impl VisitorResult for () {
    /// AST の走査を中断しないとき、`!` (never type) により
    /// 処理の中断がデフォルトでは発生しないことが型レベルで保証される。
    type Residual = !;

    fn output() -> Self {}
    fn from_residual(redidual: Self::Residual) -> Self { match redidual {} }
    fn from_branch(b: ControlFlow<Self::Residual>) -> Self {
        match b {
            ControlFlow::Continue(c) => c,
            ControlFlow::Break(residual) => match residual {},
        }
    }
    fn branch(self) -> ControlFlow<Self::Residual> {
        ControlFlow::Continue(())
    }
}

/// `Visitor` による走査が中断する可能性がある実装に使用されます。
impl<T> VisitorResult for ControlFlow<T> {
    type Residual = T;

    fn output() -> Self {
        ControlFlow::Continue(())
    }

    fn from_residual(residual: Self::Residual) -> Self {
        ControlFlow::Break(residual)
    }

    fn from_branch(b: Self) -> Self {
        b
    }

    fn branch(self) -> Self {
        self
    }
}