use crate::stelaro_common::{span::Span, symbol::Ident};

use super::token::{Lit, TokenKind};

#[derive(Debug)]
pub struct Stelo {
    pub items: Vec<Item>,
    // 将来的にここにmodulesフィールドを追加
    // pub source_info: SourceInfo,
}

#[derive(Debug)]
pub struct Item {
    pub kind: ItemKind,
    pub span: Span,
    pub ident: Ident,
}

#[derive(Debug)]
pub enum ItemKind {
    Function(Function),
    Struct(Struct),
    // Enum(Enum),
    // Const(Const),
}

#[derive(Debug)]
pub struct Function {
    pub name: Ident,
    pub span: Span,
    // pub params: Vec<Parameter>,
    pub body: Block,
}


#[derive(Debug)]
pub struct Block {
    
}

#[derive(Debug)]
pub struct Struct {
    pub name: Ident,
    pub span: Span,
}

#[derive(Debug)]
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug)]
pub enum ExprKind {
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Unary(UnOp, Box<Expr>),
    Lit(Lit),
    Return(Option<Box<Expr>>),

    Assign(Box<Expr>, Box<Expr>, Span),
    AssignOp(BinOp, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, Copy)]
pub struct BinOp {
    kind: BinOpKind,
    span: Span,
}

impl BinOp {
    pub fn from_token(kind: TokenKind, span: Span) -> Self {
        use BinOpKind::*;
        use TokenKind::*;

        let kind = match kind {
            Plus => Add,
            Minus => Sub,
            Star => Mul,
            Percent => Mod,
            Slash => Div,
            BangEqual => Ne,
            EqualEqual => Eq,
            Greater => Gt,
            GreaterEqual => Ge,
            Less => Lt,
            LessEqual => Le,
            TokenKind::And => BinOpKind::And,
            TokenKind::Or => BinOpKind::Or,
            _ => panic!("bug: 二項演算子でないトークン"),
        };

        BinOp { kind, span }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BinOpKind {
    /// `+` 演算子 (addition)
    Add,
    /// `-` 演算子 (subtraction)
    Sub,
    /// `*` 演算子 (multiplication)
    Mul,
    /// `/` 演算子 (division)
    Div,
    /// `%` 演算子 (modulus)
    Mod,
    /// `and` 演算子 (logical and)
    And,
    /// `or` 演算子 (logical or)
    Or,
    /// `==` 演算子 (equality)
    Eq,
    /// `<` 演算子 (less than)
    Lt,
    /// `<=` 演算子 (less than or equal to)
    Le,
    /// `!=` 演算子 (not equal to)
    Ne,
    /// `>=` 演算子 (greater than or equal to)
    Ge,
    /// `>` 演算子 (greater than)
    Gt
}

#[derive(Debug)]
pub enum UnOp {
    ///  `!` 演算子: 論理反転
    Not,
    ///  `-` 演算子 負の値
    Neg,
}

#[derive(Debug)]
pub struct NodeId(u32);

impl NodeId {
    pub fn dummy() -> Self {
        Self(0)
    }
}
