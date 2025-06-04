use crate::stelaro_ast::NodeId;
use crate::stelaro_common::{Ident, Span, Symbol, DUMMY_SPAN};
use crate::stelaro_resolve::{LexicalScopeBinding, NameBindingKind, PathResult};
use crate::stelaro_sir::def::{Namespace::{self}, PerNS};

use super::{late::{Scope, Segment}, Module, NameBinding, Resolver};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Determinacy {
    Determined,
    Undetermined,
}

impl Determinacy {
    fn determined(determined: bool) -> Determinacy {
        if determined { Determinacy::Determined } else { Determinacy::Undetermined }
    }
}


impl<'ra, 'tcx> Resolver<'ra, 'tcx> {

    pub fn resolve_ident_in_module(
        &mut self,
        module: Module<'ra>,
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
        let mut current_module_candidate = *parent_module;

        for (segment_idx, segment) in path.iter().enumerate() {
            let ident = segment.ident;
            let is_last = segment_idx + 1 == path.len();
            let ns_to_resolve = if is_last {
                opt_ns.unwrap_or(Namespace::TypeNS)
            } else {
                Namespace::TypeNS
            };

            let binding = if let Some(module) = module {
                self.resolve_ident_in_module(
                    module,
                    ident,
                    ns_to_resolve,
                    parent_module,
                    node_id,
                    ignore_binding,
                )
            } else if let Some(scopes) = scopes
                && let Some(_) = opt_ns
            {
                match self.resolve_ident_in_lexical_scope(
                    ident,
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
                todo!();
            };


            match binding {
                Ok(_) => todo!(),
                Err(_) => todo!(),
            }

        } // パスセグメントのループ終了

        // ループが正常に完了した場合、すべてのセグメントが処理され、
        // パスは最終的な `current_module_candidate` に解決されたことを意味します。
        PathResult::Module(current_module_candidate)
    }

    pub fn resolve_path_with_scopes_(
        &mut self,
        path: &[Segment], // stelaro_resolve::late::Segment のスライス
        opt_ns: Option<Namespace>, // 解決対象の名前空間
        node_id: Option<NodeId>,   // パス全体の AST ノード ID
        path_span: Option<Span>,   // パス全体の Span
        parent_module: &Module<'ra>, // このパス解決が開始されるモジュール
        scopes: Option<&PerNS<Vec<Scope<'ra>>>>, // 利用可能なレキシカルスコープ
        ignore_binding: Option<NameBinding<'ra>>, // 無視する特定の束縛
    ) -> PathResult<'ra> {
        if path.is_empty() {
            let span = path_span.unwrap_or_else(|| DUMMY_SPAN);
            // 空のパス: 型コンテキストなら現在のモジュール、値コンテキストならエラーが一般的
            return if opt_ns.map_or(true, |ns| ns == Namespace::TypeNS) {
                PathResult::Module(*parent_module)
            } else {
                PathResult::Failed {
                    span,
                    label: "empty path cannot be resolved to a value".to_string(),
                    is_error_from_last_segment: true,
                    module: Some(*parent_module),
                    segment_name: Symbol::intern(""),
                    error_implied_by_parse_error: true,
                }
            };
        }

        // `resolved_module_context` は、直前のセグメントが解決したモジュールを保持する。
        // ループの最初のイテレーションでは None。
        let mut resolved_module_context: Option<Module<'ra>> = None;
        // `current_search_module` は、現在のセグメントを解決しようとしているモジュール。
        let mut current_search_module = *parent_module;


        for (segment_idx, segment) in path.iter().enumerate() {
            let ident = segment.ident;
            let is_last_segment = segment_idx + 1 == path.len();

            // 解決対象の名前空間: 最後のセグメントなら opt_ns (デフォルトTypeNS)、それ以外なら TypeNS
            let ns_to_resolve = if is_last_segment {
                opt_ns.unwrap_or(Namespace::TypeNS)
            } else {
                Namespace::TypeNS
            };

            // 現在のセグメントのASTノードID (あれば)。エラー報告やDeterminacyの判断に使う。
            let current_segment_ast_node_id = segment.id.or(node_id);

            let binding_result: Result<NameBinding<'ra>, Determinacy>;

            if let Some(prev_module) = resolved_module_context {
                // 2番目以降のセグメント: 前のセグメントが解決したモジュール内で探す
                current_search_module = prev_module;
                binding_result = self.resolve_ident_in_module(
                    prev_module,
                    ident,
                    ns_to_resolve,
                    parent_module, // 元の親モジュール(エラー報告用などのコンテキストとして)
                    current_segment_ast_node_id,
                    ignore_binding,
                );
            } else {
                // 最初のセグメント
                current_search_module = *parent_module; // デフォルトの検索コンテキスト
                // スケルトンの条件 `scopes.is_some() && opt_ns.is_some()` に従う
                if scopes.is_some() && opt_ns.is_some() {
                    match self.resolve_ident_in_lexical_scope(
                        ident,
                        ns_to_resolve,
                        parent_module,
                        current_segment_ast_node_id,
                        &scopes.unwrap()[ns_to_resolve], // .unwrap() は scopes.is_some() で保護
                        ignore_binding,
                    ) {
                        Some(LexicalScopeBinding::Item(item_binding)) => {
                            binding_result = Ok(item_binding);
                        }
                        Some(LexicalScopeBinding::Res(local_res)) => {
                            // レキシカルスコープでローカル変数などが見つかった
                            if !is_last_segment {
                                return PathResult::Failed {
                                    span: ident.span,
                                    label: format!(
                                        "expected module, but `{}` (a {}) was found in a lexical scope before the end of the path",
                                        ident.name.as_str(), local_res.descr(),
                                    ),
                                    is_error_from_last_segment: false,
                                    module: Some(*parent_module), // レキシカルスコープはparent_moduleに関連
                                    segment_name: ident.name,
                                    error_implied_by_parse_error: false, // ローカル解決は通常確定的
                                };
                            }
                            return PathResult::NonModule(local_res);
                        }
                        None => {
                            // スケルトンの `_ => Err(Determinacy::determined(node_id.is_some()))` に基づく。
                            // レキシカルスコープで試みて見つからなかった場合、エラーとする。
                            // (モジュールスコープへのフォールバックはしないという解釈)
                            binding_result = Err(Determinacy::determined(current_segment_ast_node_id.is_some()));
                        }
                    }
                } else {
                    // 最初のセグメントで、レキシカルスコープを試みない場合 (スケルトンの else { todo!() } 部分)
                    // parent_module で解決を試みる
                    binding_result = self.resolve_ident_in_module(
                        *parent_module,
                        ident,
                        ns_to_resolve,
                        parent_module,
                        current_segment_ast_node_id,
                        ignore_binding,
                    );
                }
            }

            match binding_result {
                Ok(name_binding) => {
                    if Some(name_binding) == ignore_binding && current_segment_ast_node_id.is_some() {
                        return PathResult::Failed {
                            span: ident.span,
                            label: format!(
                                "resolution of `{}` in {} is specified to be ignored",
                                ident.name.as_str(), self.module_path_for_display(current_search_modules)
                            ),
                            is_error_from_last_segment: is_last_segment,
                            module: Some(current_search_module),
                            segment_name: ident.name,
                            error_implied_by_parse_error: false, // 明示的な無視は確定的エラー
                        };
                    }

                    match name_binding.kind {
                        NameBindingKind::Module(m) => {
                            resolved_module_context = Some(m); // 次のセグメントはこのモジュールで解決
                            if is_last_segment {
                                return PathResult::Module(m);
                            }
                            // パスが続くので次のセグメントへ
                        }
                        NameBindingKind::Res(res_val) => {
                            // 非モジュールに解決された
                            if !is_last_segment {
                                // パスの途中ならエラー
                                return PathResult::Failed {
                                    span: ident.span,
                                    label: format!(
                                        "expected module after segment `{}`, but it resolved to {} in {}",
                                        ident.name.as_str(), res_val.descr(), self.module_path_for_display(current_search_module)
                                    ),
                                    is_error_from_last_segment: false,
                                    module: Some(current_search_module),
                                    segment_name: ident.name,
                                    error_implied_by_parse_error: false,
                                };
                            }
                            // パスの最後ならOK
                            return PathResult::NonModule(res_val);
                        }
                    }
                }
                Err(determinacy) => {
                    // 解決に失敗
                    let is_parse_error_implied = determinacy == Determinacy::Undetermined && current_segment_ast_node_id.is_none();

                    if is_parse_error_implied {
                        // current_segment_ast_node_id がなく、かつ Determinacy が Undetermined の場合、
                        // より不確定性が高い (例: パースエラーに起因する)。
                        return PathResult::Indeterminate;
                    }

                    return PathResult::Failed {
                        span: ident.span,
                        label: format!(
                            "cannot find {} `{}` in {}",
                            if ns_to_resolve == Namespace::TypeNS { "type or module" } else { "value" },
                            ident.name.as_str(),
                            self.module_path_for_display(current_search_module)
                        ),
                        is_error_from_last_segment: is_last_segment,
                        module: Some(current_search_module),
                        segment_name: ident.name,
                        // determinacy が Undetermined で、かつ node_id が *ある* 場合も、
                        // パースエラーを示唆する未確定性と解釈できる。
                        error_implied_by_parse_error: determinacy == Determinacy::Undetermined,
                    };
                }
            }
        } // パスセグメントのループ終了

        // ループが正常に完了し、return されなかった場合。
        // これは、パス全体がモジュールとして解決され、最後のセグメントの処理で
        // `is_last_segment` が true になり、`PathResult::Module(m)` が返されるべきだったが、
        // 何らかの理由でそうなっていない場合。
        // または、空のパス (先頭で処理済み) 以外でこの地点に来るのは通常は論理エラー。
        // `resolved_module_context` に最後に解決されたモジュールが入っているはず。
        if let Some(final_module) = resolved_module_context {
            PathResult::Module(final_module)
        } else {
            // ループを抜けたのに resolved_module_context が None のまま。
            // これは、パスが1セグメントで、それがモジュールとして解決されなかった場合に起こりうる。
            // しかし、その場合はループ内の Err(determinacy) で Failed/Indeterminate が返るはず。
            // したがって、このフォールバックは通常到達すべきではない。
            let last_segment_ident = path.last().expect("path is checked non-empty at the beginning").ident;
            PathResult::Failed {
                span: path_span.unwrap_or(last_segment_ident.span),
                label: format!(
                    "internal error: path `{}` resolution ended inconclusively",
                    path.iter().map(|s| s.ident.name.as_str()).collect::<Vec<_>>().join("::")
                ),
                is_error_from_last_segment: true,
                module: Some(current_search_module), // ループで最後に使われた検索コンテキスト
                segment_name: last_segment_ident.name,
                error_implied_by_parse_error: true, // 内部エラーはパースエラー由来と見なす
            }
        }
    }
}
