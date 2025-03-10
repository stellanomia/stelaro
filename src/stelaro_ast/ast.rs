use crate::stelaro_common::span::Span;

use super::token::Lit;

pub struct Expr {
    id: NodeId,
    kind: ExprKind,
    span: Span,
}

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


struct NodeId(u32);
