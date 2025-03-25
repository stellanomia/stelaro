use std::{cell::RefCell, collections::HashMap};

use super::{span::Span, INTERNER};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Ident {
    pub name: Symbol,
    pub span: Span,
}

impl Ident {
    pub fn new(name: Symbol, span: Span) -> Self {
        Self { name, span }
    }
}


#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Symbol(u32);

impl Symbol {
    pub const UNDERSCORE: Symbol = Symbol(0);

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
    /// ※インターナーが参照する文字列が解放された後にこれを呼び出してはいけない
    pub fn as_str(&self) -> &str {
        INTERNER.with(|interner| {
            unsafe {std::mem::transmute::<&str, &str>(interner.get(*self))}
        })
    }

    pub fn is_underscore(self) -> bool {
        self == Symbol::UNDERSCORE
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
        Interner::default()
    }

    pub fn intern(&self, string: &str) -> Symbol {
        let mut inner = self.0.borrow_mut();
        inner.intern(string)
    }

    pub fn get(&self, idx: Symbol) -> &str {
        // symbolsに存在するSymbolでしかアクセスされない
        self.0.borrow().symbols.get(idx.as_usize()).unwrap()
    }
}

impl InternerInner {
    #[inline]
    pub fn intern(&mut self, string: &str) -> Symbol {
        if let Some(idx) = self.strings.get(string) {
            return Symbol::new(*idx);
        }

        let idx = self.next_idx;

        // 安全: Internerが生きている間しかこの参照にアクセスできない
        // また、&'static str は外部へ持ち込まれない
        // ライフタイムを'static に拡張
        let string: &'static str = unsafe {&*(string as *const str) };

        self.strings.insert(string, idx);
        self.next_idx += 1;

        self.symbols.push(string);

        Symbol::new(idx)
    }
}

impl Default for Interner {
    fn default() -> Self {
        let mut inner = InternerInner {
            strings: HashMap::with_capacity(1024),
            symbols: Vec::with_capacity(1024),
            next_idx: 0,
        };

        inner.intern("_");

        Interner(
            RefCell::new(
                inner
            )
        )
    }
}