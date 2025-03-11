use crate::{stelaro_common::span::Span, stelaro_diagnostic::diag::{Diag, DiagCtxtHandle, ErrorEmitted}};

pub struct DiagsLexer;

impl<'dcx> DiagsLexer {
    pub fn unexpected_character (
        dcx: DiagCtxtHandle<'dcx>,
        unexpected: char,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(100);
        diag.set_message("予期しない文字".to_string());
        diag.set_label(span, format!("不正な文字`{}`が入力されました", unexpected));

        diag
    }

    pub fn invalid_float_format (
        dcx: DiagCtxtHandle<'dcx>,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(101);
        diag.set_message("無効な浮動小数点数の表記".to_string());
        diag.set_label(span, "二個目の`.`が見つかりました".to_string());

        diag
    }

    pub fn missing_fractional_part (
        dcx: DiagCtxtHandle<'dcx>,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(102);
        diag.set_message("小数部の欠落".to_string());
        diag.set_label(span, "小数部が必要です".to_string());

        diag
    }

    pub fn invalid_escape_sequence (
        dcx: DiagCtxtHandle<'dcx>,
        invalid_ch: char,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(103);
        diag.set_message("無効なエスケープシーケンス".to_string());
        diag.set_label(span, format!("`{}`は無効なエスケープシーケンス文字です", invalid_ch));

        diag
    }

    pub fn unterminated_string_literal (
        dcx: DiagCtxtHandle<'dcx>,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(104);
        diag.set_message("閉じられていない文字列".to_string());
        diag.set_label(span, "文字列が閉じられていません".to_string());

        diag
    }
}
