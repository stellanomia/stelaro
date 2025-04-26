pub mod arena;
pub mod def_id;
pub mod symbol;
pub mod map;
pub mod source_map;
pub mod span;


pub use arena::{Arena, TypedArena};
pub use def_id::{DefId, LocalDefId};
pub use map::IndexMap;
pub use span::Span;
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