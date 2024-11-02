use std::{cell::RefCell, collections::HashMap};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Symbol(u32);

impl Symbol {
    pub fn new(idx: u32) -> Self {
        Symbol(idx)
    }

    pub fn as_u32(&self) -> u32 {
        self.0 as u32
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

struct InternerInner {
    strings: HashMap<&'static str, u32>,
    symbols: Vec<&'static str>,
    next_idx: u32
}

pub struct Interner(RefCell<InternerInner>);

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

        // 安全: symbolsが生きている間しかこの参照にアクセスできないため
        // ライフタイムを'static に拡張できる
        let string: &'static str = unsafe {&*(string as *const str) };

        inner.strings.insert(string, idx);
        inner.next_idx += 1;

        inner.symbols.push(string);

        Symbol::new(idx)
    }

    pub fn get(&self, idx: Symbol) -> &str {
        self.0.borrow().symbols.get(idx.as_usize()).unwrap()
    }
}