use std::marker::PhantomData;

use bumpalo::Bump;
pub use bumpalo::Bump as Arena;

/// bumpalo::Bump をラップし、特定の型 T のみをアロケートするアリーナ
///
/// NOTE: このアリーナは bumpalo の特性を引き継ぎ、アロケートされた
/// オブジェクトの `Drop` デストラクタを呼び出さない
/// T が Drop を実装している場合、このアリーナの使用はリソースリークや
/// 未定義動作につながる可能性がある
#[derive(Debug)]
pub struct TypedArena<'a, T> {
    inner: Bump,
    _merker: PhantomData<&'a T>,
}

impl<'a, T> TypedArena<'a, T> {
    pub fn new() -> Self {
        TypedArena { inner: Bump::new(), _merker: PhantomData }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        TypedArena { inner: Bump::with_capacity(capacity), _merker: PhantomData }
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
}

impl<'a, T> Default for TypedArena<'a, T> {
    fn default() -> Self {
        Self::new()
    }
}