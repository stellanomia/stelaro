mod def_collector;

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;

use crate::stelaro_common::{Span, Symbol, TypedArena, DefId, LocalDefId, Ident, IndexMap};
use crate::stelaro_context::TyCtxt;
use crate::stelaro_sir::def::{DefKind, Namespace, PerNS, Res};
use crate::stelaro_ast::ast::{NodeId, Stelo};


/// モジュール内の名前を識別するキー
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct BindingKey {
    pub ident: Ident,
    pub ns: Namespace,
}

impl BindingKey {
    pub fn new(ident: Ident, ns: Namespace) -> Self {
        BindingKey { ident, ns }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Module<'tcx>(&'tcx ModuleData<'tcx>);

#[derive(Debug, PartialEq, Clone)]
pub struct ModuleData<'ra> {
    /// 親スコープへの参照 (ルートモジュールでは None)
    pub parent: Option<Module<'ra>>,

    /// このスコープがどのような種類か (名前付きモジュールか、単なるブロックか) を示す
    pub kind: ModuleKind,

    pub lazy_resolutions: RefCell<IndexMap<BindingKey, &'ra RefCell<NameResolution<'ra>>>>,

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

type Resolutions<'ra> = RefCell<IndexMap<BindingKey, &'ra RefCell<NameResolution<'ra>>>>;

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NameBinding<'ra>(&'ra NameBindingData<'ra>);

/// 型やモジュール定義(将来的にプライベートである可能性のある値)を記録します。
#[derive(Debug, Clone, Copy, PartialEq)]
struct NameBindingData<'ra> {
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

#[derive(Clone, Copy, Debug, PartialEq)]
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

/// モジュールの名前空間における名前解決の情報を記録します。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NameResolution<'ra> {
    // /// 名前空間内で名前を定義する可能性のある単一インポート。
    // /// インポートはアリーナに割り当てられるため、キーとしてポインタを使用しても問題ありません。
    // pub single_imports: FxIndexSet<Import<'ra>>,
    // pub shadowed_glob: Option<NameBinding<'ra>>,

    /// この名前に対して判明している、最もシャドウ（隠蔽）されにくい束縛（binding）。
    /// または、既知の束縛がない場合は None。
    pub binding: Option<NameBinding<'ra>>, // これは必須

}

impl<'ra> NameResolution<'ra> {
    /// 名前に対する束縛（binding）が判明していればそれを返し、不明な場合は None を返します。
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

    // DefIdから対応するModuleDataへのマップ (ローカルクレート内の全モジュール)
    module_map: HashMap<DefId, Module<'ra>>,

    /// NodeId (アイテム定義ノード) から、それが定義する LocalDefId へのマップ
    node_id_to_def_id: HashMap<NodeId, LocalDefId>,

    /// LocalDefId から、それを定義するアイテムの NodeId へのマップ
    def_id_to_node_id: HashMap<LocalDefId, NodeId>,

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

    fn resolutions(&mut self, module: Module<'ra>) -> &'ra Resolutions<'ra> {
        &module.0.lazy_resolutions
    }

    fn resolution(
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
}