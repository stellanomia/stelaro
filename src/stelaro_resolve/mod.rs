mod def_collector;
mod module_graph_builder;
mod diagnostics;

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;

use crate::stelaro_common::{DefId, Ident, IndexMap, IndexVec, LocalDefId, Span, Symbol, TypedArena};
use crate::stelaro_context::TyCtxt;
use crate::stelaro_sir::def::{DefKind, Namespace, PerNS, Res};
use crate::stelaro_ast::ast::{NodeId, Stelo};


/// モジュール内の名前を識別するキー
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BindingKey {
    pub ident: Ident,
    pub ns: Namespace,
}

impl BindingKey {
    pub fn new(ident: Ident, ns: Namespace) -> Self {
        BindingKey { ident, ns }
    }
}

type Resolutions<'ra> = RefCell<IndexMap<BindingKey, &'ra RefCell<NameResolution<'ra>>>>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Module<'tcx>(&'tcx ModuleData<'tcx>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleData<'ra> {
    /// 親スコープへの参照 (ルートモジュールでは None)
    pub parent: Option<Module<'ra>>,

    /// このスコープがどのような種類か (名前付きモジュールか、単なるブロックか) を示す
    pub kind: ModuleKind,

    pub lazy_resolutions: Resolutions<'ra>,

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
            lazy_resolutions: Default::default(),
            definitions: Default::default(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NameBinding<'ra>(&'ra NameBindingData<'ra>);

/// 型やモジュール定義(将来的にプライベートである可能性のある値)を記録します。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NameBindingData<'ra> {
    kind: NameBindingKind<'ra>,
    span: Span,
    // ambiguity: Option<(NameBinding<'ra>, AmbiguityKind)>,
    // vis: ty::Visibility<DefId>,
}

trait ToNameBinding<'ra> {
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

    // pub(crate) fn add_single_import(&mut self, import: Import<'ra>) { ... }
    // pub(crate) fn set_binding(&mut self, binding: NameBinding<'ra>) { ... }
}

/// ライフタイム `'ra` を持つデータ構造を格納するためのアリーナ
#[derive(Default)]
pub struct ResolverArenas<'ra> {
    /// モジュール定義 (`mod foo {}`) やブロック (`{ ... }`) のスコープ情報を格納する
    pub modules: TypedArena<'ra, ModuleData<'ra>>,

    /// パス (`a::b::c`) のセグメント (`Ident`) のリストを格納するアリーナ
    pub paths: TypedArena<'ra, Vec<Ident>>,

    pub name_resolutions: TypedArena<'ra, RefCell<NameResolution<'ra>>>,

    /*
    /// `use` 文の情報を格納するアリーナ (インポート実装時に必要)
    pub imports: TypedArena<'ra, ImportData<'ra>>,
    */
}

impl<'ra> ResolverArenas<'ra> {
    pub fn alloc_name_resolution(&'ra self) -> &'ra RefCell<NameResolution<'ra>> {
        self.name_resolutions.alloc(Default::default())
    }
}

struct Resolver<'ra, 'tcx> {
    tcx: TyCtxt<'tcx>,
    arenas: &'ra ResolverArenas<'ra>,
    graph_root: Module<'ra>,

    /// 匿名モジュールへのマップ
    block_map: HashMap<NodeId, Module<'ra>>,

    // DefIdから対応するModuleDataへのマップ (ローカルステロ内の全モジュール)
    module_map: HashMap<DefId, Module<'ra>>,
    binding_parent_modules: HashMap<NameBinding<'ra>, Module<'ra>>,

    /// NodeId (アイテム定義ノード) から、それが定義する LocalDefId へのマップ
    node_id_to_def_id: HashMap<NodeId, LocalDefId>,

    /// LocalDefId から、それを定義するアイテムの NodeId へのマップ
    def_id_to_node_id: IndexVec<LocalDefId, NodeId>,

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

    fn create_def(
        &mut self,
        parent: LocalDefId,
        node_id: NodeId,
        name: Option<Symbol>,
        def_kind: DefKind,
    ) -> LocalDefId {
        assert!(
            !self.node_id_to_def_id.contains_key(&node_id),
        );

        let def_id = self.tcx.create_def(parent, name, def_kind);

        self.node_id_to_def_id.insert(node_id, def_id);
        assert_eq!(self.def_id_to_node_id.push(node_id), def_id);

        def_id
    }

    pub fn resolve_stelo(&mut self, stelo: &Stelo) {

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
        if let Some(old_module) = self.binding_parent_modules.insert(binding, module) {
            if module != old_module {
                panic!("bug: 同じ定義に対して親モジュールが変更されることはない")
            }
        }
    }
}