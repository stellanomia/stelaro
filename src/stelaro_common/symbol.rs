use std::{cell::RefCell, collections::HashMap, fmt, hash::{Hash, Hasher}};

use super::{span::Span, SESSION_GLOBALS};

#[derive(Debug, Clone, Copy, Eq, PartialOrd, Ord)]
pub struct Ident {
    pub name: Symbol,
    pub span: Span,
}

impl Ident {
    pub fn new(name: Symbol, span: Span) -> Self {
        Self { name, span }
    }

    pub fn is_underscore(&self) -> bool {
        self.name == sym::UNDERSCORE
    }
}

impl PartialEq for Ident {
    #[inline]
    fn eq(&self, rhs: &Self) -> bool {
        self.name == rhs.name
    }
}

impl Hash for Ident {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name.as_str())
    }
}


#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct Symbol(u32);

impl Symbol {
    pub fn new(idx: u32) -> Self {
        Symbol(idx)
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }

    pub fn intern(string: &str) -> Self {
        SESSION_GLOBALS.with(|session_globals| {
            session_globals.symbol_interner.intern(string)
        })
    }

    /// 注意: 返り値のライフタイムは&selfとは異なり、実際には
    /// 基礎となるインターナーのライフタイムに紐付いている
    /// インターナーは長命で、この関数は通常短命な用途で使用されるため、実際には問題ない
    /// ※インターナーが参照する文字列が解放された後にこれを呼び出してはいけない
    pub fn as_str(&self) -> &str {
        SESSION_GLOBALS.with(|session_globals| {
            unsafe {
                std::mem::transmute::<&str, &str>(
                    session_globals.symbol_interner.get(*self)
                )
            }
        })
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
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

        // SAFETY: Internerが生きている間しかこの参照にアクセスできない
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

        PREFILLED_STRINGS.iter().for_each(
            |str| {
                inner.intern(str);
            }
        );

        Interner(
            RefCell::new(
                inner
            )
        )
    }
}


/// 事前定義する文字列から、
/// `sym::KEYWORD` 形式のシンボル定数と、
/// インターナー初期化用の文字列配列 `PREFILLED_STRINGS` を自動生成します。
macro_rules! define_keywords_and_symbols {
    // マクロのエントリポイント
    // 末尾のカンマも許容する `$(,)?`
    ($($keyword_ident:ident => $keyword_str:literal),* $(,)?) => {
        pub mod sym {
            // 内部マクロを呼び出して、定数定義を生成する
            // `@idx 0;`: 最初のシンボルIDは0から
            // `@accumulated_defs ();`: 定義を一時保存する場所
            // `$($keyword_ident => $keyword_str,)*`: 与えられたキーワードリスト
            _generate_sym_constants_impl! {
                @idx 0;
                @accumulated_defs ();
                $($keyword_ident => $keyword_str,)*
            }
        }

        const PREFILLED_STRINGS: &[&str] = &[
            $($keyword_str),*
        ];
    };
}

/// `define_keywords_and_symbols!` のための内部ヘルパーマクロ。
/// 再帰を使ってシンボル定数を一つずつ作ります。(TT Muncher)
#[doc(hidden)]
macro_rules! _generate_sym_constants_impl {
    // 入力リストが空になったら、蓄積された定義とカウントを出力
    // `@idx $idx`: キーワードの総数 (次のID)
    // `@accumulated_defs ($($defs:tt)*)`: 作成済みの `pub const ...` 定義すべて
    (
        @idx $idx:expr;
        @accumulated_defs ($($defs:tt)*);
        // ここに何も続かない (入力リストが空)
    ) => {
        $($defs)*
        pub const PREFILLED_COUNT: usize = $idx;
    };

    // 入力リストから一つ取り出し、残りを再帰呼び出しに渡す
    (
        @idx $idx:expr;
        @accumulated_defs ($($defs:tt)*);
        $name:ident => $val:literal,
        $($rest_keywords:ident => $rest_values:literal,)* // 残りの要素
    ) => {
        _generate_sym_constants_impl! {
            @idx $idx + 1;
            // 今までの定義に、新しい `pub const $name ...` を追加して保存。
            @accumulated_defs (
                $($defs)* // 既存の定義を展開
                pub const $name: $crate::stelaro_common::symbol::Symbol = $crate::stelaro_common::symbol::Symbol($idx);
            );
            // 残りのキーワードリストを渡し、処理を継続する
            $($rest_keywords => $rest_values,)*
        }
    };
}

define_keywords_and_symbols! {
    UNDERSCORE => "_",
    UNKNOWN => "unknown",
    STELO => "stelo",
    STELARO_OUT => "stelaro_out",
    MAIN => "main",
    LET => "let",
    I32 => "i32",
    I64 => "i64",
    BOOL => "bool",
    CHAR => "char",
}
