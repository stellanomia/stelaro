mod def_collector;
mod diagnostics;
mod ident;
mod late;
mod module_graph_builder;

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;
use std::{fmt, ptr};

use crate::stelaro_ast::{ast::Stelo, NodeId, STELO_NODE_ID};
use crate::stelaro_common::{DefId, Ident, IndexMap, IndexVec, LocalDefId, Span, Symbol, TypedArena, STELO_DEF_ID};
use crate::stelaro_context::TyCtxt;
use crate::stelaro_sir::def::{DefKind, Namespace, Res};


/// モジュール内の名前を識別するキー
#[derive(Debug, Copy, Clone, Eq, PartialOrd, Ord)]
pub struct BindingKey {
    pub ident: Ident,
    pub ns: Namespace,
}

impl BindingKey {
    pub fn new(ident: Ident, ns: Namespace) -> Self {
        BindingKey { ident, ns }
    }
}

impl PartialEq for BindingKey {
    fn eq(&self, other: &Self) -> bool {
        self.ident.name == other.ident.name && self.ns == other.ns
        // self.ident.span は比較しない
    }
}

impl std::hash::Hash for BindingKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ident.name.hash(state);
        self.ns.hash(state);
    }
}

type Resolutions<'ra> = RefCell<IndexMap<BindingKey, &'ra RefCell<NameResolution<'ra>>>>;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Module<'tcx>(&'tcx ModuleData<'tcx>);

#[derive(Clone, PartialEq, Eq)]
pub struct ModuleData<'ra> {
    /// 親スコープへの参照 (ルートモジュールでは None)
    pub parent: Option<Module<'ra>>,

    /// このスコープがどのような種類か (名前付きモジュールか、単なるブロックか) を示す
    pub kind: ModuleKind,

    /// このモジュール内における名前と (進行中である可能性のある) 解決結果との対応関係。
    pub lazy_resolutions: RefCell<IndexMap<BindingKey, &'ra RefCell<NameResolution<'ra>>>>,

    pub span: Span,
}

impl<'ra> Deref for Module<'ra> {
    type Target = ModuleData<'ra>;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

// この型は Interned (重複を避けた共有データ) として使うが、
// 実際にはデータの一意性 (Hash/PartialEqによる比較) を強制していない。
//
// 現時点では Hash を実装するが、実際にハッシュ関数を呼ばれると
// 到達不能 (unreachable) としてパニックすることで、誤使用を検知する。
impl std::hash::Hash for ModuleData<'_> {
    fn hash<H>(&self, _: &mut H)
    where
        H: std::hash::Hasher,
    {
        unreachable!("ModuleData<'ra>のhash()は呼び出されるべきではありません");
    }
}

impl<'ra> fmt::Debug for Module<'ra> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.res())
    }
}

impl<'ra> Module<'ra> {
    fn res(self) -> Option<Res> {
        match self.kind {
            ModuleKind::Def(kind, def_id, _) => Some(Res::Def(kind, def_id)),
            _ => None,
        }
    }

    fn def_id(self) -> DefId {
        self.opt_def_id().expect("`ModuleData::def_id` はブロックモジュールに対して呼ばれました")
    }

    fn opt_def_id(self) -> Option<DefId> {
        match self.kind {
            ModuleKind::Def(_, def_id, _) => Some(def_id),
            _ => None,
        }
    }

    fn is_normal(self) -> bool {
        matches!(self.kind, ModuleKind::Def(DefKind::Mod, _, _))
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
            lazy_resolutions: Default::default(),
            span,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NameBinding<'ra>(&'ra NameBindingData<'ra>);

/// 型やモジュール定義(将来的にプライベートである可能性のある値)を記録します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NameBindingData<'ra> {
    kind: NameBindingKind<'ra>,
    span: Span,
    // ambiguity: Option<(NameBinding<'ra>, AmbiguityKind)>,
    // vis: ty::Visibility<DefId>,
}

impl Hash for NameBinding<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        ptr::hash(self.0, state);
    }
}

impl Hash for NameBindingData<'_> {
    fn hash<H: std::hash::Hasher>(&self, _: &mut H) {
        unreachable!()
    }
}

pub trait ToNameBinding<'ra> {
    fn to_name_binding(self, arenas: &'ra ResolverArenas<'ra>) -> NameBinding<'ra>;
}

impl<'ra> ToNameBinding<'ra> for NameBinding<'ra> {
    fn to_name_binding(self, _: &'ra ResolverArenas<'ra>) -> NameBinding<'ra> {
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NameBindingKind<'ra> {
    Res(Res<NodeId>),
    Module(Module<'ra>),
    // Import {
    //     binding: Interned<'ra, NameBindingData<'ra>>,
    //     import: Interned<'ra, ImportData<'ra>>,
    // },
}


impl<'ra> NameBindingData<'ra> {
    fn module(&self) -> Option<Module<'ra>> {
        match self.kind {
            NameBindingKind::Module(module) => Some(module),
            // NameBindingKind::Import { binding, .. } => binding.module(),
            _ => None,
        }
    }

    fn res(&self) -> Res {
        match self.kind {
            NameBindingKind::Res(res) => res,
            NameBindingKind::Module(module) => module.res().unwrap(),
            // NameBindingKind::Import { binding, .. } => binding.res(),
        }
    }
}

impl<'ra> Deref for NameBinding<'ra> {
    type Target = NameBindingData<'ra>;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// モジュールの名前空間における名前解決の情報を記録する。
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct NameResolution<'ra> {
    // /// 名前空間内で名前を定義する可能性のある単一インポート。
    // pub single_imports: FxIndexSet<Import<'ra>>,
    // pub shadowed_glob: Option<NameBinding<'ra>>,

    /// この名前に対して判明している、最もシャドウイングされにくい束縛。
    /// 既知の束縛がない場合は None。
    pub binding: Option<NameBinding<'ra>>,
}

impl<'ra> NameResolution<'ra> {
    /// 名前に対する束縛 (binding) が判明していればそれを返し、不明な場合は None を返します。
    pub fn binding(&self) -> Option<NameBinding<'ra>> {
        // self.binding.and_then(|binding| {
        //     if !binding.is_glob_import() || self.single_imports.is_empty() { // single_imports を参照しない
        //         Some(binding)
        //     } else {
        //         None
        //     }
        // })
        self.binding
    }

    // pub fn add_single_import(&mut self, import: Import<'ra>) { ... }
    // pub fn set_binding(&mut self, binding: NameBinding<'ra>) { ... }
}

#[derive(Debug)]
pub enum PathResult<'ra> {
    Module(Module<'ra>),
    NonModule(Res),
    Indeterminate,
    Failed {
        span: Span,
        label: String,
        is_error_from_last_segment: bool,
        ///  エラーが発生した際、どのモジュール内で解決しようとしていたかを示す。
        module: Option<Module<'ra>>,
        /// 見つからなかったセグメントの名前。
        segment_name: Symbol,
        error_implied_by_parse_error: bool,
    },
}

/// 中間的な解決結果。
///
/// これは、名前によって参照されるものを指します。
/// Item はそれらのブロック全体で可視であるのに対し、Res はそれらが定義された場所から
/// 前方でのみ可視であることが異なります。
#[derive(Debug, Copy, Clone)]
pub enum LexicalScopeBinding<'ra> {
    Item(NameBinding<'ra>),
    Res(Res),
}

impl<'ra> LexicalScopeBinding<'ra> {
    fn res(self) -> Res {
        match self {
            LexicalScopeBinding::Item(binding) => binding.res(),
            LexicalScopeBinding::Res(res) => res,
        }
    }
}


/// ライフタイム `'ra` を持つデータ構造を格納するためのアリーナ
#[derive(Default)]
pub struct ResolverArenas<'ra> {
    /// モジュール定義 (`mod foo {}`) やブロック (`{ ... }`) のスコープ情報を格納する
    pub modules: TypedArena<'ra, ModuleData<'ra>>,
    pub local_modules: RefCell<Vec<Module<'ra>>>,

    /// パス (`a::b::c`) のセグメント (`Ident`) のリストを格納するアリーナ
    pub ast_paths: TypedArena<'ra, Vec<Ident>>,

    pub name_resolutions: TypedArena<'ra, RefCell<NameResolution<'ra>>>,

    pub name_bindings: TypedArena<'ra, NameBindingData<'ra>>

    /*
    /// `use` 文の情報を格納するアリーナ (インポート実装時に必要)
    pub imports: TypedArena<'ra, ImportData<'ra>>,
    */
}

impl<'ra> ResolverArenas<'ra> {
    fn new_module(
        &'ra self,
        parent: Option<Module<'ra>>,
        kind: ModuleKind,
        span: Span,
        module_map: &mut IndexMap<DefId, Module<'ra>>,
    ) -> Module<'ra> {
        let module_data = ModuleData::new(
            parent,
            kind,
            span,
        );

        let module = Module(self.modules.alloc(module_data));

        let def_id = module.opt_def_id();

        if def_id.is_none_or(|def_id| def_id.is_local()) {
            self.local_modules.borrow_mut().push(module);
        }

        if let Some(def_id) = def_id {
            module_map.insert(def_id, module);
            // let vis = ty::Visibility::<DefId>::Public;
            // let binding = (module, vis, module.span).to_name_binding(self);
            // module_self_bindings.insert(module, binding);
        }

        module
    }

    pub fn alloc_name_resolution(&'ra self) -> &'ra RefCell<NameResolution<'ra>> {
        self.name_resolutions.alloc(Default::default())
    }

    fn local_modules(&'ra self) -> std::cell::Ref<'ra, Vec<Module<'ra>>> {
        self.local_modules.borrow()
    }

    fn alloc_name_binding(&'ra self, name_binding: NameBindingData<'ra>) -> NameBinding<'ra> {
        NameBinding(self.name_bindings.alloc(name_binding))
    }
}

pub struct Resolver<'ra, 'tcx> {
    tcx: TyCtxt<'tcx>,
    arenas: &'ra ResolverArenas<'ra>,
    graph_root: Module<'ra>,

    /// 匿名モジュールへのマップ
    block_map: HashMap<NodeId, Module<'ra>>,

    // DefIdから対応するModuleDataへのマップ (ローカルステロ内の全モジュール)
    module_map: IndexMap<DefId, Module<'ra>>,
    binding_parent_modules: HashMap<NameBinding<'ra>, Module<'ra>>,

    /// NodeId (アイテム定義ノード) から、それが定義する LocalDefId へのマップ
    node_id_to_def_id: HashMap<NodeId, LocalDefId>,

    /// LocalDefId から、それを定義するアイテムの NodeId へのマップ
    def_id_to_node_id: IndexVec<LocalDefId, NodeId>,

    main_def: Option<MainDefinition>,

    /// 既に重複して定義されている名前に対して、診断がさらに重複しないようにする
    name_already_seen: HashMap<Symbol, Span>,
}

impl<'ra, 'tcx> Resolver<'ra, 'tcx> {
    fn local_def_id(&self, node: NodeId) -> LocalDefId {
        *self.node_id_to_def_id
            .get(&node)
            .unwrap_or_else(||
                panic!("bug: NodeId: {:?} に対応する定義がありません", node)
            )
    }

    fn new_module(
        &mut self,
        parent: Option<Module<'ra>>,
        kind: ModuleKind,
        span: Span,
    ) -> Module<'ra> {
        let module_map = &mut self.module_map;
        self.arenas.new_module(
            parent,
            kind,
            span,
            module_map,
        )
    }

    fn local_def_kind(&self, node: NodeId) -> DefKind {
        self.tcx.def_kind(self.local_def_id(node).into())
    }

    pub fn expect_local_module(&mut self, def_id: LocalDefId) -> Module<'ra> {
        self.get_local_module(def_id).expect("引数の `DefId` はモジュールではありません")
    }

    pub fn get_local_module(&mut self, def_id: LocalDefId) -> Option<Module<'ra>> {
        if let module @ Some(..) = self.module_map.get(&def_id.to_def_id()) {
            module.copied()
        } else {
            None
        }
    }

    fn create_def(
        &mut self,
        parent: LocalDefId,
        node_id: NodeId,
        name: Option<Symbol>,
        def_kind: DefKind,
        span: Span,
    ) -> LocalDefId {
        assert!(
            !self.node_id_to_def_id.contains_key(&node_id),
        );

        let def_id = self.tcx.create_def(parent, name, def_kind);

        let _id1 = self.tcx.source_span.borrow_mut().push(span);
        let _id2 = self.tcx.def_kind_table.borrow_mut().push(def_kind);

        debug_assert_eq!(_id1, _id2);
        debug_assert_eq!(def_id, _id1);

        self.node_id_to_def_id.insert(node_id, def_id);
        assert_eq!(self.def_id_to_node_id.push(node_id), def_id);

        def_id
    }

    pub fn resolve_stelo(&mut self, stelo: &Stelo) {
        self.build_module_graph(stelo, self.graph_root);

        if self.dcx().has_errors() {
            return;
        }

        self.late_resolve_stelo(stelo);
    }

    pub fn new_binding_key(&self, ident: Ident, ns: Namespace) -> BindingKey {
        BindingKey { ident, ns }
    }

    pub fn resolutions(&mut self, module: Module<'ra>) -> &'ra Resolutions<'ra> {
        &module.0.lazy_resolutions
    }

    pub fn resolution(
        &mut self,
        module: Module<'ra>,
        key: BindingKey,
    ) -> &'ra RefCell<NameResolution<'ra>> {
        self
            .resolutions(module)
            .borrow_mut()
            .entry(key)
            .or_insert_with(|| self.arenas.alloc_name_resolution())
    }

    pub fn set_binding_parent_module(&mut self, binding: NameBinding<'ra>, module: Module<'ra>) {
        if let Some(old_module) = self.binding_parent_modules.insert(binding, module)
            && module != old_module {
                panic!("bug: 同じ定義に対して親モジュールが変更されることはない")
            }
    }

    pub fn new(
        tcx: TyCtxt<'tcx>,
        stelo_span: Span,
        arenas: &'ra ResolverArenas<'ra>,
    ) -> Resolver<'ra, 'tcx> {
        let root_def_id = STELO_DEF_ID.to_def_id();
        let mut module_map = IndexMap::default();
        let graph_root = arenas.new_module(
            None,
            ModuleKind::Def(DefKind::Mod, root_def_id, None),
            stelo_span,
            &mut module_map,
        );

        let mut def_id_to_node_id = IndexVec::default();
        assert_eq!(def_id_to_node_id.push(STELO_NODE_ID), STELO_DEF_ID);
        let mut node_id_to_def_id: HashMap<NodeId, LocalDefId> = HashMap::default();
        let stelo: LocalDefId = tcx.create_local_stelo_def_id(stelo_span);

        node_id_to_def_id.insert(STELO_NODE_ID, stelo);

        Resolver {
            tcx,
            arenas,
            graph_root,
            block_map: Default::default(),
            module_map,
            binding_parent_modules: HashMap::new(),
            node_id_to_def_id,
            def_id_to_node_id,
            main_def: None,
            name_already_seen: HashMap::new(),
        }
    }

    // pub fn into_outputs(self) -> ResolverOutputs {

    // }
}
