//! rustc の `rustc_data_structures/fingerprint.rs` に基づいて設計されています。

use std::convert::{From, TryInto};
use std::hash::{Hash, Hasher};

use super::stable_hasher::impl_hash_stable_trivial;
use super::{FromStableHash, Hash64, StableHasherHash, unhash};

/// 128ビットのフィンガープリントを表す。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct Fingerprint(u64, u64);

pub trait FingerprintComponent {
    fn as_u64(&self) -> u64;
}

impl FingerprintComponent for Hash64 {
    #[inline]
    fn as_u64(&self) -> u64 {
        Hash64::as_u64(*self)
    }
}

/// `Fingerprint` を構成できる型を示すトレイト。
impl FingerprintComponent for u64 {
    #[inline]
    fn as_u64(&self) -> u64 {
        *self
    }
}

impl Fingerprint {
    pub const ZERO: Fingerprint = Fingerprint(0, 0);

    /// 2つの構成要素から新しい `Fingerprint` を作成する。
    #[inline]
    pub fn new<A, B>(a: A, b: B) -> Fingerprint
    where
        A: FingerprintComponent,
        B: FingerprintComponent,
    {
        Fingerprint(a.as_u64(), b.as_u64())
    }

    /// `Fingerprint` をより小さなハッシュ値 (`Hash64`) に変換する。
    /// `Fingerprint` の両半分が良い品質のハッシュ値であることが期待されるが、
    /// `DefPathHash` の `Fingerprint` のように `StableSteloId` 部分が同じになる場合があるため、
    /// 2つの値を組み合わせて品質の良いハッシュ値を得る。
    #[inline]
    pub fn to_smaller_hash(&self) -> Hash64 {
        Hash64::new(self.0.wrapping_mul(3).wrapping_add(self.1))
    }

    /// Fingerprint を2つの Hash64 に分割する。
    #[inline]
    pub fn split(&self) -> (Hash64, Hash64) {
        (Hash64::new(self.0), Hash64::new(self.1))
    }

    /// 2つの `Fingerprint` を結合する。
    #[inline]
    pub fn combine(self, other: Fingerprint) -> Fingerprint {
        // この実装方法については https://stackoverflow.com/a/27952689 を参照。
        Fingerprint(
            self.0.wrapping_mul(3).wrapping_add(other.0),
            self.1.wrapping_mul(3).wrapping_add(other.1),
        )
    }

    /// 2つの `Fingerprint` を順序に依存しない方法で結合する。
    /// これが必要な場面か確認する必要がある。
    #[inline]
    pub fn combine_commutative(self, other: Fingerprint) -> Fingerprint {
        let a = u128::from(self.1) << 64 | u128::from(self.0);
        let b = u128::from(other.1) << 64 | u128::from(other.0);

        let c = a.wrapping_add(b);

        Fingerprint(c as u64, (c >> 64) as u64)
    }

    pub fn to_hex(&self) -> String {
        format!("{:016x}{:016x}", self.0, self.1)
    }

    /// `Fingerprint` をリトルエンディアンのバイト配列 ([u8; 16]) に変換する。
    #[inline]
    pub fn to_le_bytes(&self) -> [u8; 16] {
        // `unsafe { mem::transmute(*k) }` と同じ機械語に最適化されることを期待。
        let mut result = [0u8; 16];

        let first_half: &mut [u8; 8] = (&mut result[0..8]).try_into().unwrap();
        *first_half = self.0.to_le_bytes();

        let second_half: &mut [u8; 8] = (&mut result[8..16]).try_into().unwrap();
        *second_half = self.1.to_le_bytes();

        result
    }

    /// リトルエンディアンのバイト配列 (`[u8; 16]`) から `Fingerprint` を作成する。
    #[inline]
    pub fn from_le_bytes(bytes: [u8; 16]) -> Fingerprint {
        Fingerprint(
            u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
            u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
        )
    }
}

impl std::fmt::Display for Fingerprint {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:x}-{:x}", self.0, self.1)
    }
}

impl Hash for Fingerprint {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_fingerprint(self);
    }
}

trait FingerprintHasher {
    fn write_fingerprint(&mut self, fingerprint: &Fingerprint);
}

impl<H: Hasher> FingerprintHasher for H {
    #[inline]
    default fn write_fingerprint(&mut self, fingerprint: &Fingerprint) {
        self.write_u64(fingerprint.0);
        self.write_u64(fingerprint.1);
    }
}

impl FingerprintHasher for unhash::Unhasher {
    #[inline]
    fn write_fingerprint(&mut self, fingerprint: &Fingerprint) {
        // フィンガープリントの両方の部分 (64ビット×2) は、どちらも良質なハッシュ値であると期待されているが、
        // `DefPathHash` に含まれるフィンガープリントのうち、`StableSteloId` の部分は同じステロに属する
        // すべての `DefPathHash` で共通になるため、それでも2つの値を結合しておく。
        // これにより、そうしたケースにおいても高品質なハッシュが得られるようにする。
        //
        // `Unhasher` は HashMap の用途でのみ使われるため、2つの値を順序に依存しない方法で
        // (つまり `x + y` のように) 結合しても問題ない。
        // この方法は、より堅牢な `Fingerprint::to_smaller_hash()` を使うよりも安価である。
        // また、HashMap の用途では `Fingerprint(x, y)` と `Fingerprint(y, x)` が
        // 同じハッシュ値になっても特に問題はない。
        self.write_u64(fingerprint.0.wrapping_add(fingerprint.1));
    }
}

impl FromStableHash for Fingerprint {
    type Hash = StableHasherHash;

    #[inline]
    fn from(StableHasherHash([a, b]): Self::Hash) -> Self {
        Fingerprint(a, b)
    }
}

impl_hash_stable_trivial!(Fingerprint);
