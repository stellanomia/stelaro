use crate::stelaro_common::Ident;
use crate::stelaro_resolve::{
    Determinacy, Finalize, LexicalScopeBinding, ModuleKind,
    PathResult, Segment, late::{Scope, ScopeKind}, Module,
    NameBinding, BindingKey, Resolver
};
use crate::stelaro_sir::def::{Namespace::{self, ValueNS, TypeNS}, PerNS, Res};


impl<'ra, 'tcx> Resolver<'ra, 'tcx> {

    /// これは、現在のレキシカルスコープ内で、名前空間 `ns` の識別子 `ident` を解決します。
    /// より具体的には、スコープの階層を上にたどり、`ident` を定義している最初のスコープでの
    /// 束縛を返します (どのスコープもそれを定義していない場合は `None` を返します)。
    ///
    /// ブロックのアイテムは、そのアイテムがブロック内のどこで定義されているかに関わらず、
    /// スコープ階層において、そのローカル変数よりも上にあります。例えば、
    ///
    /// fn f() {
    ///    g(); // スコープ内にまだローカル変数がないため、これはアイテムを参照するように解決されます。
    ///    let g = 1;
    ///    g + g; // これはアイテムをシャドーイングするため、ローカル変数 `g` に解決されます。
    /// }
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
        let mut module;
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
            finalize,
            finalize.is_some(),
            ignore_binding,
        )
        .ok()
        .map(LexicalScopeBinding::Item)
    }

    pub(crate) fn maybe_resolve_ident_in_module(
        &mut self,
        module: &Module<'ra>,
        ident: Ident,
        ns: Namespace,
        parent_scope: &Module<'ra>,
    ) -> Result<NameBinding<'ra>, Determinacy> {
        self.resolve_ident_in_module(
            module,
            ident,
            ns,
            parent_scope,
            None,
            None
        )
    }

    pub fn resolve_ident_in_module(
        &mut self,
        module: &Module<'ra>,
        ident: Ident,
        ns: Namespace,
        _parent_module: &Module<'ra>,
        finalize: Option<Finalize>,
        ignore_binding: Option<NameBinding<'ra>>,
    ) -> Result<NameBinding<'ra>, Determinacy> {
        let key = BindingKey::new(ident, ns);
        let res = self.resolution(*module, key)
            .try_borrow_mut()
            .map_err(|_| Determinacy::Determined)?;

        // プライマリな束縛が使えない場合を探す
        let binding = if res.binding == ignore_binding {
            None
        } else {
            res.binding
        };

        if finalize.is_some() {
            let Some(binding) = binding else {
                return Err(Determinacy::Determined);
            };

            return Ok(binding);
        } else if let Some(binding) = binding {
            return Ok(binding);
        }

        Err(Determinacy::Determined)
    }

    pub fn resolve_ident_in_ambience(
        &mut self,
        ident: Ident,
        parent_module: &Module<'ra>,
        ns: Namespace,
        finalize: Option<Finalize>,
        force: bool,
        ignore_binding: Option<NameBinding<'ra>>,
    ) -> Result<NameBinding<'ra>, Determinacy> {
        assert!(force || finalize.is_none()); // `finalize` は `force` を意味する

        let mut current_module = Some(*parent_module);

        while let Some(module_to_search) = current_module {
            let result = self.resolve_ident_in_module(
                &module_to_search,
                ident,
                ns,
                parent_module,
                finalize,
                ignore_binding,
            );

            match result {
                Ok(binding) => {
                    return Ok(binding);
                }
                Err(Determinacy::Determined) => {
                    // このモジュールには無かったので、ループを継続して親モジュールを探索する。
                }
                Err(Determinacy::Undetermined) => {
                    return Err(Determinacy::Undetermined);
                }
            }

            // 親モジュールへ移動
            current_module = module_to_search.parent;
        }

        // ステロのルートまで遡っても見つからなかった
        Err(Determinacy::Determined)
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

        for (segment_idx, Segment { ident, id, .. }) in path.iter().enumerate() {
            let record_segment_res = |this: &mut Self, res| {
                if finalize.is_some()
                    && let Some(id) = id
                {
                    this.record_res(*id, res);
                }
            };

            let is_last = segment_idx + 1 == path.len();
            let ns = if is_last {
                opt_ns.unwrap_or(Namespace::TypeNS)
            } else {
                Namespace::TypeNS
            };

            let binding = if let Some(ref module) = module {
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
                        record_segment_res(self, res);
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
                    finalize.is_some(),
                    ignore_binding,
                )
            };

            match binding {
                Ok(binding) => {
                    let res = binding.res();

                    if is_last {
                        return PathResult::NonModule(res);
                    }

                    if let Some(next_module) = binding.module() {
                        record_segment_res(self, res);
                        module = Some(next_module);
                    } else if res == Res::Err {
                        return PathResult::NonModule(Res::Err);
                    } else {
                        return PathResult::Failed {
                            span: ident.span,
                            is_error_from_last_segment: is_last,
                            segment_name: ident.name,
                            module,
                            label: format!(
                                "`{ident}` は{}で、モジュールではありません",
                                res.descr_ja()
                            )
                        };
                    }
                },
                Err(Determinacy::Undetermined) => return PathResult::Indeterminate,
                Err(Determinacy::Determined) => {
                    return PathResult::Failed {
                        span: ident.span,
                        is_error_from_last_segment: is_last,
                        module,
                        segment_name: ident.name,
                        label: self.report_path_resolution_error(
                            path,
                            opt_ns,
                            parent_module,
                            scopes,
                            ignore_binding,
                            module,
                            segment_idx,
                            *ident,
                        ),
                    };
                },
            }

        }

        PathResult::Module(match module {
            Some(module) => module,
            _ => panic!("resolve_path: 空でないパス `{:?}` にはモジュールがありません", path),
        })
    }
}
