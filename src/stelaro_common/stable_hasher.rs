//! rustc の `rustc_data_structures/stable_hasher.rs` に基づいて設計されています。


use std::{hash::{Hash, Hasher}, marker::PhantomData, mem};
use super::{Hash128, Hash64, Idx, IndexVec};

pub use rustc_stable_hash::{
    FromStableHash, SipHasher128Hash as StableHasherHash, StableSipHasher128 as StableHasher,
};


/// `HashStable<CTX>` を実装するものは、複数のコンパイルセッションにわたって
/// 安定した (一貫性のある) 方法でハッシュ化できます。
///
/// `HashStable` は通常のハッシュ関数よりもかなり厳しい要件を課すことに注意してください:
///
/// - 安定ハッシュは時々識別子として使用されます。したがって、それらは
///   対応する `PartialEq` 実装に準拠しなければなりません:
///
///     - `x == y` は `hash_stable(x) == hash_stable(y)` を意味し、かつ
///     - `x != y` は `hash_stable(x) != hash_stable(y)` を意味します。
///
///   後者の条件は、通常ハッシュ関数 (例: `Hash`) には要求されません。
///   実際には、これは `hash_stable` が `PartialEq` の比較で考慮される
///   すべての情報をハッシャーに供給しなければならないことを意味します。
///   この不変条件に違反することが rustc で過去に問題を引き起こした例については、
///   [rustc: #49300](https://github.com/rust-lang/rust/issues/49300) を参照してください。
/// - `hash_stable()` は現在のコンパイルセッションから独立していなければなりません。
///   例えば、メモリアドレスやコンパイルセッションごとに「ランダムに」割り当てられる
///   他のものをハッシュしてはいけません。
///
/// - `hash_stable()` はホストアーキテクチャから独立していなければなりません。
///   `StableHasher` はエンディアンや `isize`/`usize` のプラットフォーム差を
///   処理します。
pub trait HashStable<CTX> {
    fn hash_stable(&self, hcx: &mut CTX, hasher: &mut StableHasher);
}

/// 例えば `DefPathHash` に変換できる `DefId` のように、安定したキーに変換できる型の
/// ためにこれを実装します。これは、マップをハッシュ化する前に
/// 予測可能な順序にするために使用されます。
pub trait ToStableHashKey<HCX> {
    type KeyType: Ord + Sized + HashStable<HCX>;
    fn to_stable_hash_key(&self, hcx: &HCX) -> Self::KeyType;
}


/// 型が安定したソート順 (Ordによる比較結果) を持つことを示すためのトレイトです。
///
/// このトレイトは、型の `Ord` 実装がセッション依存の状態に影響されず、
/// コンパイルの実行ごとに一貫した結果を返すことを要求します。
///
/// このトレイトの主な用途は、比較結果がプログラムの挙動や生成物 (例: 名前付け、順序)
/// に影響する場面で、非決定性を避けるためです。
///
/// エンコード/デコードを伴う永続化やキャッシュを現在の実装では想定していないため、
/// セッションをまたぐ安定性の定義は、単に「同じ値であれば常に同じ順序で比較される」
/// という意味に限定されます。
///
/// これは次のような型に対して自明に成り立ちます:
///  - u32, String, Path などの値型
///  - セッション固有の情報に依存しない構造体
///
/// 一方、次のような型では成り立ちません:
///  - `*const T` と `*mut T`: これらのポインタの値はセッション間で変化するため。
///  - `DefIndex`, `SteloNum`, `LocalDefId`: これらの具体的な値は、
///    コンパイルセッション間で異なる可能性のある状態に依存するため。
///
/// 関連定数 `CAN_USE_UNSTABLE_SORT` は、不安定ソート (`sort_unstable`) を
/// 使用できるかどうかを示します。`a == b` が 「aとbが意味的に完全に同一」
/// であることを意味する場合に限り、`true` に設定してください。
pub trait StableOrd: Ord {
    const CAN_USE_UNSTABLE_SORT: bool;

    /// `Ord` 実装がこのトレイトの要件を満たすことを
    /// 人間が確認済みであることを示すマーカー。
    const ORD_IS_SESSION_STABLE: ();
}

impl<T: StableOrd> StableOrd for &T {
    const CAN_USE_UNSTABLE_SORT: bool = T::CAN_USE_UNSTABLE_SORT;

    /// 参照の順序は、正確に参照先の順序に従うため、
    /// 参照先の順序が安定しているならば、参照の順序もまた安定していることになる。
    const ORD_IS_SESSION_STABLE: () = ();
}


/// これは `StableOrd` に対応する補助トレイトです。`Symbol` のような一部の型は、
/// セッションをまたいで安定した方法で比較することは可能ですが、
/// その `Ord` 実装自体は安定していません。このような場合、`StableOrd` の代わりに
/// 本トレイト `StableCompare` を実装することで、軽量な安定ソート手段を提供できます。
///
/// (より重い選択肢としては `ToStableHashKey` を使ってソートする方法がありますが、
/// これは安定したハッシュコンテキストへのアクセスが必要になるうえ、
/// `Symbol` のように `String` を割り当てる必要がある場合にはコストが高くなります。)
///
/// 補足: `Symbol` はセッションごとに異なる `u32` を持つため、直接比較すると安定しません。
/// `ToStableHashKey` によって `&str` に変換することで安定性を持たせることは可能ですが、
/// その際に TLS へのアクセスや文字列のアロケーションが発生し、比較コストが高くなります。
///
/// 安定なソート順序の定義については、[StableOrd] のドキュメントを参照してください。
/// 本トレイトも同じ定義に従います。このトレイトを実装する際には注意してください。
pub trait StableCompare {
    const CAN_USE_UNSTABLE_SORT: bool;

    fn stable_cmp(&self, other: &Self) -> std::cmp::Ordering;
}

/// `StableOrd` は、その型の `Ord` 実装が安定であることを示すため、
/// `StableCompare` は `Ord` への委譲によって実装することができます。
impl<T: StableOrd> StableCompare for T {
    const CAN_USE_UNSTABLE_SORT: bool = T::CAN_USE_UNSTABLE_SORT;

    fn stable_cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cmp(other)
    }
}


/// `HashStable` を単に `Hash::hash()` を呼び出すことで実装し、
/// 同じ要件を満たすために `StableOrd` も併せて実装します。
///
/// **警告**: これは 「本当に」 `Fingerprint` の生成の際にコンテキストを必要としない型にのみ有効です。
/// 誤用しやすいため ([rustc: #96013](https://github.com/rust-lang/rust/issues/96013) を参照)、このマクロは外部に公開されず、
/// このモジュール内の限られたケースでのみ使用してください。
macro_rules! impl_hash_stable_trivial {
    ($t:ty) => {
        impl<CTX> super::stable_hasher::HashStable<CTX> for $t {
            #[inline]
            fn hash_stable(&self, _: &mut CTX, hasher: &mut super::stable_hasher::StableHasher) {
                ::std::hash::Hash::hash(self, hasher);
            }
        }

        impl super::stable_hasher::StableOrd for $t {
            const CAN_USE_UNSTABLE_SORT: bool = true;

            // `Ord::cmp` はそれらのバイト列にのみ依存します
            const ORD_IS_SESSION_STABLE: () = ();
        }
    };
}

pub(crate) use impl_hash_stable_trivial;


impl_hash_stable_trivial!(i8);
impl_hash_stable_trivial!(i16);
impl_hash_stable_trivial!(i32);
impl_hash_stable_trivial!(i64);
impl_hash_stable_trivial!(isize);

impl_hash_stable_trivial!(u8);
impl_hash_stable_trivial!(u16);
impl_hash_stable_trivial!(u32);
impl_hash_stable_trivial!(u64);
impl_hash_stable_trivial!(usize);

impl_hash_stable_trivial!(u128);
impl_hash_stable_trivial!(i128);

impl_hash_stable_trivial!(char);
impl_hash_stable_trivial!(());

impl_hash_stable_trivial!(Hash64);
impl_hash_stable_trivial!(::std::ffi::OsStr);
impl_hash_stable_trivial!(::std::path::Path);
impl_hash_stable_trivial!(::std::path::PathBuf);


// デフォルトのハッシュ関数はビットの半分しかハッシュしないため、カスタム実装が必要です。
// 安定ハッシュ化のためには、完全な128ビットハッシュをハッシュしておきたいです。
impl<CTX> HashStable<CTX> for Hash128 {
    #[inline]
    fn hash_stable(&self, _: &mut CTX, hasher: &mut StableHasher) {
        self.as_u128().hash(hasher);
    }
}

impl StableOrd for Hash128 {
    const CAN_USE_UNSTABLE_SORT: bool = true;

    // `Ord::cmp` はそれらのバイト列にのみ依存します
    const ORD_IS_SESSION_STABLE: () = ();
}

impl<CTX> HashStable<CTX> for ! {
    fn hash_stable(&self, _ctx: &mut CTX, _hasher: &mut StableHasher) {
        unreachable!()
    }
}

impl<CTX, T> HashStable<CTX> for PhantomData<T> {
    fn hash_stable(&self, _ctx: &mut CTX, _hasher: &mut StableHasher) {}
}

impl<CTX> HashStable<CTX> for f32 {
    fn hash_stable(&self, ctx: &mut CTX, hasher: &mut StableHasher) {
        let val: u32 = self.to_bits();
        val.hash_stable(ctx, hasher);
    }
}

impl<CTX> HashStable<CTX> for f64 {
    fn hash_stable(&self, ctx: &mut CTX, hasher: &mut StableHasher) {
        let val: u64 = self.to_bits();
        val.hash_stable(ctx, hasher);
    }
}


impl<T: HashStable<CTX>, CTX> HashStable<CTX> for [T] {
    default fn hash_stable(&self, ctx: &mut CTX, hasher: &mut StableHasher) {
        self.len().hash_stable(ctx, hasher);
        for item in self {
            item.hash_stable(ctx, hasher);
        }
    }
}

impl<CTX> HashStable<CTX> for [u8] {
    fn hash_stable(&self, ctx: &mut CTX, hasher: &mut StableHasher) {
        self.len().hash_stable(ctx, hasher);
        hasher.write(self);
    }
}

impl<T: HashStable<CTX>, CTX> HashStable<CTX> for Vec<T> {
    #[inline]
    fn hash_stable(&self, ctx: &mut CTX, hasher: &mut StableHasher) {
        self[..].hash_stable(ctx, hasher);
    }
}


impl<CTX> HashStable<CTX> for str {
    #[inline]
    fn hash_stable(&self, ctx: &mut CTX, hasher: &mut StableHasher) {
        self.as_bytes().hash_stable(ctx, hasher);
    }
}

impl StableOrd for &str {
    const CAN_USE_UNSTABLE_SORT: bool = true;

    // `Ord::cmp` はそれらのバイト列にのみ依存します
    const ORD_IS_SESSION_STABLE: () = ();
}

impl<CTX> HashStable<CTX> for String {
    #[inline]
    fn hash_stable(&self, hcx: &mut CTX, hasher: &mut StableHasher) {
        self[..].hash_stable(hcx, hasher);
    }
}

impl StableOrd for String {
    const CAN_USE_UNSTABLE_SORT: bool = true;

    // 文字列の比較はその内容にのみ依存します。
    const ORD_IS_SESSION_STABLE: () = ();
}

impl<HCX> ToStableHashKey<HCX> for String {
    type KeyType = String;
    #[inline]
    fn to_stable_hash_key(&self, _: &HCX) -> Self::KeyType {
        self.clone()
    }
}


impl<CTX> HashStable<CTX> for bool {
    #[inline]
    fn hash_stable(&self, ctx: &mut CTX, hasher: &mut StableHasher) {
        (if *self { 1u8 } else { 0u8 }).hash_stable(ctx, hasher);
    }
}

impl StableOrd for bool {
    const CAN_USE_UNSTABLE_SORT: bool = true;

    const ORD_IS_SESSION_STABLE: () = ();
}

impl<T, CTX> HashStable<CTX> for Option<T>
where
    T: HashStable<CTX>,
{
    #[inline]
    fn hash_stable(&self, ctx: &mut CTX, hasher: &mut StableHasher) {
        if let Some(ref value) = *self {
            1u8.hash_stable(ctx, hasher);
            value.hash_stable(ctx, hasher);
        } else {
            0u8.hash_stable(ctx, hasher);
        }
    }
}

impl<T: StableOrd> StableOrd for Option<T> {
    const CAN_USE_UNSTABLE_SORT: bool = T::CAN_USE_UNSTABLE_SORT;

    // Option<T> は比較に不安定性をもたらしません。
    const ORD_IS_SESSION_STABLE: () = ();
}

impl<T1, T2, CTX> HashStable<CTX> for Result<T1, T2>
where
    T1: HashStable<CTX>,
    T2: HashStable<CTX>,
{
    #[inline]
    fn hash_stable(&self, ctx: &mut CTX, hasher: &mut StableHasher) {
        mem::discriminant(self).hash_stable(ctx, hasher);
        match *self {
            Ok(ref x) => x.hash_stable(ctx, hasher),
            Err(ref x) => x.hash_stable(ctx, hasher),
        }
    }
}

impl<T, CTX> HashStable<CTX> for &'_ T
where
    T: HashStable<CTX> + ?Sized,
{
    #[inline]
    fn hash_stable(&self, ctx: &mut CTX, hasher: &mut StableHasher) {
        (**self).hash_stable(ctx, hasher);
    }
}

impl<T, CTX> HashStable<CTX> for ::std::mem::Discriminant<T> {
    #[inline]
    fn hash_stable(&self, _: &mut CTX, hasher: &mut StableHasher) {
        ::std::hash::Hash::hash(self, hasher);
    }
}

impl<I: Idx, T, CTX> HashStable<CTX> for IndexVec<I, T>
where
    T: HashStable<CTX>,
{
    fn hash_stable(&self, ctx: &mut CTX, hasher: &mut StableHasher) {
        self.len().hash_stable(ctx, hasher);
        for v in &self.raw {
            v.hash_stable(ctx, hasher);
        }
    }
}