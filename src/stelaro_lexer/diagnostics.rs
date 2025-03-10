use crate::{stelaro_common::span::Span, stelaro_diagnostic::diag::{Diag, DiagCtxtHandle, ErrorEmitted}};

pub struct DiagsLexer;

impl<'dcx> DiagsLexer {
    pub fn unexpected_character (
        dcx: DiagCtxtHandle<'dcx>,
        unexpected: char,
        span: Span) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(100);
        diag.set_message("予期しない文字".to_string());
        diag.set_label(span, format!("不正な文字`{}`が入力されました", unexpected));

        diag
    }
}
