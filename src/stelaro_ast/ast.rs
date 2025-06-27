use crate::stelaro_common::{Ident, Span, Spanned};

use super::{token::{Lit, Token, TokenKind}, ty::Ty, NodeId};

#[derive(Debug)]
pub struct Stelo {
    pub items: Vec<Box<Item>>,
    pub span: ModSpan,
    pub id: NodeId,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub kind: ItemKind,
    pub id: NodeId,
    // pub vis: Visibility,
    pub span: Span,
    pub ident: Ident,
}

#[derive(Debug, Clone)]
pub enum ItemKind {
    Fn(Box<Function>),
    Mod(Ident, ModKind),
    // Struct(Struct),
    // Enum(Enum),
    // Const(Const),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub span: Span,
    pub ident: Ident,
    pub sig: FnSig,
    pub body: Box<Block>,
}

#[derive(Debug, Clone)]
pub struct FnSig {
    pub decl: FnDecl,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub inputs: Vec<Param>,
    pub output: FnRetTy,
}

#[derive(Debug, Clone)]
pub enum FnRetTy {
    /// `Span` は返り値の型が入るべき場所のSpanを表す
    Default(Span),
    Ty(Box<Ty>),
}

#[derive(Debug, Clone)]
pub struct Param {
    pub id: NodeId,
    pub ty: Box<Ty>,
    pub ident: Ident,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ModKind {
    /// `mod my_module { ... }` を表す
    Inline(Vec<Box<Item>>, ModSpan)
    // /// `mod my_module;` を表す
    // Outline,
}

#[derive(Debug, Clone)]
pub struct ModSpan {
    /// モジュールの括弧 `{ ... }` を除いた位置を指す
    pub inner_span: Span,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub id: NodeId,
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub id: NodeId,
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    Let(Box<Local>),

    /// expr 値を返す式
    Expr(Box<Expr>),

    /// expr; 式文
    Semi(Box<Expr>),

    /// while expr { block }
    While(Box<Expr>, Box<Block>),

    /// return expr;
    Return(Box<Expr>),
}

#[derive(Debug, Clone)]
pub struct Local {
    pub id: NodeId,
    pub pat: Box<Pat>,
    pub kind: LocalKind,
    pub ty: Option<Ty>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum LocalKind {
    Decl,
    Init(Box<Expr>),
}

impl LocalKind {
        pub fn init(&self) -> Option<&Expr> {
        match self {
            Self::Decl => None,
            Self::Init(i) => Some(i),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pat {
    pub id: NodeId,
    pub kind: PatKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum PatKind {
    WildCard,
    // Rustの binding @ OPT_SUBPATTERN が Option<Box<Pat>> で実装可能
    // FIXME: letバインディングによって生成される Pat は 様々な種類をとれるべきで、
    // 将来的に PatKind::Path やデストラクトを作成し、一時的な実装を廃止する。
    Ident(Ident),
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    /// 関数呼び出し
    Call(Box<Expr>, Vec<Expr>),
    /// if expr { block } else { expr }
    If(Box<Expr>, Box<Block>, Option<Box<Expr>>),
    Block(Box<Block>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Unary(UnOp, Box<Expr>),
    Lit(Lit),
    Paren(Box<Expr>),
    /// `Span` は `=` の位置を表す
    Assign(Box<Expr>, Box<Expr>, Span),
    Path(Path),
    // AssignOp(BinOp, Box<Expr>, Box<Expr>),
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

pub type BinOp = Spanned<BinOpKind>;

impl BinOp {
    pub fn from_token(token: Token) -> Self {
        use BinOpKind::*;
        use TokenKind::*;

        let kind = match token.kind {
            Plus => Add,
            Minus => Sub,
            Star => Mul,
            Percent => BinOpKind::Mod,
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

        BinOp { node: kind, span: token.span }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnOp {
    ///  `!` 演算子: 論理反転
    Not,
    ///  `-` 演算子 負の値
    Neg,
}
