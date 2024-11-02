pub mod error;
pub mod symbol;

#[cfg(test)]
mod tests {
    use symbol::{Interner, Symbol};

    use super::*;

    #[test]
    fn interner_tests() {
        let interner = Interner::new();
        assert_eq!(interner.intern("abc").as_u32(), 0);
        assert_eq!(interner.intern("abc").as_u32(), 0);
        assert_eq!(interner.intern("def").as_u32(), 1);
        assert_eq!(interner.intern("ghi").as_u32(), 2);
        assert_eq!(interner.intern("def").as_u32(), 1);

        assert_eq!("ghi", interner.get(Symbol::new(2)));
        assert_eq!("def", interner.get(Symbol::new(1)));
    }
}