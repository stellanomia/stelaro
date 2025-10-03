//! rustc の `rustc_index_macros/lib.rs` に基づいて設計されています。

mod newtype;

use proc_macro::TokenStream;

/// `IndexVec` などでインデックスとして使用できる構造体型 `S` を作成します。
///
/// これらのインデックスを操作するには2つの方法があります：
///
/// - `From` の実装が推奨される方法です。そのため、`usize` や `u32` を使って
///   `S::from(v)` とすることができます。また、`u32::from(s)` で整数に
///   変換し直すことも可能です。
///
/// - 別の方法として、メソッド `S::new(v)` で値を作成し、`s.index()` で
///   値を取得することもできます。
///
/// 内部的に、インデックスは u32 を使用するため、インデックスは
/// `u32::MAX` を超えてはなりません。
///
/// デフォルトで提供される実装は `Clone`、`Copy`、`PartialEq`、`Eq`、`Hash` です。
///
/// カスタマイズ用に受け付けられる属性：
/// - `#[derive(HashStable_Generic)]`/`#[derive(HashStable)]`: 通常通り `HashStable` を導出します。
/// - `#[orderable]`: `PartialOrd`/`Ord` に加えて、ステップ関連のメソッドを導出します。
/// - `#[debug_format = "Foo({})"]`: 特定の出力形式で `Debug` を導出します。
/// - `#[max = 0xFFFF_FFFD]`: 最大値を指定し、これによりニッチ最適化が可能になります。
///   デフォルトの最大値は 0xFFFF_FF00 です。
#[proc_macro]
pub fn newtype_index(input: TokenStream) -> TokenStream {
    newtype::newtype(input)
}
