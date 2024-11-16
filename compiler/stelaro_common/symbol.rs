use std::{cell::RefCell, collections::HashMap};

use super::INTERNER;


#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Symbol(u32);

impl Symbol {
    pub fn new(idx: u32) -> Self {
        Symbol(idx)
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }

    pub fn intern(string: &str) -> Self {
        INTERNER.with(|interner| {
            interner.intern(string)
        })
    }

    /// 注意: 返り値のライフタイムは&selfとは異なり、実際には
    /// 基礎となるインターナーのライフタイムに紐付いている
    /// インターナーは長命で、この関数は通常短命な用途で使用されるため、実際には問題ない
    pub fn as_str(&self) -> &str {
        INTERNER.with(|interner| {
            unsafe {std::mem::transmute::<&str, &str>(interner.get(*self))}
        })
    }
}


pub struct Interner(RefCell<InternerInner>);

struct InternerInner {
    strings: HashMap<&'static str, u32>,
    symbols: Vec<&'static str>,
    next_idx: u32
}


impl Interner {
    pub fn new() -> Self {
        Interner(
            RefCell::new(
                InternerInner {
                    strings: HashMap::with_capacity(1024),
                    symbols: Vec::with_capacity(1024),
                    next_idx: 0,
                }
            )
        )
    }

    pub fn intern(&self, string: &str) -> Symbol {
        let mut inner = self.0.borrow_mut();

        if let Some(idx) = inner.strings.get(string) {
            return Symbol::new(*idx);
        }

        let idx = inner.next_idx;

        // 安全: Internersが生きている間しかこの参照にアクセスできない
        // ライフタイムを'static に拡張
        let string: &'static str = unsafe {&*(string as *const str) };

        inner.strings.insert(string, idx);
        inner.next_idx += 1;

        inner.symbols.push(string);

        Symbol::new(idx)
    }

    // symbolsに存在するSymbolでしかアクセスされない
    pub fn get(&self, idx: Symbol) -> &str {
        self.0.borrow().symbols.get(idx.as_usize()).unwrap()
    }
}