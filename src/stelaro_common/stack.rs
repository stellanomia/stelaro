//! rustc の `rustc_data_structures/stack.rs` に基づいて設計されています。
//!
//! 深い再帰を伴う操作でスタックオーバーフローを防ぐために、
//! 明示的にスタックの増加を促すための関数です。


// スタック拡張前に、確保しておくべきスタック領域(バイト数)。
// `ensure_sufficient_stack` を呼ばないコードが必要とするスタックサイズ以上である必要があります。
const RED_ZONE: usize = 100 * 1024; // 100KB

// 最初のスタック拡張だけが指数関数的に増加します(2^n * STACK_PER_RECURSION)。
// 高すぎる値を設定する必要はなく、性能に影響します。
const STACK_PER_RECURSION: usize = 1024 * 1024; // 1MB

/// スタックオーバーフローを防ぐために、必要に応じてスタックを拡張します。
///
/// 再帰が深くなる可能性がある処理 (たとえば `visit_expr`) において、
/// 処理の入口付近でこの関数を呼び出すことで安全性を向上させます。
///
/// ただし、呼び出しにはわずかなオーバーヘッドがあるため、
/// 頻繁に呼ぶべきではありません。
#[inline]
pub fn ensure_sufficient_stack<R>(f: impl FnOnce() -> R) -> R {
    stacker::maybe_grow(RED_ZONE, STACK_PER_RECURSION, f)
}