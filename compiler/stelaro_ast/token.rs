use std::collections::{vec_deque::Iter, VecDeque};

use crate::stelaro_common::{span::Span, symbol::Symbol};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    /// '('
    LParen,
    /// ')'
    RParen,
    /// '{'
    LBrace,
    /// '}'
    RBrace,
    /// ','
    Comma,
    /// '.'
    Dot,
    /// '+'
    Plus,
    /// '-'
    Minus,
    /// '*'
    Star,
    /// '%'
    Percent,
    /// ';'
    Semicolon,

    /// '/'
    Slash,
    /// '//'
    LineComment,
    /// '!'
    Bang,
    /// '!='
    BangEqual,
    /// '='
    Equal,
    /// '=='
    EqualEqual,
    /// '>'
    Greater,
    /// '>='
    GreaterEqual,
    /// '<'
    Less,
    // '<='
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

    Question, // 分からない
}

impl Token {
    pub fn dummy() -> Self{
        Self {
            kind: TokenKind::Question,
            span: Span {
                line: 1,
                start: 0,
                end: 0
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Lit {
    pub kind: LiteralKind,
    pub symbol: Symbol,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
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

    pub fn next(&mut self) -> Option<Token> {
        self.0.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn current_line(&self) -> Option<u32> {
        self.peek().map(|t| t.span.line)
    }

    pub fn check(&self, kind: TokenKind) -> bool {
        self.peek().map_or(false, |t| t.kind == kind)
    }

    pub fn check_nth(&self, n: usize, kind: TokenKind) -> bool {
        self.peek_nth(n).map_or(false, |t| t.kind == kind)
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