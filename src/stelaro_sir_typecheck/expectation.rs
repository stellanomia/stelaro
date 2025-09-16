use crate::stelaro_common::Span;
use crate::stelaro_sir_typecheck::FnCtxt;
use crate::stelaro_ty::Ty;
use Expectation::*;

/// 式の型チェックを行う際、利用可能な型ヒントを `Expectation` として
/// 下方向に伝播させます。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Expectation<'tcx> {
    /// この式がどのような型を持つべきかについて、文脈からの情報が何もないことを示します。
    NoExpectation,

    /// この式は指定された型を持つべきである、という強い制約を示します。
    /// `let x: i32 = ...;` の `...` の部分のように、型注釈などから得られる
    /// トップダウンの型推論で利用されます。
    ExpectHasType(Ty<'tcx>),
}

impl<'a, 'tcx> Expectation<'tcx> {
    /// 期待型が変数である場合、1段階だけ解決します。期待型が存在しない場合や、
    /// 解決が不可能な場合（例えばまだ制約が存在しない場合）は、単に `self` を返します。
    fn resolve(self, fcx: &FnCtxt<'a, 'tcx>) -> Expectation<'tcx> {
        match self {
            NoExpectation => NoExpectation,
            ExpectHasType(t) => ExpectHasType(fcx.resolve_vars_if_possible(t)),
        }
    }

    pub(super) fn to_option(self, fcx: &FnCtxt<'a, 'tcx>) -> Option<Ty<'tcx>> {
        match self.resolve(fcx) {
            NoExpectation => None,
            ExpectHasType(ty) => Some(ty),
        }
    }

    /// `Expectation` を、プログラムの型検査を通過するために必ず満たさなければならない
    /// 強い制約に変換しようと試みます。
    pub(super) fn only_has_type(self, fcx: &FnCtxt<'a, 'tcx>) -> Option<Ty<'tcx>> {
        match self {
            ExpectHasType(ty) => Some(fcx.resolve_vars_if_possible(ty)),
            NoExpectation => None,
        }
    }

    /// `only_has_type` に似ていますが、強い制約が存在しない場合、新しい型変数を生成して返します。
    /// これは「期待される型があればそれを使い、なければこれから推論する」という型推論の
    /// 基本的なパターンです。
    pub(super) fn coercion_target_type(self, fcx: &FnCtxt<'a, 'tcx>, span: Span) -> Ty<'tcx> {
        self.only_has_type(fcx).unwrap_or_else(|| fcx.next_ty_var(span))
    }


    /// `if` や `match` の分岐を型検査する際に、`Expectation` を調整します。
    ///
    /// もし期待される型が未解決の型変数 (`?T`) の場合、期待を一時的に取り下げて
    /// `NoExpectation` にします。これにより、一方の分岐での型推論が、他方の分岐に
    /// 対して不当に早すぎる制約を課すのを防ぎます。
    ///
    /// 例えば `let x = if c { 1 } else { 2.0 };` というコードで、`x` の型が `?T` のとき、
    /// `then`節の `1` を検査する際に `ExpectHasType(?T)` を伝播させると、`1` をデフォルトの
    /// `i32` と推論し `?T` を `i32` に束縛してしまう可能性があります。期待を取り下げることで、
    /// 各分岐が独立して型を推論し (`i32` と `f64`)、その後でそれらを統一 (unify) する
    /// という正しい推論フローが可能になります。
    ///
    /// NOTE: rustc ではこの処理に `try_structurally_resolve_type` を用いて、
    /// 型エイリアスや関連型など、より複雑な型を解決します。現在の単純な型システムでは、
    /// `resolve_vars_if_possible` で型変数を解決し、それがまだ型変数であるかを
    /// チェックするだけで十分です。
    pub(super) fn adjust_for_branches(&self, fcx: &FnCtxt<'a, 'tcx>) -> Expectation<'tcx> {
        if let ExpectHasType(ety) = self {
            let ety = fcx.resolve_vars_if_possible(*ety);
            if !ety.is_ty_var() {
                // 型が具体的に定まっているか、あるいは型変数ではない場合 (e.g., struct) は、
                // その期待をそのまま使う。
                ExpectHasType(ety)
            } else {
                // 期待が未解決の型変数なので、推論の邪魔をしないよう期待をリセットする。
                NoExpectation
            }
        } else {
            NoExpectation
        }
    }
}
