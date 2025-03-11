use crate::{stelaro_ast::token::Token, stelaro_common::span::Span, stelaro_diagnostic::diag::{Diag, DiagCtxtHandle, ErrorEmitted}};

struct DiagsParser;

impl<'dcx> DiagsParser {
    pub fn unexpected_character (
        dcx: DiagCtxtHandle<'dcx>,
        unexpected: Token,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(200);
        diag.set_message("予期しないトークン".to_string());
        diag.set_label(span, format!("不正なトークン`{}`が入力されました", unexpected));

        diag
    }

}