use crate::stelaro_common::{Ident, Span, Symbol, VisitorResult};
use crate::stelaro_sir::{sir::*, sir_id::SirId};
use crate::{try_visit, walk_list};



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

/// SIR の各ノードを訪問します。
/// 各メソッドのデフォルト実装は、対応する`walk`メソッドを介して
/// 入力の部分構造を再帰的に訪問します。
///
/// クレート内のすべてのアイテムを何らかの順序で訪問したいだけの場合は、
/// `tcx.sir_visit_all_item_likes_in_stelo`を呼び出すべきです。
pub trait Visitor<'v>: Sized {
    /// `visit_*`メソッドの結果の型。`()`または`ControlFlow<T>`のいずれかになる。
    type Result: VisitorResult = ();

    fn visit_item(&mut self, i: &'v Item<'v>) -> Self::Result {
        walk_item(self, i)
    }

    fn visit_body(&mut self, b: &Body<'v>) -> Self::Result {
        walk_body(self, b)
    }

    fn visit_mod(&mut self, m: &'v Mod<'v>, _s: Span, _n: SirId) -> Self::Result {
        todo!()
    }

    fn visit_param(&mut self, param: &'v Param) -> Self::Result {
        walk_param(self, param)
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

    fn visit_expr(&mut self, ex: &'v Expr<'v>) -> Self::Result {
        walk_expr(self, ex)
    }
}

pub fn walk_item<'v, V: Visitor<'v>>(visitor: &mut V, item: &'v Item<'v>) -> V::Result {
    let Item { owner_id: _, kind, span: _ } = item;
    try_visit!(visitor.visit_id(item.sir_id()));
    match *kind {
        ItemKind::Fn { sig, ident, body } => {
            try_visit!(visitor.visit_ident(ident));
            todo!()
        },
        ItemKind::Mod(ident, module) => {
            try_visit!(visitor.visit_ident(ident));
            todo!()
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
    visitor.visit_expr(*value)
}

pub fn walk_ident<'v, V: Visitor<'v>>(visitor: &mut V, ident: Ident) -> V::Result {
    visitor.visit_name(ident.name)
}

pub fn walk_expr<'v, V: Visitor<'v>>(visitor: &mut V, expr: &'v Expr<'v>) -> V::Result {
    todo!();
    V::Result::output()
}
