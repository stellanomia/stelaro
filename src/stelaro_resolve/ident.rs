use crate::stelaro_ast::NodeId;
use crate::stelaro_common::{Ident, Span, Symbol, DUMMY_SPAN};
use crate::stelaro_resolve::{BindingKey, LexicalScopeBinding, NameBindingKind, PathResult};
use crate::stelaro_sir::def::{Namespace::{self}, PerNS};

use super::{late::{Scope, Segment}, Module, NameBinding, Resolver};

/// 名前解決の試行結果が、その時点で最終的なものと見なせるか、
/// それとも後続の処理によって変化する可能性があるかを示します。
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Determinacy {
    /// 名前解決の試行が完了し、その結果 (成功または失敗) が
    /// この段階において最終的であると判断されたことを示します。
    Determined,

    /// 名前解決の試行がまだ完了しておらず、現在の結果が最終決定ではないことを示します。
    Undetermined,
}

impl Determinacy {
    fn determined(determined: bool) -> Determinacy {
        if determined { Determinacy::Determined } else { Determinacy::Undetermined }
    }
}


impl<'ra, 'tcx> Resolver<'ra, 'tcx> {
    pub fn visit_scopes<T>(
        &mut self,
        ns: Namespace,
        parent_module: &Module<'ra>,
        mut visitor: impl FnMut(&mut Self, Scope<'ra>) -> Option<T>,
    ) -> Option<T> {
        todo!()
    }

    pub fn resolve_ident_in_module(
        &mut self,
        module: &Module<'ra>,
        ident: Ident,
        ns: Namespace,
        parent_module: &Module<'ra>,
        node_id: Option<NodeId>,
        ignore_binding: Option<NameBinding<'ra>>,
    ) -> Result<NameBinding<'ra>, Determinacy>{
        todo!()
    }

    pub fn resolve_ident_in_lexical_scope(
        &mut self,
        ident: Ident,
        ns: Namespace,
        parent_module: &Module<'ra>,
        node_id: Option<NodeId>,
        scopes: &[Scope<'ra>],
        ignore_binding: Option<NameBinding<'ra>>,
    ) -> Option<LexicalScopeBinding<'ra>> {
        todo!()
    }

    pub fn resolve_path(
        &mut self,
        path: &[Segment],
        opt_ns: Option<Namespace>,
        node_id: Option<NodeId>,
        path_span: Option<Span>,
        parent_module: &Module<'ra>,
        ignore_binding: Option<NameBinding<'ra>>,
    ) -> PathResult<'ra> {
        self.resolve_path_with_scopes(
            path,
            opt_ns,
            node_id,
            path_span,
            parent_module,
            None,
            ignore_binding,
        )
    }

    pub fn resolve_path_with_scopes(
        &mut self,
        path: &[Segment],
        opt_ns: Option<Namespace>,
        node_id: Option<NodeId>,
        path_span: Option<Span>,
        parent_module: &Module<'ra>,
        scopes: Option<&PerNS<Vec<Scope<'ra>>>>,
        ignore_binding: Option<NameBinding<'ra>>,
    ) -> PathResult<'ra> {
        let mut module = None;
        let mut second_binding: Option<NameBinding> = None;

        for (segment_idx, Segment { ident, id, .. }) in path.iter().enumerate() {
            let is_last = segment_idx + 1 == path.len();
            let ns_to_resolve = if is_last {
                opt_ns.unwrap_or(Namespace::TypeNS)
            } else {
                Namespace::TypeNS
            };

            let binding = if let Some(module) = module {
                self.resolve_ident_in_module(
                    module,
                    *ident,
                    ns_to_resolve,
                    parent_module,
                    node_id,
                    ignore_binding,
                )
            } else if let Some(scopes) = scopes
                && let Some(_) = opt_ns
            {
                match self.resolve_ident_in_lexical_scope(
                    *ident,
                    ns_to_resolve,
                    parent_module,
                    node_id,
                    &scopes[ns_to_resolve],
                    ignore_binding,
                ) {
                    // 現在のスコープで利用可能なアイテムやモジュールが見つかった
                    Some(LexicalScopeBinding::Item(binding)) => Ok(binding),
                    // ローカル変数を見つけた
                    Some(LexicalScopeBinding::Res(res)) => {
                        return PathResult::NonModule(res);
                    }
                    _ => Err(Determinacy::determined(node_id.is_some())),
                }
            } else {
                self.resolve_initial_module_segment(
                    *ident,
                    parent_module,
                    ns_to_resolve,
                    node_id,
                )
            };


            match binding {
                Ok(_) => todo!(),
                Err(_) => todo!(),
            }

        }

        todo!()
    }

    fn resolve_initial_module_segment(
        &mut self,
        ident: Ident,
        parent_module: &Module<'ra>,
        ns: Namespace,
        node_id: Option<NodeId>,
    ) -> Result<NameBinding<'ra>, Determinacy> {
        todo!()
    }
}
