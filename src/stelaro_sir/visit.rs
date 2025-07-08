use crate::stelaro_common::{Ident, LocalDefId, Span, Symbol, VisitorResult};
use crate::stelaro_sir::{sir::*, sir_id::SirId};
use crate::{try_visit, visit_opt, walk_list};


pub trait IntoVisitor<'sir> {
    type Visitor: Visitor<'sir>;
    fn into_visitor(self) -> Self::Visitor;
}

/// `TyCtxt`から取得可能なSIRの要素。`TyCtxt`への明示的な依存を回避します。
/// このトレイトを実装するのは`!` (この場合、これらの関数が呼ばれることはありません) と
/// `TyCtxt`だけです。
pub trait SirTyCtxt<'sir> {
    /// `id` に対応する `Node` を取得します。
    fn sir_node(&self, sir_id: SirId) -> Node<'sir>;
    fn sir_body(&self, id: BodyId) -> &'sir Body<'sir>;
    fn sir_item(&self, id: ItemId) -> &'sir Item<'sir>;
}

// 実際のtcxが利用できず、ネストしたビジターの手動実装を強制する場合に使用されます。
impl<'sir> SirTyCtxt<'sir> for ! {
    fn sir_node(&self, _: SirId) -> Node<'sir> {
        unreachable!();
    }
    fn sir_body(&self, _: BodyId) -> &'sir Body<'sir> {
        unreachable!();
    }
    fn sir_item(&self, _: ItemId) -> &'sir Item<'sir> {
        unreachable!();
    }
}


pub mod nested_filter {
    use crate::stelaro_context::TyCtxt;

    use super::SirTyCtxt;

    /// ビジターがどのネストされたものを走査したいかを指定します。
    pub trait NestedFilter<'sir> {
        type MaybeTyCtxt: SirTyCtxt<'sir>;

        /// ビジターがネストされたアイテムを走査するかどうか。
        const INTER: bool;
        /// ビジターがアイテム内部のものを走査するかどうか。
        /// e.g., 関数本体
        const INTRA: bool;
    }

    /// ネストされたものを一切走査しません。新しいネストされていないものを
    /// 追加した際には、このフィルターの使用箇所が依然として有効であるかを
    /// 確認する必要があります。
    ///
    /// 特定の種類のツリー (e.g., 型や関数シグネチャ) のみを走査し、
    /// `tcx`を使い回したくない場合に使用します。
    pub struct None(());
    impl NestedFilter<'_> for None {
        type MaybeTyCtxt = !;
        const INTER: bool = false;
        const INTRA: bool = false;
    }

    /// ネストされたアイテムは走査しませんが、アイテムの
    /// 内部にあるネストされたものは走査します。
    ///
    /// `visit_all_item_likes_in_crate()` を
    /// 外側のループとして使用し、各アイテムの内容をこの設定を使って走査する
    /// ビジターを持つ、というのが一般的なパターンです。
    pub struct OnlyBodies(());
    impl<'tcx> NestedFilter<'tcx> for OnlyBodies {
        type MaybeTyCtxt = TyCtxt<'tcx>;
        const INTER: bool = false;
        const INTRA: bool = true;
    }

    /// アイテム的なものを含め、すべてのネストされたものを走査します。
    ///
    /// すべてをその字句的な文脈の中で
    /// 処理したい場合に使用されます。通常、`walk_stelo()` を実行する
    /// ことで走査を開始します。
    pub struct All(());
    impl<'tcx> NestedFilter<'tcx> for All {
        type MaybeTyCtxt = TyCtxt<'tcx>;
        const INTER: bool = true;
        const INTRA: bool = true;
    }
}

use nested_filter::NestedFilter;

/// SIR の各ノードを訪問します。
/// 各メソッドのデフォルト実装は、対応する`walk`メソッドを介して
/// 入力の部分構造を再帰的に訪問します。
///
/// クレート内のすべてのアイテムを何らかの順序で訪問したいだけの場合は、
/// `tcx.sir_visit_all_item_likes_in_stelo` を呼び出すべきです。
pub trait Visitor<'v>: Sized {
    // この型はオーバーライドされるべきではありません。`Self::MaybeTyCtxt`として便利に使うために存在します。
    type MaybeTyCtxt: SirTyCtxt<'v> = <Self::NestedFilter as NestedFilter<'v>>::MaybeTyCtxt;

    /// どのネストされたSIRを走査するかを制御するために、この型をオーバーライドしてください。
    /// この型をオーバーライドする場合、[`maybe_tcx`](Self::maybe_tcx)もオーバーライドしなければなりません。
    type NestedFilter: NestedFilter<'v> = nested_filter::None;


    /// `visit_*`メソッドの結果の型。`()`または`ControlFlow<T>`のいずれかになる。
    type Result: VisitorResult = ();

    /// `type NestedFilter`がネストされたアイテムを走査するように設定されている場合、
    /// ネストされたアイテムを取得するためのマップを提供するために、このメソッドも
    /// オーバーライドする必要があります。
    fn maybe_tcx(&mut self) -> Self::MaybeTyCtxt {
        panic!(
            "maybe_tcxを実装するか、`type NestedFilter = nested_filter::None` (デフォルト) の使用を検討してください"
        );
    }

    /// ネストされたアイテムに遭遇したときに呼び出されます。デフォルトでは、
    /// `Self::NestedFilter`が`nested_filter::None`の場合、このメソッドは何もしません。
    fn visit_nested_item(&mut self, id: ItemId) -> Self::Result {
        if Self::NestedFilter::INTER {
            let item = self.maybe_tcx().sir_item(id);
            try_visit!(self.visit_item(item));
        }
        Self::Result::output()
    }

    /// 関数の本体を走査するために呼び出されます。
    /// `visit_nested_item`と同様に、`Self::NestedFilter`を
    /// オーバーライドしない限り、デフォルトでは何もしません。
    fn visit_nested_body(&mut self, id: BodyId) -> Self::Result {
        if Self::NestedFilter::INTRA {
            let body = self.maybe_tcx().sir_body(id);
            try_visit!(self.visit_body(body));
        }
        Self::Result::output()
    }

    fn visit_item(&mut self, i: &'v Item<'v>) -> Self::Result {
        walk_item(self, i)
    }

    fn visit_body(&mut self, b: &Body<'v>) -> Self::Result {
        walk_body(self, b)
    }

    fn visit_mod(&mut self, module: &'v Mod<'v>, _s: Span, _n: SirId) -> Self::Result {
        walk_mod(self, module)
    }

    fn visit_ty(&mut self, ty: &'v Ty<'v>) -> Self::Result {
        walk_ty(self, ty)
    }

    fn visit_infer(&mut self, inf_id: SirId, _inf_span: Span) -> Self::Result {
        self.visit_id(inf_id)
    }

    fn visit_param(&mut self, param: &'v Param) -> Self::Result {
        walk_param(self, param)
    }

    fn visit_fn_ret_ty(&mut self, ret_ty: &'v FnRetTy<'v>) -> Self::Result {
        walk_fn_ret_ty(self, ret_ty)
    }

    fn visit_fn_decl(&mut self, decl: &'v FnDecl<'v>) -> Self::Result {
        walk_fn_decl(self, decl)
    }

    fn visit_fn(
        &mut self,
        ident: Ident,
        sig: FnSig<'v>,
        decl: &'v FnDecl<'v>,
        body_id: BodyId,
        _: Span,
        id: LocalDefId,
    ) -> Self::Result {
        walk_fn(self, ident, sig, decl, body_id, id)
    }

    fn visit_id(&mut self, _sir_id: SirId) -> Self::Result {
        Self::Result::output()
    }

    fn visit_name(&mut self, _name: Symbol) -> Self::Result {
        Self::Result::output()
    }

    fn visit_ident(&mut self, ident: Ident) -> Self::Result {
        walk_ident(self, ident)
    }

    fn visit_local(&mut self, l: &'v LetStmt<'v>) -> Self::Result {
        walk_local(self, l)
    }

    fn visit_block(&mut self, b: &'v Block<'v>) -> Self::Result {
        walk_block(self, b)
    }

    fn visit_stmt(&mut self, stmt: &'v Stmt<'v>) -> Self::Result {
        walk_stmt(self, stmt)
    }

    fn visit_lit(&mut self, _sir_id: SirId, _lit: Lit, _negated: bool) -> Self::Result {
        Self::Result::output()
    }

    fn visit_pat(&mut self, p: &'v Pat<'v>) -> Self::Result {
        walk_pat(self, p)
    }

    fn visit_path(&mut self, path: &'v Path<'v>) -> Self::Result {
        walk_path(self, path)
    }

    fn visit_path_segment(&mut self, segment: &'v PathSegment) -> Self::Result {
        walk_path_segment(self, segment)
    }

    fn visit_expr(&mut self, expr: &'v Expr<'v>) -> Self::Result {
        walk_expr(self, expr)
    }
}

pub fn walk_item<'v, V: Visitor<'v>>(visitor: &mut V, item: &'v Item<'v>) -> V::Result {
    let Item { owner_id: _, kind, span: _ } = item;
    try_visit!(visitor.visit_id(item.sir_id()));
    match *kind {
        ItemKind::Fn { sig, ident, body } => {
            try_visit!(visitor.visit_ident(ident));
            try_visit!(
                visitor.visit_fn(
                    ident, sig, sig.decl, body, item.span, item.owner_id.def_id
            ))
        },
        ItemKind::Mod(ident, module) => {
            try_visit!(visitor.visit_ident(ident));
            try_visit!(visitor.visit_mod(module, item.span, item.sir_id()))
        },
    }

    V::Result::output()
}

pub fn walk_param<'v, V: Visitor<'v>>(visitor: &mut V, param: &'v Param) -> V::Result {
    let Param { sir_id, ident, ty_span: _, span: _, } = param;
    try_visit!(visitor.visit_id(*sir_id));
    visitor.visit_ident(*ident)
}

pub fn walk_body<'v, V: Visitor<'v>>(visitor: &mut V, body: &Body<'v>) -> V::Result {
    let Body { params, value } = body;
    walk_list!(visitor, visit_param, *params);
    visitor.visit_expr(value)
}

pub fn walk_ty<'v, V: Visitor<'v>>(visitor: &mut V, ty: &'v Ty<'v>) -> V::Result {
    let Ty { sir_id, span: _, kind } = ty;
    try_visit!(visitor.visit_id(*sir_id));

    match *kind {
        TyKind::Path(ref path) => try_visit!(visitor.visit_path(path)),
        TyKind::Unit => {},
        TyKind::Infer => try_visit!(visitor.visit_infer(ty.sir_id, ty.span)),
    }

    V::Result::output()
}


pub fn walk_fn_decl<'v, V: Visitor<'v>>(
    visitor: &mut V,
    decl: &'v FnDecl<'v>,
) -> V::Result {
    let FnDecl { inputs, output, .. } =
        decl;
    walk_list!(visitor, visit_ty, *inputs);
    visitor.visit_fn_ret_ty(output)
}

pub fn walk_fn_ret_ty<'v, V: Visitor<'v>>(visitor: &mut V, ret_ty: &'v FnRetTy<'v>) -> V::Result {
    if let FnRetTy::Return(output_ty) = *ret_ty {
        try_visit!(visitor.visit_ty(output_ty));
    }
    V::Result::output()
}

pub fn walk_fn<'v, V: Visitor<'v>>(
    visitor: &mut V,
    _ident: Ident,
    _sig: FnSig,
    function_declaration: &'v FnDecl<'v>,
    body_id: BodyId,
    _: LocalDefId,
) -> V::Result {
    try_visit!(visitor.visit_fn_decl(function_declaration));
    visitor.visit_nested_body(body_id)
}


pub fn walk_ident<'v, V: Visitor<'v>>(visitor: &mut V, ident: Ident) -> V::Result {
    visitor.visit_name(ident.name)
}

pub fn walk_local<'v, V: Visitor<'v>>(visitor: &mut V, local: &'v LetStmt<'v>) -> V::Result {
    let LetStmt { pat, ty, init, sir_id, span: _ } = local;
    visit_opt!(visitor, visit_expr, *init);
    try_visit!(visitor.visit_id(*sir_id));
    try_visit!(visitor.visit_pat(pat));
    // visit_opt!(visitor, visit_ty_unambsig, *ty);
    V::Result::output()
}


pub fn walk_block<'v, V: Visitor<'v>>(visitor: &mut V, block: &'v Block<'v>) -> V::Result {
    let Block { stmts, expr, sir_id, span: _ } = block;
    try_visit!(visitor.visit_id(*sir_id));
    walk_list!(visitor, visit_stmt, *stmts);
    visit_opt!(visitor, visit_expr, *expr);
    V::Result::output()
}

pub fn walk_stmt<'v, V: Visitor<'v>>(visitor: &mut V, statement: &'v Stmt<'v>) -> V::Result {
    let Stmt { kind, sir_id, span: _ } = statement;
    try_visit!(visitor.visit_id(*sir_id));
    match *kind {
        StmtKind::Let(local) => visitor.visit_local(local),
        StmtKind::Item(item) => visitor.visit_nested_item(item),
        StmtKind::Expr(expression) | StmtKind::Semi(expression) => {
            visitor.visit_expr(expression)
        }
        StmtKind::Return(expr) => {
            visit_opt!(visitor, visit_expr, expr);
            V::Result::output()
        },
        StmtKind::While(e, block) => todo!(),
    }
}

pub fn walk_pat<'v, V: Visitor<'v>>(visitor: &mut V, pattern: &'v Pat<'v>) -> V::Result {
    todo!()
}

pub fn walk_mod<'v, V: Visitor<'v>>(visitor: &mut V, module: &'v Mod<'v>) -> V::Result {
    let Mod { spans: _, item_ids } = module;
    walk_list!(visitor, visit_nested_item, item_ids.iter().copied());
    V::Result::output()
}

pub fn walk_path<'v, V: Visitor<'v>>(visitor: &mut V, path: &'v Path<'v>) -> V::Result {
    let Path { segments, span: _, res: _ } = path;
    walk_list!(visitor, visit_path_segment, *segments);
    V::Result::output()
}

pub fn walk_path_segment<'v, V: Visitor<'v>>(
    visitor: &mut V,
    segment: &'v PathSegment,
) -> V::Result {
    let PathSegment { ident, sir_id, res: _ } = segment;
    try_visit!(visitor.visit_ident(*ident));
    try_visit!(visitor.visit_id(*sir_id));
    V::Result::output()
}

pub fn walk_expr<'v, V: Visitor<'v>>(visitor: &mut V, expr: &'v Expr<'v>) -> V::Result {
    todo!();
    V::Result::output()
}
