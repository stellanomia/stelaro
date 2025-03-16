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

    pub fn missing_operator (
        dcx: DiagCtxtHandle<'dcx>,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(202);
        diag.set_message("不足した演算子".to_string());
        diag.set_label(span, "式と式の間に演算子がありません".to_string());
        diag.set_help("演算子(e.g., `+`, `-`)か、`;`を追加してください".to_string());

        diag
    }

    pub fn expect_expression (
        dcx: DiagCtxtHandle<'dcx>,
        unexpected: Token,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(203);
        diag.set_message("無効な式".to_string());
        diag.set_label(span, format!("`{}`は式ではありません", unexpected));
        diag.set_help("これを削除するか、間に値を追加してください".to_string());

        diag
    }

    pub fn prefix_increment (
        dcx: DiagCtxtHandle<'dcx>,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(204);
        diag.set_message("前置インクリメント".to_string());
        diag.set_label(span, "steraloは前置インクリメント演算子をもちません".to_string());
        diag.set_help("`lhs += 1;`を使用してください".to_string());

        diag
    }

}