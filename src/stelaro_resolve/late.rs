use crate::stelaro_ast::{ast::{Function, Item, Stelo}, visit, NodeId, Visitor};
use crate::stelaro_common::{Ident, IndexMap, Span};
use crate::stelaro_sir::def::{DefKind, PerNS, Res};

use super::{Module, Resolver};

/// 単一のローカルスコープを表します。
///
/// スコープは名前が存在できる環境を定義します。これらはブロック `{...}` だけでなく、
/// アイテム定義など、名前の可視性が変わる様々な場所で導入されます。
/// スコープ内には、識別子とその解決結果 (`Res`) のマッピングが保持されます。
#[derive(Debug)]
struct Scope<'ra, R = Res<NodeId>> {
    /// このスコープ内で定義された束縛
    pub bindings: IndexMap<Ident, R>,
    pub kind: ScopeKind<'ra>,
}

/// 特定のスコープでどのような名前アクセスが許可されるか、あるいは制限されるかを定義します。
#[derive(Debug, Clone, Copy)]
enum ScopeKind<'ra> {

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

    /// 名前空間（Value, Type, Macro）ごとに、現在のローカルスコープのスタックを保持する
    scopes: PerNS<Vec<Scope<'ra>>>,

    /// 最後にポップされたスコープを診断のために保持する
    last_block_scope: Option<Scope<'ra>>,

    /// 診断用のメタデータを保持する
    diag_metadata: Box<DiagMetadata<'ast>>,
}

impl<'ra: 'ast, 'ast, 'tcx> Visitor<'ast> for LateResolutionVisitor<'_, 'ast, 'ra, 'tcx> {

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
        }
    }
}


impl<'ra, 'tcx> Resolver<'ra, 'tcx> {
    pub fn late_resolve_stelo(&mut self, stelo: &Stelo) {
        let mut late_resolution_visitor = LateResolutionVisitor::new(self);

        visit::walk_stelo(&mut late_resolution_visitor, stelo);
    }
}