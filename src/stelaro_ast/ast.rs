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
    /// The `+` operator (addition)
    Add,
    /// The `-` operator (subtraction)
    Sub,
    /// The `*` operator (multiplication)
    Mul,
    /// The `/` operator (division)
    Div,
    /// The `%` operator (modulus)
    Mod,
    /// The `and` operator (logical and)
    And,
    /// The `or` operator (logical or)
    Or,
    /// The `==` operator (equality)
    Eq,
    /// The `<` operator (less than)
    Lt,
    /// The `<=` operator (less than or equal to)
    Le,
    /// The `!=` operator (not equal to)
    Ne,
    /// The `>=` operator (greater than or equal to)
    Ge,
    /// The `>` operator (greater than)
    Gt
}

#[derive(Debug)]
pub enum UnOp {
    /// The `!` operator for logical inversion
    Not,
    /// The `-` operator for negation
    Neg,
}


struct NodeId(u32);
