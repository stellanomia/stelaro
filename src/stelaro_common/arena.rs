use std::marker::PhantomData;

use bumpalo::Bump;
use bumpalo::collections;

/// bumpalo::Bump をラップし、特定の型 T のみをアロケートするアリーナ
///
/// NOTE: このアリーナは bumpalo の特性を引き継ぎ、アロケートされた
/// オブジェクトの `Drop` デストラクタを呼び出さない
/// T が Drop を実装している場合、このアリーナの使用はリソースリークや
/// 未定義動作につながる可能性がある
#[derive(Debug)]
pub struct TypedArena<'a, T> {
    inner: Bump,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> TypedArena<'a, T> {
    pub fn new() -> Self {
        TypedArena { inner: Bump::new(), _marker: PhantomData }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        TypedArena { inner: Bump::with_capacity(capacity), _marker: PhantomData }
    }

    #[inline]
    pub fn alloc(&'a self, val: T) -> &'a T {
        self.inner.alloc(val)
    }

    #[inline]
    pub fn alloc_slice_copy(&'a self, slice: &[T]) -> &'a [T]
    where
        T: Copy,
    {
        self.inner.alloc_slice_copy(slice)
    }

    pub fn capacity(&'a self) -> usize {
        self.inner.chunk_capacity()
    }
}

impl<'a, T> Default for TypedArena<'a, T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Arena {
    inner: Bump,
}

impl Arena {
    pub fn new() -> Self {
        Arena { inner: Bump::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Arena { inner: Bump::with_capacity(capacity) }
    }

    #[inline]
    pub fn alloc<T>(&self, val: T) -> &T {
        self.inner.alloc(val)
    }

    #[inline]
    pub fn alloc_slice_copy<T>(&self, slice: &[T]) -> &[T]
    where
        T: Copy,
    {
        self.inner.alloc_slice_copy(slice)
    }

    pub fn capacity(&self) -> usize {
        self.inner.chunk_capacity()
    }

    /// イテレータの要素をアリーナに連続したスライスとして確保し、可変参照を返す。
    ///
    /// メモリ確保に失敗した場合にパニックする。
    #[inline]
    pub fn alloc_from_iter<'a, T, I>(&'a self, iter: I) -> &'a mut [T]
    where
        T: Copy + 'a,
        I: IntoIterator<Item = T>,
    {
        let mut vec = collections::Vec::new_in(&self.inner);
        vec.extend(iter);
        vec.into_bump_slice_mut()
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}
