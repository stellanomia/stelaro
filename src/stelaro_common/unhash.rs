//! rustc の `rustc_data_structures/unhash.rs` に基づいて設計されています。

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};

pub type UnhashMap<K, V> = HashMap<K, V, BuildHasherDefault<Unhasher>>;
pub type UnhashSet<V> = HashSet<V, BuildHasherDefault<Unhasher>>;
pub type UnindexMap<K, V> = indexmap::IndexMap<K, V, BuildHasherDefault<Unhasher>>;

/// この何もしないハッシャーは、単一の `write_u64` 呼び出しのみを想定しています。
/// `Fingerprint` のように、すでにハッシュとして十分な性質を持つキーに対して使うことを意図しています。
#[derive(Default)]
pub struct Unhasher {
    value: u64,
}

impl Hasher for Unhasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.value
    }

    fn write(&mut self, _bytes: &[u8]) {
        unimplemented!("use write_u64");
    }

    #[inline]
    fn write_u64(&mut self, value: u64) {
        debug_assert_eq!(0, self.value, "Unhasher doesn't mix values!");
        self.value = value;
    }
}