use crate::stelaro_common::{span::Span, symbol::Ident};

use super::{token::{Lit, Token, TokenKind}, ty::Ty};

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
    // Struct(Struct),
    // Enum(Enum),
    // Const(Const),
}

#[derive(Debug)]
pub struct Function {
    pub span: Span,
    pub sig: FnSig,
    pub body: Box<Block>,
}

#[derive(Debug)]
pub struct FnSig {
    pub params: Vec<Param>,
    pub ret_ty: FnRetTy,
    pub span: Span,
}

#[derive(Debug)]
pub enum FnRetTy {
    Default,
    Ty(Box<Ty>),
}

#[derive(Debug)]
pub struct Param {
    pub id: NodeId,
    pub ty: Box<Ty>,
    pub ident: Ident,
    pub span: Span,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Block {
    pub id: NodeId,
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Stmt {
    pub id: NodeId,
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StmtKind {
    Let(Box<Local>),

    /// expr 値を返す式
    Expr(Box<Expr>),

    /// expr; 式文
    Semi(Box<Expr>),

    /// while expr { block }
    While(Box<Expr>, Box<Block>),

    /// return expr;
    Return(Box<Expr>)
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Local {
    pub id: NodeId,
    pub pat: Box<Pat>,
    pub kind: LocalKind,
    pub ty: Option<Ty>,
    pub span: Span,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LocalKind {
    Decl,
    Init(Box<Expr>),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pat {
    pub id: NodeId,
    pub kind: PatKind,
    pub span: Span,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PatKind {
    WildCard,
    // Rustの binding @ OPT_SUBPATTERN が Option<Box<Pat>> で実装可能
    Ident(Ident)
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExprKind {
    /// 関数呼び出し
    Call(Box<Expr>, Vec<Expr>),
    /// if expr { block } else { expr }
    If(Box<Expr>, Box<Block>, Option<Box<Expr>>),
    Block(Box<Block>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Unary(UnOp, Box<Expr>),
    Lit(Lit),
    Return(Option<Box<Expr>>),
    Paren(Box<Expr>),
    Assign(Box<Expr>, Box<Expr>),
    AssignOp(BinOp, Box<Expr>, Box<Expr>),
    Path(Path),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Path {
    pub span: Span,
    pub segments: Vec<PathSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PathSegment {
    pub ident: Ident,
    pub id: NodeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BinOp {
    pub kind: BinOpKind,
    pub span: Span,
}

impl BinOp {
    pub fn from_token(token: Token) -> Self {
        use BinOpKind::*;
        use TokenKind::*;

        let kind = match token.kind {
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

        BinOp { kind, span: token.span }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnOp {
    ///  `!` 演算子: 論理反転
    Not,
    ///  `-` 演算子 負の値
    Neg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(u32);

impl NodeId {
    pub const STELO_NODE_ID: NodeId = NodeId(0);

    #[inline]
    pub fn dummy() -> Self {
        Self(u32::MAX)
    }
}
