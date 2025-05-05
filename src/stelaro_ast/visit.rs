use std::ops::ControlFlow;

use crate::{stelaro_ast::ast::LocalKind, stelaro_common::Ident};

use super::{ast::*, ty::{Ty, TyKind}};


#[macro_export]
macro_rules! try_visit {
    ($e:expr) => {
        match $crate::stelaro_ast::VisitorResult::branch($e) {
            std::ops::ControlFlow::Continue(()) => (),
            #[allow(unreachable_code)]
            std::ops::ControlFlow::Break(r) => {
                return $crate::stelaro_ast::VisitorResult::from_residual(r);
            }
        }
    };
}

#[macro_export]
macro_rules! walk_list {
    ($visitor: expr, $method: ident, $list: expr $(, $($extra_args: expr),+ )?) => {
        for elem in $list {
            try_visit!($visitor.$method(elem $(, $($extra_args),*)? ));
        }
    };
}

#[macro_export]
macro_rules! visit_opt {
    ($visitor: expr, $method: ident, $opt: expr $(, $($extra_args: expr),+ )?) => {
        if let Some(value) = $opt {
            try_visit!($visitor.$method(value $(, $($extra_args),*)? ));
        }
    };
}

/// Visitor パターンの走査結果を抽象化するトレイト。
///
/// このトレイトは、走査が中断せずに完了した (`Continue`) か、
/// 途中で中断された (`Break`) かを統一的に扱うための抽象です。
/// `Visitor::Result` のトレイト境界として使用され、主に `()` (中断しない) と
/// `ControlFlow<T>` (中断する可能性がある) の2つの型に対して実装されます。
pub trait VisitorResult {
    /// 走査が中断された場合に返される値の型。
    type Residual;

    /// 走査が中断せずに完了した場合に返されるデフォルトの「継続」を示す値。
    fn output() -> Self;

    /// 中断を示す `Residual` の値から `VisitorResult` 型の値を生成します。
    fn from_residual(residual: Self::Residual) -> Self;

    /// `ControlFlow<Self::Residual>` (中断の可能性を含むフロー) から `VisitorResult` 型の値を生成します。
    fn from_branch(b: ControlFlow<Self::Residual>) -> Self;

    /// `VisitorResult` の値から、中断 (`Break`) または継続 (`Continue`) を示す
    /// `ControlFlow<Self::Residual>` を得ます。
    fn branch(self) -> ControlFlow<Self::Residual>;
}

/// `Visitor` による走査が中断しないときの型。
impl VisitorResult for () {
    /// AST の走査を中断しないとき、`!` (never type) により
    /// 処理の中断がデフォルトでは発生しないことが型レベルで保証される。
    type Residual = !;

    fn output() -> Self {}
    fn from_residual(redidual: Self::Residual) -> Self { match redidual {} }
    fn from_branch(b: ControlFlow<Self::Residual>) -> Self {
        match b {
            ControlFlow::Continue(c) => c,
            ControlFlow::Break(residual) => match residual {},
        }
    }
    fn branch(self) -> ControlFlow<Self::Residual> {
        ControlFlow::Continue(())
    }
}

/// `Visitor` による走査が中断する可能性がある実装に使用されます。
impl<T> VisitorResult for ControlFlow<T> {
    type Residual = T;

    fn output() -> Self {
        ControlFlow::Continue(())
    }

    fn from_residual(residual: Self::Residual) -> Self {
        ControlFlow::Break(residual)
    }

    fn from_branch(b: Self) -> Self {
        b
    }

    fn branch(self) -> Self {
        self
    }
}

/// AST (Abstract Syntax Tree) を走査するための Visitor パターン。
///
/// 各 `visit_*` メソッドは `VisitorResult` を返します。
/// これは、走査を継続するか、中断するかの結果の抽象です。
///
/// ## 走査の中断
///
/// このトレイトのデフォルト実装 (`walk_*`) はすべてのノードを訪問しようとしますが、
/// `Visitor` トレイトを実装する型は、任意の `visit_*` メソッドをオーバーライドして
/// `Self::Result` を返すことにより、任意のオーバーライドされた型を返却しつつ走査を早期中断できます。
///
/// そのため、エラー検出や特定ノードの発見などで走査を早期中断したい場合は、
/// `Visitor` 実装時に `type Result` を具体的な型にオーバーライドしてください。
/// (e.g. `type Result = ControlFlow<Span>;`)
pub trait Visitor<'ast> {
    /// この `Visitor` の返り値を抽象化する関連型。
    /// デフォルトの `()` (ユニット型) では、`Residual` が `!` (never type) となり、
    /// 処理の中断が発生しないことが型レベルで保証される。
    /// 中断が必要な場合は、この関連型をオーバーライドする。
    type Result: VisitorResult = ();

    fn visit_stelo(&mut self, stelo: &'ast Stelo) -> Self::Result {
        walk_stelo(self, stelo)
    }

    fn visit_item(&mut self, item: &'ast Item) -> Self::Result {
        walk_item(self, item)
    }

    fn visit_fn_decl(&mut self, f: &'ast Function) -> Self::Result {
        walk_fn_decl(self, f)
    }

    fn visit_ident(&mut self, _ident: &'ast Ident) -> Self::Result {
        Self::Result::output()
    }

    fn visit_block(&mut self, b: &'ast Block) -> Self::Result {
        walk_block(self, b)
    }

    fn visit_param(&mut self, param: &'ast Param) -> Self::Result {
        walk_param(self, param)
    }

    fn visit_fn_ret_ty(&mut self, ret_ty: &'ast FnRetTy) -> Self::Result {
        walk_fn_ret_ty(self, ret_ty)
    }

    fn visit_stmt(&mut self, stmt: &'ast Stmt) -> Self::Result {
        walk_stmt(self, stmt)
    }

    fn visit_ty(&mut self, ty: &'ast Ty) -> Self::Result {
        walk_ty(self, ty)
    }

    fn visit_local(&mut self, local: &'ast Local) -> Self::Result {
        walk_local(self, local)
    }

    fn visit_path(&mut self, path: &'ast Path) -> Self::Result {
        walk_path(self, path)
    }

    fn visit_path_segment(&mut self, path_segment: &'ast PathSegment) -> Self::Result {
        walk_path_segment(self, path_segment)
    }

    fn visit_pat(&mut self, pat: &'ast Pat) -> Self::Result {
        walk_pat(self, pat)
    }

    fn visit_expr(&mut self, expr: &'ast Expr) -> Self::Result {
        walk_expr(self, expr)
    }
}


pub fn walk_stelo<'ast, V>(
    visitor: &mut V,
    stelo: &'ast Stelo,
) -> V::Result
where
    V: Visitor<'ast> + ?Sized,
{
    let Stelo { items, .. } = stelo;
    walk_list!(visitor, visit_item, items);
    V::Result::output()
}

/// ASTの複数の要素が ItemKind をもつように変更するとき、WalkItemKind トレイトによって共通の訪問ができる。
pub fn walk_item<'ast, V>(
    visitor: &mut V,
    item: &'ast Item,
) -> V::Result
where
    V: Visitor<'ast> + ?Sized,
{
    let Item { kind, ident, .. } = item;

    try_visit!(visitor.visit_ident(ident));

    match kind {
        super::ast::ItemKind::Fn(function) => try_visit!(visitor.visit_fn_decl(function)),
        super::ast::ItemKind::Mod(module) => {
            match module {
                Mod::Inline(items, ..) => walk_list!(visitor, visit_item, items),
            }
        },
    }
    V::Result::output()
}

pub fn walk_fn_decl<'ast, V>(
    visitor: &mut V,
    f: &'ast Function,
) -> V::Result
where
    V: Visitor<'ast> + ?Sized,
{
    let Function { sig, body, .. } = f;
    let FnSig { params, ret_ty, ..} = sig;

    walk_list!(visitor, visit_param, params);
    try_visit!(visitor.visit_fn_ret_ty(ret_ty));
    try_visit!(visitor.visit_block(body));

    V::Result::output()
}

pub fn walk_block<'ast, V>(
    visitor: &mut V,
    b: &'ast Block,
) -> V::Result
where
    V: Visitor<'ast> + ?Sized,
{
    let Block { stmts, .. } = b;

    walk_list!(visitor, visit_stmt, stmts);

    V::Result::output()
}

pub fn walk_param<'ast, V>(
    visitor: &mut V,
    param: &'ast Param,
) -> V::Result
where
    V: Visitor<'ast> + ?Sized,
{
    let Param { ty, ident, .. } = param;

    try_visit!(visitor.visit_ty(ty));
    try_visit!(visitor.visit_ident(ident));

    V::Result::output()
}

pub fn walk_fn_ret_ty<'ast, V>(
    visitor: &mut V,
    ret_ty: &'ast FnRetTy,
) -> V::Result
where
    V: Visitor<'ast> + ?Sized,
{
    match ret_ty {
        FnRetTy::Default => {},
        FnRetTy::Ty(ty) => try_visit!(visitor.visit_ty(ty)),
    }

    V::Result::output()
}

pub fn walk_stmt<'ast, V>(
    visitor: &mut V,
    stmt: &'ast Stmt,
) -> V::Result
where
    V: Visitor<'ast> + ?Sized,
{
    let Stmt { kind, .. } = stmt;

    match kind {
        StmtKind::Let(local) => try_visit!(visitor.visit_local(local)),
        StmtKind::Expr(expr) => try_visit!(visitor.visit_expr(expr)),
        StmtKind::Semi(expr) => try_visit!(visitor.visit_expr(expr)),
        StmtKind::While(expr, block) => {
            try_visit!(visitor.visit_expr(expr));
            try_visit!(visitor.visit_block(block));
        },
        StmtKind::Return(expr) => try_visit!(visitor.visit_expr(expr)),
    }

    V::Result::output()
}

pub fn walk_ty<'ast, V>(
    visitor: &mut V,
    ty: &'ast Ty,
) -> V::Result
where
    V: Visitor<'ast> + ?Sized,
{
    let Ty { kind, .. } = ty;

    match kind {
        TyKind::Path(path) => try_visit!(visitor.visit_path(path)),
        TyKind::Infer => {},
        TyKind::Unit => {},
    }

    V::Result::output()
}

pub fn walk_local<'ast, V>(
    visitor: &mut V,
    local: &'ast Local,
) -> V::Result
where
V: Visitor<'ast> + ?Sized,
{
    let Local { pat, kind, ty, .. } = local;

    try_visit!(visitor.visit_pat(pat));
    visit_opt!(visitor, visit_ty, ty);

    match kind {
        LocalKind::Decl => {},
        LocalKind::Init(expr) => try_visit!(visitor.visit_expr(expr)),
    }

    V::Result::output()
}

pub fn walk_path<'ast, V>(
    visitor: &mut V,
    path: &'ast Path,
) -> V::Result
where
    V: Visitor<'ast> + ?Sized,
{
    let Path { segments, .. } = path;

    walk_list!(visitor, visit_path_segment, segments);

    V::Result::output()
}

pub fn walk_path_segment<'ast, V>(
    visitor: &mut V,
    path_segment: &'ast PathSegment,
) -> V::Result
where
    V: Visitor<'ast> + ?Sized,
{
    let PathSegment { ident, .. } = path_segment;

    try_visit!(visitor.visit_ident(ident));

    V::Result::output()
}

pub fn walk_pat<'ast, V>(
    visitor: &mut V,
    pat: &'ast Pat,
) -> V::Result
where
    V: Visitor<'ast> + ?Sized,
{
    let Pat { kind, .. } = pat;

    match kind {
        PatKind::WildCard => {},
        PatKind::Ident(ident) => try_visit!(visitor.visit_ident(ident)),
    }

    V::Result::output()
}

pub fn walk_expr<'ast, V>(
    visitor: &mut V,
    expr: &'ast Expr,
) -> V::Result
where
    V: Visitor<'ast> + ?Sized,
{
    let Expr { kind, .. } = expr;

    match kind {
        ExprKind::Call(func_expr, args) => {
            try_visit!(visitor.visit_expr(func_expr));
            walk_list!(visitor, visit_expr, args);
        },
        ExprKind::If(cond, then_block, else_branch) => {
            try_visit!(visitor.visit_expr(cond));
            try_visit!(visitor.visit_block(then_block));
            visit_opt!(visitor, visit_expr, else_branch);
        },
        ExprKind::Block(block) => {
            try_visit!(visitor.visit_block(block));
        },
        ExprKind::Binary(_bin_op, lhs, rhs) => {
            try_visit!(visitor.visit_expr(lhs));
            try_visit!(visitor.visit_expr(rhs));
        },
        ExprKind::Unary(_un_op, inner_expr) => {
            try_visit!(visitor.visit_expr(inner_expr));
        },
        ExprKind::Lit(_lit) => {},
        ExprKind::Return(return_expr) => {
            visit_opt!(visitor, visit_expr, return_expr);
        },
        ExprKind::Paren(expr) => {
            try_visit!(visitor.visit_expr(expr));
        },
        ExprKind::Assign(lhs, rhs) => {
            try_visit!(visitor.visit_expr(lhs));
            try_visit!(visitor.visit_expr(rhs));
        },
        ExprKind::AssignOp(_bin_op, lhs, rhs) => {
            try_visit!(visitor.visit_expr(lhs));
            try_visit!(visitor.visit_expr(rhs));
        },
        ExprKind::Path(path) => {
            try_visit!(visitor.visit_path(path));
        },
    }

    V::Result::output()
}