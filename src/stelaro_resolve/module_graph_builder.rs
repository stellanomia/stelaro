use visit::Visitor;

use super::BindingKey;
use super::Module;
use super::ModuleKind;
use super::NameBinding;
use super::Resolver;
use super::ToNameBinding;

use crate::stelaro_ast::visit;
use crate::stelaro_ast::ast::*;
use crate::stelaro_common::Ident;
use crate::stelaro_sir::def::Namespace;
use crate::stelaro_sir::def::Res;


impl<'ra, 'tcx> Resolver<'ra, 'tcx> {
    pub fn define<T>(&mut self, parent: Module<'ra>, ident: Ident, ns: Namespace, def: T)
    where
        T: ToNameBinding<'ra>,
    {
        let binding = def.to_name_binding(self.arenas);
        let key = self.new_binding_key(ident, ns);
        if let Err(old_binding) = self.try_define(parent, key, binding) {
            // self.report_conflict(parent, ident, ns, old_binding, binding);
            todo!()
        }
    }

    // 衝突時は既存の NameBinding を返す
    pub(crate) fn try_define(
        &mut self,
        module: Module<'ra>,
        key: BindingKey,
        binding: NameBinding<'ra>, // 定義しようとする新しいバインディング
    ) -> Result<(), NameBinding<'ra>> {
        let new_res = binding.res();
        self.set_binding_parent_module(binding, module);

        let mut resolution = self.resolution(module, key).borrow_mut();

        if let Some(old_binding) = resolution.binding {

            // エラー回復: 新しいバインディングが Res::Err で、既存がそうでない場合、
            // 既存の有効な定義を上書きしない。
            if new_res == Res::Err && old_binding.res() != Res::Err {
                // 何もせず成功 (Ok) として扱う
                return Ok(());
            }

            // 既存の定義があるところに新しい定義をしようとしているので、
            // これは重複定義エラーとなる。
            Err(old_binding) // 衝突した既存のバインディングを返す

        } else {
            // 既存のバインディングがない場合、新しいバインディングを設定
            resolution.binding = Some(binding);
            Ok(())
        }
    }
}

struct ModuleGraphBuilder<'r, 'ra, 'tcx> {
    r: &'r mut Resolver<'ra, 'tcx>,
    parent_module: Module<'ra>,
}

impl<'r, 'ra, 'tcx> ModuleGraphBuilder<'r, 'ra, 'tcx> {
    // まだ Statement は Item をとりうらないので、匿名モジュールとして収集する必要はない。
    fn block_needs_anonymous_module(&mut self, _block: &Block) -> bool {
        // もし Statements が Item を含むなら、匿名モジュールとして作る必要がある
        // block
        //     .stmts
        //     .iter()
        //     .any(|statement| matches!(statement.kind, StmtKind::Item(_)))
        false
    }
}

impl<'r, 'ra, 'tcx> Visitor<'r> for ModuleGraphBuilder<'r, 'ra, 'tcx> {
    // 将来的に resolve_visibility する
    // fn visit_item(&mut self, item: &'r Item) -> ControlFlow<Self::BreakTy> {
    //     let parent = self.parent_module;
    //     let Item { kind, id, span, ident } = item;
    //     let def_id = local_def_id.to_def_id();
    //     let def_kind = self.r.tcx.def_kind(def_id);
    //     let res = Res::Def(def_kind, def_id);

    //     match kind {
    //         ItemKind::Function(function) => {
    //             let module = ;
    //             self.r.define(parent, *ident, Namespace::ValueNS, (module, /* vis */,span));
    //         },
    //         ItemKind::Mod(_) => todo!(),
    //     }
    //     visit::walk_item(self, item)
    // }

    fn visit_fn_decl(&mut self, f: &'r Function) {
        visit::walk_fn_decl(self, f)
    }

    fn visit_block(&mut self, b: &'r Block) {
        let parent: Module<'ra> = self.parent_module;

        if self.block_needs_anonymous_module(b) {
            let module = self.r.new_module(
                Some(parent),
                ModuleKind::Block,
                b.span,
            );

            self.r.block_map.insert(b.id, module);
            self.parent_module = module;
        }
        visit::walk_block(self, b);
        // 元の親に戻す
        self.parent_module = parent;
    }
}