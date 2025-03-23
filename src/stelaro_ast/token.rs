use std::{collections::{vec_deque::Iter, VecDeque}, fmt};

use crate::stelaro_common::{span::Span, symbol::Symbol};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `%`
    Percent,
    /// `:`
    Colon,
    /// `::`
    PathSep,
    /// `;`
    Semicolon,

    /// `/`
    Slash,
    /// `//`
    LineComment,
    /// `!`
    Bang,
    /// `!=`
    BangEqual,
    /// `=`
    Equal,
    /// `==`
    EqualEqual,
    /// `>`
    Greater,
    /// `>=`
    GreaterEqual,
    /// `<`
    Less,
    // `<=`
    LessEqual,

    Ident(Symbol),

    // ExprKind が直接Litを保持するため、この構造体に抽象化している
    Literal(Lit),

    // Keywords
    Null, // null
    Fn, // fn
    Return, // return
    Let, // let
    If, // if
    Else, // else
    And, // and
    Or, // or
    For, // for
    Print, // print
    While, // while

    Eof,
}

impl Token {
    pub fn dummy() -> Self{
        Self {
            kind: TokenKind::Eof, // 便宜上EOFとする
            span: (0..0).into()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Lit {
    pub kind: LiteralKind,
    pub symbol: Symbol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LiteralKind {
    Bool(bool),
    Char,
    Integer,
    Float,
    Str,
}

#[derive(Debug)]
pub struct TokenStream (VecDeque<Token>);

impl TokenStream {
    pub fn new(tokens: VecDeque<Token>) -> Self {
        Self(tokens)
    }

    pub fn empty() -> Self {
        Self(VecDeque::new())
    }

    pub fn push(&mut self, token: Token) {
        self.0.push_back(token);
    }

    pub fn pop(&mut self) -> Option<Token> {
        self.0.pop_back()
    }

    pub fn iter(&self) -> Iter<'_, Token> {
        self.0.iter()
    }

    pub fn peek(&self) -> Option<&Token> {
        self.0.front()
    }

    pub fn peek_nth(&self, n: usize) -> Option<&Token> {
        self.0.get(n)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn check(&self, kind: TokenKind) -> bool {
        self.peek().is_some_and(|t| t.kind == kind)
    }

    pub fn check_nth(&self, n: usize, kind: TokenKind) -> bool {
        self.peek_nth(n).is_some_and(|t| t.kind == kind)
    }

    pub fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.next();
            true
        } else {
            false
        }
    }

    pub fn prepend(&mut self, token: Token) {
        self.0.push_front(token);
    }

    pub fn extend(&mut self, other: TokenStream) {
        self.0.extend(other.0);
    }

    pub fn debug_tokens(&self) -> String {
        self.0.iter()
            .map(|t| format!("{:?}", t))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Iterator for TokenStream {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }
}

impl FromIterator<Token> for TokenStream {
    fn from_iter<I: IntoIterator<Item = Token>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl std::ops::Index<usize> for TokenStream {
    type Output = Token;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

#[macro_export]
macro_rules! wrt {
    ($formatter:expr, $expr:expr) => {
        write!($formatter, "{}", $expr)
    };
}


impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::LParen => wrt!(f, "("),
            TokenKind::RParen => wrt!(f, ")"),
            TokenKind::LBrace => wrt!(f, "{"),
            TokenKind::RBrace => wrt!(f, "}"),
            TokenKind::Comma => wrt!(f, ","),
            TokenKind::Dot => wrt!(f, "."),
            TokenKind::Plus => wrt!(f, "+"),
            TokenKind::Minus => wrt!(f, "-"),
            TokenKind::Star => wrt!(f, "*"),
            TokenKind::Percent => wrt!(f, "%"),
            TokenKind::Colon => wrt!(f, ":"),
            TokenKind::PathSep => wrt!(f, "::"),
            TokenKind::Semicolon => wrt!(f, ";"),
            TokenKind::Slash => wrt!(f, "/"),
            TokenKind::LineComment => wrt!(f, "コメント"),
            TokenKind::Bang => wrt!(f, "!"),
            TokenKind::BangEqual => wrt!(f, "!="),
            TokenKind::Equal => wrt!(f, "="),
            TokenKind::EqualEqual => wrt!(f, "=="),
            TokenKind::Greater => wrt!(f, ">"),
            TokenKind::GreaterEqual => wrt!(f, ">="),
            TokenKind::Less => wrt!(f, "<"),
            TokenKind::LessEqual => wrt!(f, "<="),
            TokenKind::Ident(symbol) => wrt!(f, symbol.as_str()),
            TokenKind::Literal(lit) => wrt!(f, lit.symbol.as_str()),
            TokenKind::Null => wrt!(f, "null"),
            TokenKind::Fn => wrt!(f, "fn"),
            TokenKind::Return => wrt!(f, "return"),
            TokenKind::Let => wrt!(f, "let"),
            TokenKind::If => wrt!(f, "if"),
            TokenKind::Else => wrt!(f, "else"),
            TokenKind::And => wrt!(f, "and"),
            TokenKind::Or => wrt!(f, "or"),
            TokenKind::For => wrt!(f, "for"),
            TokenKind::Print => wrt!(f, "print"),
            TokenKind::While => wrt!(f, "while"),
            TokenKind::Eof => wrt!(f, "入力の終端"),
        }
    }
}