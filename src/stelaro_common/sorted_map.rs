//! rustc の `rustc_data_structures/sorted_map.rs` に基づいて設計されています。

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::mem;
use std::ops::{Bound, Index, IndexMut, RangeBounds};

/// `SortedMap`は`BTreeMap`と似た特性を持つデータ構造ですが、トレードオフが若干異なります。
/// ルックアップは *O*(log(*n*))、挿入と削除は *O*(*n*) ですが、要素を順序通りに安価にイテレートできます。
///
/// `SortedMap`は、データをよりコンパクトに格納するため、小さいサイズ (<50) の場合`BTreeMap`よりも
/// 高速になることがあります。また、連続した範囲の要素にスライスとしてアクセスすることや、
/// すでにソート済みの要素のスライスを効率的に挿入することもサポートしています。
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SortedMap<K, V> {
    data: Vec<(K, V)>,
}

impl<K, V> Default for SortedMap<K, V> {
    #[inline]
    fn default() -> SortedMap<K, V> {
        SortedMap { data: Vec::new() }
    }
}

impl<K, V> SortedMap<K, V> {
    #[inline]
    pub const fn new() -> SortedMap<K, V> {
        SortedMap { data: Vec::new() }
    }
}

impl<K: Ord, V> SortedMap<K, V> {
    /// 事前にソートされた要素のセットから`SortedMap`を構築します。これは、空のマップを
    /// 作成してから要素を個別に挿入するよりも高速です。
    ///
    /// 要素がキーでソートされており、重複がないことを確認するのは呼び出し元の責任です。
    #[inline]
    pub fn from_presorted_elements(elements: Vec<(K, V)>) -> SortedMap<K, V> {
        debug_assert!(elements.array_windows().all(|[fst, snd]| fst.0 < snd.0));

        SortedMap { data: elements }
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.lookup_index_for(&key) {
            Ok(index) => {
                let slot = unsafe { self.data.get_unchecked_mut(index) };
                Some(mem::replace(&mut slot.1, value))
            }
            Err(index) => {
                self.data.insert(index, (key, value));
                None
            }
        }
    }

    #[inline]
    pub fn remove(&mut self, key: &K) -> Option<V> {
        match self.lookup_index_for(key) {
            Ok(index) => Some(self.data.remove(index).1),
            Err(_) => None,
        }
    }

    #[inline]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        match self.lookup_index_for(key) {
            Ok(index) => unsafe { Some(&self.data.get_unchecked(index).1) },
            Err(_) => None,
        }
    }

    #[inline]
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        match self.lookup_index_for(key) {
            Ok(index) => unsafe { Some(&mut self.data.get_unchecked_mut(index).1) },
            Err(_) => None,
        }
    }

    /// エントリの値への可変参照を取得するか、新しい値を挿入します。
    #[inline]
    pub fn get_mut_or_insert_default(&mut self, key: K) -> &mut V
    where
        K: Eq,
        V: Default,
    {
        let index = match self.lookup_index_for(&key) {
            Ok(index) => index,
            Err(index) => {
                self.data.insert(index, (key, V::default()));
                index
            }
        };
        unsafe { &mut self.data.get_unchecked_mut(index).1 }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// キーでソートされた要素をイテレートします。
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, (K, V)> {
        self.data.iter()
    }

    /// ソートされたキーをイテレートします。
    #[inline]
    pub fn keys(&self) -> impl ExactSizeIterator<Item = &K> + DoubleEndedIterator {
        self.data.iter().map(|(k, _)| k)
    }

    /// キーでソートされた値をイテレートします。
    #[inline]
    pub fn values(&self) -> impl ExactSizeIterator<Item = &V> + DoubleEndedIterator {
        self.data.iter().map(|(_, v)| v)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn range<R>(&self, range: R) -> &[(K, V)]
    where
        R: RangeBounds<K>,
    {
        let (start, end) = self.range_slice_indices(range);
        &self.data[start..end]
    }

    /// `sm.range_is_empty(r)` は `sm.range(r).is_empty()` と等価ですが、より高速です。
    #[inline]
    pub fn range_is_empty<R>(&self, range: R) -> bool
    where
        R: RangeBounds<K>,
    {
        // `range`は (`range_slice_indices`を介して) 開始と終了を別々に検索する必要があります。
        // しかし、ここでは範囲全体に対して一度のバイナリサーチを実行できます。
        // `range`に一致する`x`が1つでも見つかれば、その範囲は空では*ありません*。
        self.data
            .binary_search_by(|(x, _)| {
                // `x`は`range`より下か？
                match range.start_bound() {
                    Bound::Included(start) if x < start => return Ordering::Less,
                    Bound::Excluded(start) if x <= start => return Ordering::Less,
                    _ => {}
                };

                // `x`は`range`より上か？
                match range.end_bound() {
                    Bound::Included(end) if x > end => return Ordering::Greater,
                    Bound::Excluded(end) if x >= end => return Ordering::Greater,
                    _ => {}
                };

                // `x`は`range`内にあるはずです。
                Ordering::Equal
            })
            .is_err()
    }

    #[inline]
    pub fn remove_range<R>(&mut self, range: R)
    where
        R: RangeBounds<K>,
    {
        let (start, end) = self.range_slice_indices(range);
        self.data.splice(start..end, std::iter::empty());
    }

    /// 与えられた関数`f`ですべてのキーを変更します。この変更はキーのソート順を変えてはなりません。
    #[inline]
    pub fn offset_keys<F>(&mut self, f: F)
    where
        F: Fn(&mut K),
    {
        self.data.iter_mut().map(|(k, _)| k).for_each(f);
    }

    /// 事前にソートされた要素の範囲をマップに挿入します。もしその範囲をマップの
    /// 既存の要素の間にまとめて挿入できる場合、要素を個別に挿入するよりも
    /// 高速になります。
    ///
    /// 要素がキーでソートされており、重複がないことを確認するのは呼び出し元の責任です。
    #[inline]
    pub fn insert_presorted(&mut self, elements: Vec<(K, V)>) {
        if elements.is_empty() {
            return;
        }

        debug_assert!(elements.array_windows().all(|[fst, snd]| fst.0 < snd.0));

        let start_index = self.lookup_index_for(&elements[0].0);

        let elements = match start_index {
            Ok(index) => {
                let mut elements = elements.into_iter();
                self.data[index] = elements.next().unwrap();
                elements
            }
            Err(index) => {
                if index == self.data.len() || elements.last().unwrap().0 < self.data[index].0 {
                    // 既存の要素と混ぜ合わせることなく、範囲全体をコピーできます。
                    self.data.splice(index..index, elements);
                    return;
                }

                let mut elements = elements.into_iter();
                self.data.insert(index, elements.next().unwrap());
                elements
            }
        };

        // 残りを挿入
        for (k, v) in elements {
            self.insert(k, v);
        }
    }

    /// `slice::binary_search()`を介して`self.data`内のキーを検索します。
    #[inline(always)]
    fn lookup_index_for<Q>(&self, key: &Q) -> Result<usize, usize>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.data.binary_search_by(|(x, _)| x.borrow().cmp(key))
    }

    #[inline]
    fn range_slice_indices<R>(&self, range: R) -> (usize, usize)
    where
        R: RangeBounds<K>,
    {
        let start = match range.start_bound() {
            Bound::Included(k) => match self.lookup_index_for(k) {
                Ok(index) | Err(index) => index,
            },
            Bound::Excluded(k) => match self.lookup_index_for(k) {
                Ok(index) => index + 1,
                Err(index) => index,
            },
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(k) => match self.lookup_index_for(k) {
                Ok(index) => index + 1,
                Err(index) => index,
            },
            Bound::Excluded(k) => match self.lookup_index_for(k) {
                Ok(index) | Err(index) => index,
            },
            Bound::Unbounded => self.data.len(),
        };

        (start, end)
    }

    #[inline]
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get(key).is_some()
    }
}

impl<K: Ord, V> IntoIterator for SortedMap<K, V> {
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<(K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<K, Q, V> Index<&Q> for SortedMap<K, V>
where
    K: Ord + Borrow<Q>,
    Q: Ord + ?Sized,
{
    type Output = V;

    fn index(&self, key: &Q) -> &Self::Output {
        self.get(key).expect("no entry found for key")
    }
}

impl<K, Q, V> IndexMut<&Q> for SortedMap<K, V>
where
    K: Ord + Borrow<Q>,
    Q: Ord + ?Sized,
{
    fn index_mut(&mut self, key: &Q) -> &mut Self::Output {
        self.get_mut(key).expect("no entry found for key")
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for SortedMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut data: Vec<(K, V)> = iter.into_iter().collect();

        data.sort_unstable_by(|(k1, _), (k2, _)| k1.cmp(k2));
        data.dedup_by(|(k1, _), (k2, _)| k1 == k2);

        SortedMap { data }
    }
}

impl<K: Debug, V: Debug> Debug for SortedMap<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.data.iter().map(|(a, b)| (a, b)))
            .finish()
    }
}
