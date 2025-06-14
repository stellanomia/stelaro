use crate::stelaro_common::Ident;
use crate::stelaro_resolve::{
    Determinacy, Finalize, LexicalScopeBinding, ModuleKind,
    PathResult, Segment, late::{Scope, ScopeKind}, Module,
    NameBinding, Resolver
};
use crate::stelaro_sir::def::{Namespace::{self, ValueNS, TypeNS}, PerNS};


impl<'ra, 'tcx> Resolver<'ra, 'tcx> {
    pub fn resolve_ident_in_module(
        &mut self,
        module: &Module<'ra>,
        ident: Ident,
        ns: Namespace,
        parent_module: &Module<'ra>,
        finalize: Option<Finalize>,
        ignore_binding: Option<NameBinding<'ra>>,
    ) -> Result<NameBinding<'ra>, Determinacy>{
        todo!()
    }

    /// これは、現在のレキシカルスコープ内で、名前空間 `ns` の識別子 `ident` を解決します。
    /// より具体的には、スコープの階層を上にたどり、`ident` を定義している最初のスコープでの
    /// 束縛を返します (どのスコープもそれを定義していない場合は `None` を返します)。
    ///
    /// ブロックのアイテムは、そのアイテムがブロック内のどこで定義されているかに関わらず、
    /// スコープ階層において、そのローカル変数よりも上にあります。例えば、
    /// ```
    /// fn f() {
    ///    g(); // スコープ内にまだローカル変数がないため、これはアイテムを参照するように解決されます。
    ///    let g = 1;
    ///    g + g; // これはアイテムをシャドーイングするため、ローカル変数 `g` に解決されます。
    /// }
    /// ```
    pub fn resolve_ident_in_lexical_scope(
        &mut self,
        ident: Ident,
        ns: Namespace,
        parent_module: &Module<'ra>,
        finalize: Option<Finalize>,
        scopes: &[Scope<'ra>],
        ignore_binding: Option<NameBinding<'ra>>,
    ) -> Option<LexicalScopeBinding<'ra>> {

        // スコープスタックを逆順に辿る
        let mut module = self.graph_root;
        for scope in scopes.iter().rev() {
            if let Some(res) = scope.bindings.get(&ident) {
                // 識別子は型パラメータまたはローカル変数に解決される。
                return Some(
                    LexicalScopeBinding::Res(
                        *res
                    )
                );
            }

            module = match scope.kind {
                ScopeKind::Module(module) => module,
                _ => continue,
            };

            match module.kind {
                ModuleKind::Block => {}, // ブロックは通過して見ることができる
                _ => break,
            }

            let item = self.resolve_ident_in_module(
                &module,
                ident,
                ns,
                parent_module,
                finalize,
                ignore_binding
            );

            if let Ok(binding) = item {
                return Some(LexicalScopeBinding::Item(binding));
            }
        }

        self.resolve_ident_in_ambience(
            ident,
            parent_module,
            ns,
            finalize
        )
        .ok()
        .map(LexicalScopeBinding::Item)
    }

    pub fn resolve_ident_in_ambience(
        &mut self,
        ident: Ident,
        parent_module: &Module<'ra>,
        ns: Namespace,
        finalize: Option<Finalize>,
    ) -> Result<NameBinding<'ra>, Determinacy> {
        todo!()
    }

    pub fn resolve_path(
        &mut self,
        path: &[Segment],
        opt_ns: Option<Namespace>,
        finalize: Option<Finalize>,
        parent_module: &Module<'ra>,
        ignore_binding: Option<NameBinding<'ra>>,
    ) -> PathResult<'ra> {
        self.resolve_path_with_scopes(
            path,
            opt_ns,
            finalize,
            parent_module,
            None,
            ignore_binding,
        )
    }

    pub fn resolve_path_with_scopes(
        &mut self,
        path: &[Segment],
        opt_ns: Option<Namespace>,
        finalize: Option<Finalize>,
        parent_module: &Module<'ra>,
        scopes: Option<&PerNS<Vec<Scope<'ra>>>>,
        ignore_binding: Option<NameBinding<'ra>>,
    ) -> PathResult<'ra> {
        let mut module = None;
        let mut second_binding: Option<NameBinding> = None;

        for (segment_idx, Segment { ident, id, .. }) in path.iter().enumerate() {
            let is_last = segment_idx + 1 == path.len();
            let ns = if is_last {
                opt_ns.unwrap_or(Namespace::TypeNS)
            } else {
                Namespace::TypeNS
            };

            let binding = if let Some(module) = module {
                self.resolve_ident_in_module(
                    module,
                    *ident,
                    ns,
                    parent_module,
                    finalize,
                    ignore_binding,
                )
            } else if let Some(scopes) = scopes
                && let Some(ValueNS | TypeNS) = opt_ns
            {
                match self.resolve_ident_in_lexical_scope(
                    *ident,
                    ns,
                    parent_module,
                    finalize,
                    &scopes[ns],
                    ignore_binding,
                ) {
                    // 現在のスコープで利用可能なアイテムやモジュールが見つかった
                    Some(LexicalScopeBinding::Item(binding)) => Ok(binding),
                    // ローカル変数を見つけた
                    Some(LexicalScopeBinding::Res(res)) => {
                        return PathResult::NonModule(res);
                    }
                    _ => Err(Determinacy::determined(finalize.is_some())),
                }
            } else {
                self.resolve_ident_in_ambience(
                    *ident,
                    parent_module,
                    ns,
                    finalize,
                )
            };


            match binding {
                Ok(_) => todo!(),
                Err(_) => todo!(),
            }

        }

        todo!()
    }
}
