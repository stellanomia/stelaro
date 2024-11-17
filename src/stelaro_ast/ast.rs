use super::token::{Lit, LiteralKind};


pub struct Expr {
    id: NodeId,
    kind: ExprKind,
    line: u32,
    start: u32,
    end: u32,
}

pub enum ExprKind {
    Binary(BinOpKind, Box<Expr>, Box<Expr>),
    Unary(BinOpKind, Box<Expr>, Box<Expr>),
    Lit(Lit),
    Return(Option<Box<Expr>>),
}

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

struct NodeId(u32);