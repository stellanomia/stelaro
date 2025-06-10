use crate::stelaro_common::{Ident, Span};
use crate::stelaro_diagnostic::diag::{Diag, DiagCtxtHandle, ErrorEmitted};
use crate::stelaro_sir::def::Namespace;

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
            (Namespace::ValueNS, _) => "値",
            (Namespace::TypeNS, Some(module)) if module.is_normal() => "モジュール",
            (Namespace::TypeNS, _) => "型",
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
}

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
