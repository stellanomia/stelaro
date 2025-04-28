use std::ops::ControlFlow;
use visit::Visitor;

use super::BindingKey;
use super::Module;
use super::NameBinding;
use super::Resolver;
use super::ToNameBinding;

use crate::stelaro_ast::ty;
use crate::stelaro_ast::visit;
use crate::stelaro_ast::ast::*;
use crate::stelaro_common::Ident;
use crate::stelaro_sir::def::Namespace;


impl<'ra, 'tcx> Resolver<'ra, 'tcx> {
    pub fn define<T>(&mut self, parent: Module<'ra>, ident: Ident, ns: Namespace, def: T)
    where
        T: ToNameBinding<'ra>,
    {
        let binding = def.to_name_binding(self.arenas);
        let key = self.new_binding_key(ident, ns);
        if let Err(old_binding) = self.try_define(parent, key, binding) {
            self.report_conflict(parent, ident, ns, old_binding, binding);
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

        let mut resolution = todo!();

        if let Some(old_binding) = resolution.binding {
            // --- 衝突処理 ---

            // エラー回復: 新しいバインディングが Res::Err で、既存がそうでない場合、
            // 既存の有効な定義を上書きしない。
            if new_res == Res::Err && old_binding.res() != Res::Err {
                // 何もせず成功 (Ok) として扱う
                return Ok(());
            }

            // 上記以外の場合、インポート解決をスキップする前提では、
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

struct DefCollector<'r, 'ra, 'tcx> {
    resolver: &'r mut Resolver<'ra, 'tcx>,
}

impl<'r, 'ra, 'tcx> Visitor<'r> for DefCollector<'r, 'ra, 'tcx> {
    fn visit_stelo(&mut self, stelo: &'r Stelo) -> ControlFlow<Self::BreakTy> {
        visit::walk_stelo(self, stelo)
    }

    fn visit_item(&mut self, item: &'r Item) -> ControlFlow<Self::BreakTy> {
        visit::walk_item(self, item)
    }

    fn visit_fn_decl(&mut self, f: &'r Function) -> ControlFlow<Self::BreakTy> {
        visit::walk_fn_decl(self, f)
    }

    fn visit_ident(&mut self, _ident: &'r crate::stelaro_common::Ident) -> ControlFlow<Self::BreakTy> {
        ControlFlow::Continue(())
    }

    fn visit_block(&mut self, b: &'r Block) -> ControlFlow<Self::BreakTy> {
        visit::walk_block(self, b)
    }

    fn visit_param(&mut self, param: &'r Param) -> ControlFlow<Self::BreakTy> {
        visit::walk_param(self, param)
    }

    fn visit_fn_ret_ty(&mut self, ret_ty: &'r FnRetTy) -> ControlFlow<Self::BreakTy> {
        visit::walk_fn_ret_ty(self, ret_ty)
    }

    fn visit_stmt(&mut self, stmt: &'r Stmt) -> ControlFlow<Self::BreakTy> {
        visit::walk_stmt(self, stmt)
    }

    fn visit_ty(&mut self, ty: &'r ty::Ty) -> ControlFlow<Self::BreakTy> {
        visit::walk_ty(self, ty)
    }

    fn visit_local(&mut self, local: &'r Local) -> ControlFlow<Self::BreakTy> {
        visit::walk_local(self, local)
    }

    fn visit_path(&mut self, path: &'r Path) -> ControlFlow<Self::BreakTy> {
        visit::walk_path(self, path)
    }

    fn visit_path_segment(&mut self, path_segment: &'r PathSegment) -> ControlFlow<Self::BreakTy> {
        visit::walk_path_segment(self, path_segment)
    }

    fn visit_pat(&mut self, pat: &'r Pat) -> ControlFlow<Self::BreakTy> {
        visit::walk_pat(self, pat)
    }

    fn visit_expr(&mut self, expr: &'r Expr) -> ControlFlow<Self::BreakTy> {
        visit::walk_expr(self, expr)
    }
}