use crate::{stelaro_ast::token::{Token, TokenKind}, stelaro_common::span::Span, stelaro_diagnostic::diag::{Diag, DiagCtxtHandle, ErrorEmitted}};

pub struct DiagsParser;

impl<'dcx> DiagsParser {
    pub fn unexpected_token (
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

    pub fn unexpected_token_with_expected (
        dcx: DiagCtxtHandle<'dcx>,
        unexpected: TokenKind,
        expected: TokenKind,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(200);
        diag.set_message("予期しないトークン".to_string());
        diag.set_label(span, format!("`{}`を期待していましたが、`{}` は無効な入力です", expected, unexpected));

        diag
    }

    pub fn chained_comparison (
        dcx: DiagCtxtHandle<'dcx>,
        op1: Span,
        op2: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(op1.merge(&op2));
        diag.set_code(201);
        diag.set_message("連鎖した比較演算子".to_string());
        diag.set_label(op1, "この比較は無効です".to_string());
        diag.set_label(op2, "比較演算子が連鎖しています".to_string());

        diag
    }

}