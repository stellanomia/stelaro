use crate::stelaro_ast::ast::LocalKind;
use crate::stelaro_common::{Ident, VisitorResult};
use crate::{try_visit, visit_opt, walk_list};

use super::{
    ast::*,
    ty::{Ty, TyKind},
};

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

    fn visit_fn(&mut self, f: &'ast Function) -> Self::Result {
        walk_fn(self, f)
    }

    fn visit_fn_decl(&mut self, decl: &'ast FnDecl) -> Self::Result {
        walk_fn_decl(self, decl)
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
        super::ast::ItemKind::Fn(function) => try_visit!(visitor.visit_fn(function)),
        super::ast::ItemKind::Mod(_, module) => {
            match module {
                ModKind::Inline(items, ..) => walk_list!(visitor, visit_item, items),
            }
        },
    }
    V::Result::output()
}

pub fn walk_fn<'ast, V>(
    visitor: &mut V,
    f: &'ast Function,
) -> V::Result
where
    V: Visitor<'ast> + ?Sized,
{
    let Function { sig, body, .. } = f;
    let FnSig { decl, ..} = sig;

    try_visit!(visitor.visit_fn_decl(decl));
    try_visit!(visitor.visit_block(body));

    V::Result::output()
}

pub fn walk_fn_decl<'ast, V>(
    visitor: &mut V,
    decl: &'ast FnDecl,
) -> V::Result
where
    V: Visitor<'ast> + ?Sized,
{
    let FnDecl { inputs, output, .. } = decl;

    walk_list!(visitor, visit_param, inputs);
    try_visit!(visitor.visit_fn_ret_ty(output));

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
    let Param { ty, pat, .. } = param;

    try_visit!(visitor.visit_ty(ty));
    try_visit!(visitor.visit_pat(pat));

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
        FnRetTy::Default(_) => {},
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
        StmtKind::Loop(b) => try_visit!(visitor.visit_block(b)),
        StmtKind::While(expr, block) => {
            try_visit!(visitor.visit_expr(expr));
            try_visit!(visitor.visit_block(block));
        },
        StmtKind::Break(expr) => visit_opt!(visitor, visit_expr, expr),
        StmtKind::Continue => {},
        StmtKind::Return(expr) => visit_opt!(visitor, visit_expr, expr),
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
        ExprKind::Paren(expr) => {
            try_visit!(visitor.visit_expr(expr));
        },
        ExprKind::Assign(lhs, rhs, ..) => {
            try_visit!(visitor.visit_expr(lhs));
            try_visit!(visitor.visit_expr(rhs));
        },
        ExprKind::Path(path) => {
            try_visit!(visitor.visit_path(path));
        },
    }

    V::Result::output()
}
