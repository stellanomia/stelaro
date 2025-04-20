use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;

use crate::stelaro_common::{Span, Symbol, TypedArena, DefId, LocalDefId, Ident};
use crate::stelaro_context::TyCtxt;
use crate::stelaro_sir::def::{DefKind, PerNS, Res};
use crate::stelaro_ast::ast::{NodeId, Stelo};


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Module<'tcx>(&'tcx ModuleData<'tcx>);

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ModuleData<'ra> {
    /// 親スコープへの参照 (ルートモジュールでは None)
    pub parent: Option<Module<'ra>>,

    /// このスコープがどのような種類か (名前付きモジュールか、単なるブロックか) を示す
    pub kind: ModuleKind,

    /// このスコープ内で直接利用可能な定義をもつ
    /// 識別子 (名前) をキーとし、異なる名前空間での解決結果を値として持つ
    pub definitions: RefCell<HashMap<Ident, PerNS<Option<Res>>>>,

    pub span: Span,
}

impl<'ra> Deref for Module<'ra> {
    type Target = ModuleData<'ra>;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'ra> Module<'ra> {
    fn res(self) -> Option<Res> {
        match self.kind {
            ModuleKind::Def(kind, def_id, _) => Some(Res::Def(kind, def_id)),
            _ => None,
        }
    }

    fn def_id(self) -> Option<DefId> {
        match self.kind {
            ModuleKind::Def(_, def_id, _) => Some(def_id),
            _ => None,
        }
    }
}

impl<'ra> ModuleData<'ra> {
    pub fn new(
        parent: Option<Module<'ra>>,
        kind: ModuleKind,
        span: Span,
    ) -> Self {
        ModuleData {
            parent,
            kind,
            definitions: Default::default(),
            span,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ModuleKind {
    /// ブロックなどの、匿名のモジュール
    Block,
    /// 名前を伴うモジュール
    Def(DefKind, DefId, Option<Symbol>),
}

#[derive(Clone, Copy, Debug)]
pub struct MainDefinition {
    pub res: Res<NodeId>,
    pub span: Span,
}

/// ライフタイム `'ra` を持つデータ構造を格納するためのアリーナ
#[derive(Default)]
pub struct ResolverArenas<'ra> {
    /// モジュール定義 (`mod foo {}`) やブロック (`{ ... }`) のスコープ情報を格納する
    pub modules: TypedArena<'ra, ModuleData<'ra>>,

    /*
    /// `use` 文の情報を格納するアリーナ (インポート実装時に必要)
    pub imports: TypedArena<'ra, ImportData<'ra>>,

    /// パス (`a::b::c`) のセグメント (`Ident`) のリストを格納するアリーナ
    /// (`&'ra [Ident]` を安全に作るために使う場合)
    pub paths: TypedArena<'ra, Vec<Ident>>,

    /// 遅延名前解決を再導入する場合に必要
    pub name_resolutions: TypedArena<'ra, RefCell<NameResolution<'ra>>>,
    */
}

struct Resolver<'ra, 'tcx> {
    tcx: TyCtxt<'tcx>,
    arenas: &'ra ResolverArenas<'ra>,
    graph_root: Module<'ra>,

    /// 匿名モジュールへのマップ
    block_map: HashMap<NodeId, Module<'ra>>,

    // DefIdから対応するModuleDataへのマップ (ローカルクレート内の全モジュール)
    module_map: HashMap<DefId, Module<'ra>>,

    /// AST/SIRノード (パス、識別子など) から解決結果 (Res) へのマップ
    resolutions: RefCell<HashMap<NodeId, PerNS<Option<Res>>>>,

    /// NodeId (アイテム定義ノード) から、それが定義する LocalDefId へのマップ
    node_id_to_def_id: RefCell<HashMap<NodeId, LocalDefId>>,

    /// LocalDefId から、それを定義するアイテムの NodeId へのマップ
    def_id_to_node_id: RefCell<HashMap<LocalDefId, NodeId>>,

    /// 現在解決中のモジュール
    current_module: RefCell<Module<'ra>>,

    main_def: Option<MainDefinition>,

    /// 既に重複して定義されている名前に対して、診断がさらに重複しないようにする
    name_already_seen: HashMap<Symbol, Span>,
}

impl<'ra, 'tcx> Resolver<'ra, 'tcx> {
    fn new_module(
        &mut self,
        parent: Option<Module<'ra>>,
        kind: ModuleKind,
        span: Span,
    ) -> Module<'ra> {
        let module_data = ModuleData::new(
            parent,
            kind,
            span,
        );

        let module = Module(self.arenas.modules.alloc(module_data));

        if let Some(def_id) = module.def_id() {
            self.module_map.insert(def_id, module);
        }

        module
    }

    pub fn resolve_stelo(&mut self, stelo: &Stelo) {

    }
}