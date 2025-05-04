pub mod arena;
pub mod def_id;
pub mod fingerprint;
pub mod hashes;
pub mod idx;
pub mod index_vec;
pub mod map;
pub mod slice;
pub mod source_map;
pub mod span;
pub mod stable_hasher;
pub mod symbol;
pub mod unhash;


pub use arena::{Arena, TypedArena};
pub use def_id::{DefId, DefPathHash, StableSteloId, LocalDefId, DefIndex, SteloNum, LOCAL_STELO, STELO_DEF_ID, STELO_ROOT_INDEX};
// pub use fingerprint::{Fingerprint, FingerprintComponent, PackedFingerprint};
pub use hashes::{Hash64, Hash128};
pub use idx::{Idx, IntoSliceIdx};
pub use index_vec::IndexVec;
pub use map::IndexMap;
pub use slice::IndexSlice;
pub use span::Span;
// impl_hash_stable_trivial は stelaro_common 外部に公開されるべきではない
pub use stable_hasher::{StableHasher, StableHasherHash, FromStableHash};
pub use symbol::{Symbol, Ident};

use symbol::Interner;


thread_local! {
    static INTERNER: Interner = Interner::new();
}


#[cfg(test)]
mod tests {
    use symbol::{Interner, Symbol};

    use super::*;

    #[test]
    fn interner_tests() {
        let interner = Interner::new();
        assert_eq!(interner.intern("abc").as_usize(), 1);
        assert_eq!(interner.intern("abc").as_usize(), 1);
        assert_eq!(interner.intern("def").as_usize(), 2);
        assert_eq!(interner.intern("ghi").as_usize(), 3);
        assert_eq!(interner.intern("def").as_usize(), 2);

        assert_eq!("ghi", interner.get(Symbol::new(3)));
        assert_eq!("def", interner.get(Symbol::new(2)));
    }


    #[test]
    fn test_symbol() {
        let str = "Hello, World!";
        let symbol1 = Symbol::intern(&str[0..5]);
        let symbol2 = Symbol::intern(&str[5..7]);
        let symbol3 = Symbol::intern(&str[7..]);

        assert_eq!("Hello", symbol1.as_str());
        assert_eq!(", ", symbol2.as_str());
        assert_eq!("World!", symbol3.as_str());
    }

}