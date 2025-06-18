use crate::stelaro_common::{Ident, Span};
use crate::stelaro_diagnostic::diag::{Diag, DiagCtxtHandle, ErrorEmitted};
use crate::stelaro_resolve::{Segment, LexicalScopeBinding, late::Scope};
use crate::stelaro_sir::def::{Namespace::{self, ValueNS, TypeNS}, PerNS, Res};

use super::{Module, ModuleKind, NameBinding, Resolver};


impl<'ra, 'tcx> Resolver<'ra, 'tcx> {
    pub fn dcx(&self) -> DiagCtxtHandle<'tcx> {
        self.tcx.dcx()
    }

    pub fn report_conflict(
        &mut self,
        parent: Module<'_>,
        ident: Ident,
        ns: Namespace,
        new_binding: NameBinding<'ra>,
        old_binding: NameBinding<'ra>,
    ) {
        // 名前衝突が起こった二個目の名前に対して診断する
        if old_binding.span.start > new_binding.span.start {
            // new_binding, old_binding を入れ替える
            return self.report_conflict(parent, ident, ns, old_binding, new_binding);
        }

        let container_dscr = match parent.kind {
            ModuleKind::Def(kind, _, _) => kind.descr_ja(parent.def_id()),
            ModuleKind::Block => "ブロック",
        };

        let (name, span) =
            (ident.name, self.tcx.sess.source_map().truncate_span_to_item_header(new_binding.span));

        if self.name_already_seen.get(&name) == Some(&span) {
            return;
        }

        let name_str = name.as_str();

        let old_kind_dscr = match (ns, old_binding.module()) {
            (ValueNS, _) => "値",
            (TypeNS, Some(module)) if module.is_normal() => "モジュール",
            (TypeNS, _) => "型",
        };

        let mut diag = DiagsResolver::name_defined_multiple_time(
            self.dcx(),
            name_str,
            ns.descr_ja(),
            container_dscr,
            span
        );

        diag.set_label(
            self.tcx.sess
                .source_map()
                .truncate_span_to_item_header(old_binding.span),
            format!(
                "既に{}名前空間内に `{}` はここで定義されています",
                old_kind_dscr,
                name_str,
            )
        );

        diag.emit();
        self.name_already_seen.insert(name, span);
    }

    pub fn report_path_resolution_error(
        &mut self,
        path: &[Segment],
        opt_ns: Option<Namespace>,
        parent_module: &Module<'ra>,
        scopes: Option<&PerNS<Vec<Scope<'ra>>>>,
        ignore_binding: Option<NameBinding<'ra>>,
        module: Option<Module<'ra>>,
        failed_segment_idx: usize,
        ident: Ident,
    ) -> String {
        let is_last = failed_segment_idx == path.len() - 1;
        let ns = if is_last { opt_ns.unwrap_or(TypeNS) } else { TypeNS };
        if failed_segment_idx > 0 {
            let parent = path[failed_segment_idx - 1].ident.name;
            let parent = format!("`{parent}`");

            let mut msg = format!("`{ident}` は {parent} の中で見つかりませんでした");
            let ns_to_try = if ns == ValueNS { TypeNS } else { ValueNS };
            let binding = if let Some(module) = module {
                self.resolve_ident_in_module(
                    &module,
                    ident,
                    ns_to_try,
                    parent_module,
                    None,
                    ignore_binding,
                )
                .ok()
            } else if let Some(scopes) = scopes
                && let Some(TypeNS | ValueNS) = opt_ns
            {
                match self.resolve_ident_in_lexical_scope(
                    ident,
                    ns_to_try,
                    parent_module,
                    None,
                    &scopes[ns_to_try],
                    ignore_binding,
                ) {
                    Some(LexicalScopeBinding::Item(binding)) => Some(binding),
                    _ => None,
                }
            } else {
                self.resolve_ident_in_ambience(
                    ident,
                    parent_module,
                    ns_to_try,
                    None,
                    false,
                    ignore_binding,
                )
                .ok()
            };
            if let Some(binding) = binding {
                let mut found = |what| {
                    msg = format!(
                        "{}を期待していましたが、{} `{}` が {} で見つかりました",
                        ns.descr_ja(),
                        what,
                        ident,
                        parent
                    )
                };
                if binding.module().is_some() {
                    found("モジュール")
                } else {
                    match binding.res() {
                        Res::Def(kind, id) => found(kind.descr_ja(id)),
                        _ => found(ns_to_try.descr_ja()),
                    }
                }
            };

            msg
        } else if let Ok(binding) = self.resolve_ident_in_ambience(
            ident,
            parent_module,
            ns,
            None,
            false,
            ignore_binding
        ) {
            let descr = binding.res().descr_ja();
            format!("{descr} `{ident}` はモジュール名ではありません")
        } else {
            format!("`{ident}` を解決することができませんでした")
        }
    }
}

// impl<'a> PathSource<'a> {
//     fn error_code(self, has_unexpected_resolution: bool) -> ErrorCode {
//         match (self, has_unexpected_resolution) {
//             (PathSource::Type, true) => todo!(),
//             (PathSource::Type, false) => todo!(),
//             (PathSource::Expr(..), true) => todo!(),
//             (PathSource::Expr(..), false) => todo!(),
//         }
//     }
// }

pub struct DiagsResolver;

impl<'dcx> DiagsResolver {
    pub fn name_defined_multiple_time(
        dcx: DiagCtxtHandle<'dcx>,
        name: &str,
        ns: &str,
        container: &str,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(ErrorCode::NameDefinedMultipleTime.into());
        diag.set_message(format!("名前 `{name}` の重複した定義"));
        diag.set_label(
            span,
            format!("`{name}` は重複して定義されています\n
            `{name}` はこの{container}の{ns}名前空間で一回だけ定義することができます")
        );

        diag
    }

    pub fn duplicate_identifier_in_parameter_list(
        dcx: DiagCtxtHandle<'dcx>,
        span: Span,
        ident: Ident,
    ) -> Diag<'dcx, ErrorEmitted> {
        let name = ident.name.as_str();
        let mut diag = dcx.struct_err(span);
        diag.set_code(ErrorCode::DuplicateIdentifierInParameterList.into());
        diag.set_message(format!("パラメーター `{name}` の重複した定義"));
        diag.set_label(
            span,
            format!("`{name}` は引数リストの中で重複して定義されています")
        );

        diag
    }
}

#[repr(i32)]
enum ErrorCode {
    NameDefinedMultipleTime = 300,
    DuplicateIdentifierInParameterList = 301,
}

impl From<ErrorCode> for i32 {
    fn from(value: ErrorCode) -> Self {
        value as i32
    }
}
