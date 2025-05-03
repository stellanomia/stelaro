use std::fmt;
use std::ops::BitXorAssign;

use rustc_stable_hash::{FromStableHash, SipHasher128Hash as StableHasherHash};

/// ハッシュ値を表す専用の `u64` ラッパー型です。
///
/// この型は、ハッシュとしてのみ使用されることを明示するためのものであり、
/// 通常の整数演算や比較ではなく、ハッシュ値としての意味を持つ操作を意図しています。
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Hash64 {
    inner: u64,
}

impl Hash64 {
    pub const ZERO: Hash64 = Hash64 { inner: 0 };

    #[inline]
    pub fn new(n: u64) -> Self {
        Self { inner: n }
    }

    #[inline]
    pub fn as_u64(self) -> u64 {
        self.inner
    }

    #[inline]
    pub fn wrapping_add(self, other: Self) -> Self {
        Self { inner: self.inner.wrapping_add(other.inner) }
    }
}

impl BitXorAssign<u64> for Hash64 {
    #[inline]
    fn bitxor_assign(&mut self, rhs: u64) {
        self.inner ^= rhs;
    }
}

impl FromStableHash for Hash64 {
    type Hash = StableHasherHash;

    #[inline]
    fn from(StableHasherHash([a, _b]): Self::Hash) -> Self {
        Self { inner: a }
    }
}

impl fmt::Debug for Hash64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl fmt::LowerHex for Hash64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.inner, f)
    }
}

/// ハッシュ値を表す専用の `u128` ラッパー型です。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Hash128 {
    inner: u128,
}

// Hash128 は十分にシャッフルされたハッシュ値であることを前提としています。
// そのため、上下 64 ビットの両方をハッシュにかける理由はありません。
// また、この設計により、Hash128 を含む型を UnHash ベースのハッシュマップで使用できます。
// もし 64 ビット以上をハッシュに書き込んでしまうと、debug_assert! に引っかかる可能性があります。
impl std::hash::Hash for Hash128 {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        h.write_u64(self.truncate().as_u64());
    }
}

impl Hash128 {
    #[inline]
    pub fn new(n: u128) -> Self {
        Self { inner: n }
    }

    #[inline]
    pub fn truncate(self) -> Hash64 {
        Hash64 { inner: self.inner as u64 }
    }

    #[inline]
    pub fn wrapping_add(self, other: Self) -> Self {
        Self { inner: self.inner.wrapping_add(other.inner) }
    }

    #[inline]
    pub fn as_u128(self) -> u128 {
        self.inner
    }
}

impl FromStableHash for Hash128 {
    type Hash = StableHasherHash;

    #[inline]
    fn from(StableHasherHash([a, b]): Self::Hash) -> Self {
        Self { inner: u128::from(a) | (u128::from(b) << 64) }
    }
}

impl fmt::Debug for Hash128 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl fmt::LowerHex for Hash128 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.inner, f)
    }
}