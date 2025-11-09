//! rustc の `rustc_type_ir/data_structures/delayed_map.rs` に基づいて設計されています。

use std::{collections::{HashMap, HashSet}, hash::Hash};

const CACHE_CUTOFF: u32 = 32;

/// 最初の数要素を無視した後にのみハッシュ計算を開始するハッシュマップ。
///
/// この構造体は主に type folder で使用され、ほとんどの場合キャッシュは有効ではありません。
/// なぜなら、folded types のほぼすべては非常に小さいためです。ただし、極めて稀に、
/// キャッシュしないとシステムがハングアップするような非常に巨大な型が存在する場合があります。
#[derive(Debug)]
pub struct DelayedMap<K, V> {
    cache: HashMap<K, V>,
    count: u32,
}

impl<K, V> Default for DelayedMap<K, V> {
    fn default() -> Self {
        DelayedMap { cache: Default::default(), count: 0 }
    }
}

impl<K: Hash + Eq, V> DelayedMap<K, V> {
    #[inline(always)]
    pub fn insert(&mut self, key: K, value: V) -> bool {
        if self.count >= CACHE_CUTOFF {
            self.cold_insert(key, value)
        } else {
            self.count += 1;
            true
        }
    }

    #[cold]
    #[inline(never)]
    fn cold_insert(&mut self, key: K, value: V) -> bool {
        self.cache.insert(key, value).is_none()
    }

    #[inline(always)]
    pub fn get(&self, key: &K) -> Option<&V> {
        if self.cache.is_empty() { None } else { self.cold_get(key) }
    }

    #[cold]
    #[inline(never)]
    fn cold_get(&self, key: &K) -> Option<&V> {
        self.cache.get(key)
    }
}

#[derive(Debug)]
pub struct DelayedSet<T> {
    cache: HashSet<T>,
    count: u32,
}

impl<T> Default for DelayedSet<T> {
    fn default() -> Self {
        DelayedSet { cache: Default::default(), count: 0 }
    }
}

impl<T: Hash + Eq> DelayedSet<T> {
    #[inline(always)]
    pub fn insert(&mut self, value: T) -> bool {
        if self.count >= CACHE_CUTOFF {
            self.cold_insert(value)
        } else {
            self.count += 1;
            true
        }
    }

    #[cold]
    #[inline(never)]
    fn cold_insert(&mut self, value: T) -> bool {
        self.cache.insert(value)
    }

    #[inline(always)]
    pub fn contains(&self, value: &T) -> bool {
        !self.cache.is_empty() && self.cold_contains(value)
    }

    #[cold]
    #[inline(never)]
    fn cold_contains(&self, value: &T) -> bool {
        self.cache.contains(value)
    }
}
