use std::ops::ControlFlow;

use crate::{stelaro_ast::ast::LocalKind, stelaro_common::Ident};

use super::{ast::*, ty::{Ty, TyKind}};


#[macro_export]
macro_rules! try_visit {
    ($e:expr) => {
        // $e は ControlFlow<T> を返す式を想定
        match $e {
            // Continue なら何もしない (ループや処理を続ける)
            core::ops::ControlFlow::Continue(()) => (),
            // Break なら residual 値を現在の関数から返す
            // (Visitorの訪問が終了する)
            core::ops::ControlFlow::Break(residual) => {
                return core::ops::ControlFlow::Break(residual);
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

/// AST (Abstract Syntax Tree) を走査するための Visitor パターン。
///
/// 各 `visit_*` メソッドは `ControlFlow<Self::BreakTy>` を返します。
/// これは、走査を継続するか (`Continue(())`)、中断するか (`Break(value)`) を示します。
///
/// ## 走査の中断 (`ControlFlow::Break`)
///
/// このトレイトのデフォルト実装（`walk_*`）はすべてのノードを訪問しようとしますが、
/// `Visitor` トレイトを実装する型は、任意の `visit_*` メソッドをオーバーライドして
/// `ControlFlow::Break` を返すことにより、任意の型 (`BreakTy`) を返却しつつ走査を早期中断できます。
///
/// そのため、エラー検出や特定ノードの発見などで走査を早期中断したい場合は、
/// `Visitor` 実装時に `BreakTy` を具体的な型にオーバーライドしてください。
pub trait Visitor<'ast> {
    /// 早期リターン時に `ControlFlow::Break` で返される値の型。
    /// `!` (never type)により、処理の中断がデフォルトでは発生しないことが型レベルで保証される。
    /// 中断が必要な場合は、この関連型をオーバーライドする。
    type BreakTy = !;

    fn visit_stelo(&mut self, stelo: &'ast Stelo) -> ControlFlow<Self::BreakTy> {
        walk_stelo(self, stelo)
    }

    fn visit_item(&mut self, item: &'ast Item) -> ControlFlow<Self::BreakTy> {
        walk_item(self, item)
    }

    fn visit_fn_decl(&mut self, f: &'ast Function) -> ControlFlow<Self::BreakTy> {
        walk_fn_decl(self, f)
    }

    fn visit_ident(&mut self, _ident: &'ast Ident) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }

    fn visit_block(&mut self, b: &'ast Block) -> ControlFlow<Self::BreakTy> {
        walk_block(self, b)
    }

    fn visit_param(&mut self, param: &'ast Param) -> ControlFlow<Self::BreakTy> {
        walk_param(self, param)
    }

    fn visit_fn_ret_ty(&mut self, ret_ty: &'ast FnRetTy) -> ControlFlow<Self::BreakTy> {
        walk_fn_ret_ty(self, ret_ty)
    }

    fn visit_stmt(&mut self, stmt: &'ast Stmt) -> ControlFlow<Self::BreakTy> {
        walk_stmt(self, stmt)
    }

    fn visit_ty(&mut self, ty: &'ast Ty) -> ControlFlow<Self::BreakTy> {
        walk_ty(self, ty)
    }

    fn visit_local(&mut self, local: &'ast Local) -> ControlFlow<Self::BreakTy> {
        walk_local(self, local)
    }

    fn visit_path(&mut self, path: &'ast Path) -> ControlFlow<Self::BreakTy> {
        walk_path(self, path)
    }

    fn visit_path_segment(&mut self, path_segment: &'ast PathSegment) -> ControlFlow<Self::BreakTy> {
        walk_path_segment(self, path_segment)
    }

    fn visit_pat(&mut self, pat: &'ast Pat) -> ControlFlow<Self::BreakTy> {
        walk_pat(self, pat)
    }

    fn visit_expr(&mut self, expr: &'ast Expr) -> ControlFlow<Self::BreakTy> {
        walk_expr(self, expr)
    }
}


pub fn walk_stelo<'ast, V>(
    visitor: &mut V,
    stelo: &'ast Stelo,
) -> ControlFlow<V::BreakTy>
where
    V: Visitor<'ast> + ?Sized,
{
    let Stelo { items, .. } = stelo;
    walk_list!(visitor, visit_item, items);
    ControlFlow::Continue(())
}

/// ASTの複数の要素が ItemKind をもつように変更するとき、WalkItemKind トレイトによって共通の訪問ができる。
pub fn walk_item<'ast, V>(
    visitor: &mut V,
    item: &'ast Item,
) -> ControlFlow<V::BreakTy>
where
    V: Visitor<'ast> + ?Sized,
{
    let Item { kind, ident, .. } = item;

    visitor.visit_ident(ident)?;

    match kind {
        super::ast::ItemKind::Function(function) => visitor.visit_fn_decl(function)?,
        super::ast::ItemKind::Mod(module) => {
            match module {
                Mod::Inline(items, ..) => walk_list!(visitor, visit_item, items),
            }
        },
    }
    ControlFlow::Continue(())
}

pub fn walk_fn_decl<'ast, V>(
    visitor: &mut V,
    f: &'ast Function,
) -> ControlFlow<V::BreakTy>
where
    V: Visitor<'ast> + ?Sized,
{
    let Function { sig, body, .. } = f;
    let FnSig { params, ret_ty, ..} = sig;

    walk_list!(visitor, visit_param, params);
    visitor.visit_fn_ret_ty(ret_ty)?;
    visitor.visit_block(body)?;

    ControlFlow::Continue(())
}

pub fn walk_block<'ast, V>(
    visitor: &mut V,
    b: &'ast Block,
) -> ControlFlow<V::BreakTy>
where
    V: Visitor<'ast> + ?Sized,
{
    let Block { stmts, .. } = b;

    walk_list!(visitor, visit_stmt, stmts);

    ControlFlow::Continue(())
}

pub fn walk_param<'ast, V>(
    visitor: &mut V,
    param: &'ast Param,
) -> ControlFlow<V::BreakTy>
where
    V: Visitor<'ast> + ?Sized,
{
    let Param { ty, ident, .. } = param;

    visitor.visit_ty(ty)?;
    visitor.visit_ident(ident)?;

    ControlFlow::Continue(())
}

pub fn walk_fn_ret_ty<'ast, V>(
    visitor: &mut V,
    ret_ty: &'ast FnRetTy,
) -> ControlFlow<V::BreakTy>
where
    V: Visitor<'ast> + ?Sized,
{
    match ret_ty {
        FnRetTy::Default => {},
        FnRetTy::Ty(ty) => visitor.visit_ty(ty)?,
    }

    ControlFlow::Continue(())
}

pub fn walk_stmt<'ast, V>(
    visitor: &mut V,
    stmt: &'ast Stmt,
) -> ControlFlow<V::BreakTy>
where
    V: Visitor<'ast> + ?Sized,
{
    let Stmt { kind, .. } = stmt;

    match kind {
        StmtKind::Let(local) => visitor.visit_local(local)?,
        StmtKind::Expr(expr) => visitor.visit_expr(expr)?,
        StmtKind::Semi(expr) => visitor.visit_expr(expr)?,
        StmtKind::While(expr, block) => {
            visitor.visit_expr(expr)?;
            visitor.visit_block(block)?;
        },
        StmtKind::Return(expr) => visitor.visit_expr(expr)?,
    }

    ControlFlow::Continue(())
}

pub fn walk_ty<'ast, V>(
    visitor: &mut V,
    ty: &'ast Ty,
) -> ControlFlow<V::BreakTy>
where
    V: Visitor<'ast> + ?Sized,
{
    let Ty { kind, .. } = ty;

    match kind {
        TyKind::Path(path) => visitor.visit_path(path)?,
        TyKind::Infer => {},
        TyKind::Unit => {},
    }

    ControlFlow::Continue(())
}

pub fn walk_local<'ast, V>(
    visitor: &mut V,
    local: &'ast Local,
) -> ControlFlow<V::BreakTy>
where
V: Visitor<'ast> + ?Sized,
{
    let Local { pat, kind, ty, .. } = local;

    visitor.visit_pat(pat)?;
    visit_opt!(visitor, visit_ty, ty);

    match kind {
        LocalKind::Decl => {},
        LocalKind::Init(expr) => visitor.visit_expr(expr)?,
    }

    ControlFlow::Continue(())
}

pub fn walk_path<'ast, V>(
    visitor: &mut V,
    path: &'ast Path,
) -> ControlFlow<V::BreakTy>
where
    V: Visitor<'ast> + ?Sized,
{
    let Path { segments, .. } = path;

    walk_list!(visitor, visit_path_segment, segments);

    ControlFlow::Continue(())
}

pub fn walk_path_segment<'ast, V>(
    visitor: &mut V,
    path_segment: &'ast PathSegment,
) -> ControlFlow<V::BreakTy>
where
    V: Visitor<'ast> + ?Sized,
{
    let PathSegment { ident, .. } = path_segment;

    visitor.visit_ident(ident)?;

    ControlFlow::Continue(())
}

pub fn walk_pat<'ast, V>(
    visitor: &mut V,
    pat: &'ast Pat,
) -> ControlFlow<V::BreakTy>
where
    V: Visitor<'ast> + ?Sized,
{
    let Pat { kind, .. } = pat;

    match kind {
        PatKind::WildCard => {},
        PatKind::Ident(ident) => visitor.visit_ident(ident)?,
    }

    ControlFlow::Continue(())
}

pub fn walk_expr<'ast, V>(
    visitor: &mut V,
    expr: &'ast Expr,
) -> ControlFlow<V::BreakTy>
where
    V: Visitor<'ast> + ?Sized,
{
    let Expr { kind, .. } = expr;

    match kind {
        ExprKind::Call(func_expr, args) => {
            visitor.visit_expr(func_expr)?;
            walk_list!(visitor, visit_expr, args);
        },
        ExprKind::If(cond, then_block, else_branch) => {
            visitor.visit_expr(cond)?;
            visitor.visit_block(then_block)?;
            visit_opt!(visitor, visit_expr, else_branch);
        },
        ExprKind::Block(block) => {
            visitor.visit_block(block)?;
        },
        ExprKind::Binary(_bin_op, lhs, rhs) => {
            visitor.visit_expr(lhs)?;
            visitor.visit_expr(rhs)?;
        },
        ExprKind::Unary(_un_op, inner_expr) => {
            visitor.visit_expr(inner_expr)?;
        },
        ExprKind::Lit(_lit) => {},
        ExprKind::Return(return_expr) => {
            visit_opt!(visitor, visit_expr, return_expr);
        },
        ExprKind::Paren(expr) => {
            visitor.visit_expr(expr)?;
        },
        ExprKind::Assign(lhs, rhs) => {
            visitor.visit_expr(lhs)?;
            visitor.visit_expr(rhs)?;
        },
        ExprKind::AssignOp(_bin_op, lhs, rhs) => {
            visitor.visit_expr(lhs)?;
            visitor.visit_expr(rhs)?;
        },
        ExprKind::Path(path) => {
            visitor.visit_path(path)?;
        },
    }

    ControlFlow::Continue(())
}