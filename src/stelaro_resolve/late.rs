use std::mem;

use crate::{stelaro_ast::{ast::*, ty::Ty, visit::{self}, NodeId, Visitor, VisitorResult}, stelaro_resolve::PathResult, stelaro_sir::def::Namespace};
use crate::{stelaro_common::{Ident, IndexMap, Span}};
use crate::stelaro_sir::def::{DefKind, Namespace::{ValueNS, TypeNS}, PerNS, Res};

use super::{Module, Resolver};

/// 単一のローカルスコープを表します。
///
/// スコープは名前が存在できる環境を定義します。これらはブロック `{...}` だけでなく、
/// アイテム定義など、名前の可視性が変わる様々な場所で導入されます。
/// スコープ内には、識別子とその解決結果 (`Res`) のマッピングが保持されます。
#[derive(Debug)]
pub struct Scope<'ra, R = Res<NodeId>> {
    /// このスコープ内で定義された束縛
    pub bindings: IndexMap<Ident, R>,
    pub kind: ScopeKind<'ra>,
}

/// 特定のスコープでどのような名前アクセスが許可されるか、あるいは制限されるかを定義します。
#[derive(Debug, Clone, Copy)]
pub enum ScopeKind<'ra> {

    /// 通常のスコープ。特別なアクセス制限は適用されません。
    NoRestriction,

    /// アイテム定義の境界を表すスコープ。
    Item(DefKind),

    /// `mod my_module { ... }` のようなモジュール定義を表します。
    Module(Module<'ra>),
}

impl<'ra, R> Scope<'ra, R> {
    fn new(kind: ScopeKind<'ra>) -> Scope<'ra, R> {
        Scope {
            bindings: Default::default(),
            kind,
        }
    }
}

/// パス (`Path`) が出現する構文上の文脈。
#[derive(Copy, Clone, Debug)]
pub enum PathSource<'a> {
    /// 型注釈などで使われるパス。
    Type,

    /// 式の中で使われるパス。
    /// `Option<&'a Expr>` は親の式への参照で、文脈依存の解決に役立つ。
    Expr(Option<&'a Expr>),

    /// パターン内で使われるパス。
    Pat,
}

impl<'a> PathSource<'a> {
    fn namespace(self) -> Namespace {
        match self {
            PathSource::Type => TypeNS,
            PathSource::Expr(..)
            | PathSource::Pat => ValueNS,
        }
    }
}

/// AST に依存しない `PathSegment` の最小限の表現。
#[derive(Clone, Copy, Debug)]
pub struct Segment {
    ident: Ident,
    id: Option<NodeId>,
}

impl Segment {
    fn from_path(path: &Path) -> Vec<Segment> {
        path.segments.iter().map(|s| s.into()).collect()
    }

    fn from_ident(ident: Ident) -> Segment {
        Segment {
            ident,
            id: None,
        }
    }

    fn from_ident_and_id(ident: Ident, id: NodeId) -> Segment {
        Segment {
            ident,
            id: Some(id),
        }
    }
}

impl<'a> From<&'a PathSegment> for Segment {
    fn from(seg: &'a PathSegment) -> Segment {
        Segment {
            ident: seg.ident,
            id: Some(seg.id),
        }
    }
}

/// 診断メッセージ生成時に使用される文脈情報を保持する構造体。
#[derive(Debug, Default)]
pub struct DiagMetadata<'ast> {
    /// 現在処理中の `Item` を表す
    current_item: Option<&'ast Item>,

    /// 現在処理中の関数を表す
    current_function: Option<(&'ast Function, Span)>,
}


/// ASTを走査し、名前解決の後半フェーズを実行する。
pub struct LateResolutionVisitor<'a, 'ast, 'ra, 'tcx> {
    r: &'a mut Resolver<'ra, 'tcx>,

    /// 処理中の親となるモジュール
    parent_module: Module<'ra>,

    /// 名前空間 (Value, Type, Macro) ごとに、現在のローカルスコープのスタックを保持する
    scopes: PerNS<Vec<Scope<'ra>>>,

    /// 最後にポップされたスコープを診断のために保持する
    last_block_scope: Option<Scope<'ra>>,

    /// 診断用のメタデータを保持する
    diag_metadata: Box<DiagMetadata<'ast>>,

    /// 関数の本体の中を処理しているかどうかを表す
    in_func_body: bool,
}

impl<'ra: 'ast, 'ast, 'tcx> Visitor<'ast> for LateResolutionVisitor<'_, 'ast, 'ra, 'tcx> {

    fn visit_item(&mut self, item: &'ast Item) -> Self::Result {
        let prev_item = self.diag_metadata.current_item.replace(item);
        let prev_in_func_body = mem::replace(&mut self.in_func_body, false);
        self.resolve_item(item);
        self.in_func_body = prev_in_func_body;
        self.diag_metadata.current_item = prev_item;
    }

    fn visit_block(&mut self, b: &'ast Block) -> Self::Result {
        self.resolve_block(b);
    }

    fn visit_expr(&mut self, expr: &'ast Expr) -> Self::Result {
        self.resolve_expr(expr, None);
    }

    fn visit_fn_decl(&mut self, f: &'ast Function) -> Self::Result {
        visit::walk_fn_decl(self, f)
    }

    fn visit_ident(&mut self, _ident: &'ast Ident) -> Self::Result {
        Self::Result::output()
    }

    fn visit_param(&mut self, param: &'ast Param) -> Self::Result {
        visit::walk_param(self, param)
    }

    fn visit_fn_ret_ty(&mut self, ret_ty: &'ast FnRetTy) -> Self::Result {
        visit::walk_fn_ret_ty(self, ret_ty)
    }

    fn visit_stmt(&mut self, stmt: &'ast Stmt) -> Self::Result {
        visit::walk_stmt(self, stmt)
    }

    fn visit_ty(&mut self, ty: &'ast Ty) -> Self::Result {
        visit::walk_ty(self, ty)
    }

    fn visit_local(&mut self, local: &'ast Local) -> Self::Result {
        visit::walk_local(self, local)
    }

    fn visit_path(&mut self, path: &'ast Path) -> Self::Result {
        visit::walk_path(self, path)
    }

    fn visit_path_segment(&mut self, path_segment: &'ast PathSegment) -> Self::Result {
        visit::walk_path_segment(self, path_segment)
    }

    fn visit_pat(&mut self, pat: &'ast Pat) -> Self::Result {
        visit::walk_pat(self, pat)
    }
}

impl<'a, 'ast, 'ra: 'ast, 'tcx> LateResolutionVisitor<'a, 'ast, 'ra, 'tcx> {
    pub fn new(resolver: &'a mut Resolver<'ra, 'tcx>) -> LateResolutionVisitor<'a, 'ast, 'ra, 'tcx> {
        let graph_root = resolver.graph_root;
        let root_kind = ScopeKind::Module(graph_root);

        LateResolutionVisitor {
            r: resolver,
            parent_module: graph_root,
            scopes: PerNS {
                value_ns: vec![Scope::new(root_kind)],
                type_ns: vec![Scope::new(root_kind)],
            },
            last_block_scope: None,
            diag_metadata: Default::default(),
            in_func_body: false,
        }
    }

    /// 与えられた名前空間 (`ns`) において、与えられた `kind` の新しい最も内側のスコープ内で、
    /// 何らかの `work` (処理) を実行します。
    fn with_rib<T>(
        &mut self,
        ns: Namespace,
        kind: ScopeKind<'ra>,
        work: impl FnOnce(&mut Self) -> T,
    ) -> T {
        self.scopes[ns].push(Scope::new(kind));
        let ret = work(self);
        self.scopes[ns].pop();
        ret
    }

    fn with_mod_scope<T>(&mut self, id: NodeId, f: impl FnOnce(&mut Self) -> T) -> T {
        let module = self.r.expect_local_module(self.r.local_def_id(id));
        // 現在のモジュールを f を実行する際の親モジュールに設定する
        let orig_module = mem::replace(&mut self.parent_module, module);
        self.with_rib(ValueNS, ScopeKind::Module(module), |this| {
            this.with_rib(TypeNS, ScopeKind::Module(module), |this| {
                let ret = f(this);
                this.parent_module = orig_module;
                ret
            })
        })
    }

    fn with_param_scope<T>(
        &mut self,
        kind: ScopeKind<'ra>,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        self.scopes[ValueNS].push(Scope::new(kind));
        self.scopes[TypeNS].push(Scope::new(kind));

        let ret = f(self);

        self.scopes[TypeNS].pop();
        self.scopes[ValueNS].pop();

        ret
    }

    fn resolve_item(&mut self, item: &'ast Item) {
        let def_kind = self.r.local_def_kind(item.id);

        match &item.kind {
            ItemKind::Fn(_) => {
                self.with_param_scope(ScopeKind::Item(def_kind), |this| {
                    visit::walk_item(this, item)
                })
            },
            ItemKind::Mod(_) => {
                self.with_mod_scope(item.id, |this| {
                    visit::walk_item(this, item)
                })
            }
        }
    }

    fn resolve_block(&mut self, block: &'ast Block) {
        let orig_module = self.parent_module;

        // NOTE: 現在、Stmt は Item をとり得らない。
        // anonymous_module が存在する場合、グラフを下げる必要がある。
        // let anonymous_module = self.r.block_map.get(&block.id).cloned();

        self.scopes[ValueNS].push(Scope::new(ScopeKind::NoRestriction));

        for stmt in &block.stmts {
            visit::walk_stmt(self, stmt)
        }

        self.parent_module = orig_module;
        self.last_block_scope = self.scopes[ValueNS].pop();
        // if anonymous_module.is_some() {
        //     self.ribs[TypeNS].pop();
        // }
    }

    fn resolve_expr(&mut self, expr: &'ast Expr, parent: Option<&'ast Expr>) {
        match &expr.kind {
            ExprKind::Call(callee, args) => {
                self.resolve_expr(callee, Some(expr));

                for arg in args {
                    self.resolve_expr(arg, None);
                }
            },
            ExprKind::If(cond, then, opt_else) => {
                self.with_rib(ValueNS, ScopeKind::NoRestriction, |this| {
                    this.visit_expr(cond);
                    this.visit_block(then);
                });

                if let Some(expr) = opt_else {
                    self.visit_expr(expr)
                }
            },
            ExprKind::Block(block) => {
                self.resolve_block(block);
            },
            ExprKind::Return(opt_expr) => {
                if let Some(expr) = opt_expr {
                    self.resolve_expr(expr, None);
                }
            },
            ExprKind::Path(path) => {
                self.resolve_path_with_context(
                    expr.id,
                    path,
                    PathSource::Expr(parent)
                );

                visit::walk_expr(self, expr);
            },
            _ => {
                visit::walk_expr(self, expr)
            }
        }
    }

    fn resolve_path_with_context(
        &mut self,
        id: NodeId,
        path: &'ast Path,
        source: PathSource<'ast>,
    ) {
        self.resolve_path_fragment_with_context(
            id,
            &Segment::from_path(path),
            path.span,
            source,
        );
    }

    fn resolve_path_fragment_with_context(
        &mut self,
        id: NodeId,
        path: &[Segment],
        path_span: Span,
        source: PathSource<'ast>,
    ) -> Res {
        let ns = source.namespace();

        let res = self.r.resolve_path_with_scopes(
            path,
             Some(ns),
            &self.parent_module,
            None,
            None,
        );


        match res {
            PathResult::Module(module) => {
                if let Some(res) = module.res() {
                    res
                } else {
                    todo!()
                }
            },
            PathResult::NonModule(res) => res,
            PathResult::Indeterminate => todo!(),
            PathResult::Failed {
                span,
                label,
                is_error_from_last_segment,
                module,
                segment_name,
                error_implied_by_parse_error
            } => todo!(),
        }
    }


    fn resolve_path(
        &mut self,
        path: &[Segment],
        opt_ns: Option<Namespace>,
    ) -> PathResult<'ra> {
        self.r.resolve_path_with_scopes(
            path,
            opt_ns,
            &self.parent_module,
            Some(&self.scopes),
            None,
        )
    }
}


impl<'ra, 'tcx> Resolver<'ra, 'tcx> {
    pub fn late_resolve_stelo(&mut self, stelo: &Stelo) {
        let mut late_resolution_visitor = LateResolutionVisitor::new(self);

        visit::walk_stelo(&mut late_resolution_visitor, stelo);
    }
}