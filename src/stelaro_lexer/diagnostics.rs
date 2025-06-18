use crate::stelaro_common::Span;
use crate::stelaro_diagnostic::{Diag, DiagCtxtHandle, ErrorEmitted};

pub struct DiagsLexer;

impl<'dcx> DiagsLexer {
    pub fn unexpected_character (
        dcx: DiagCtxtHandle<'dcx>,
        unexpected: char,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(ErrorCode::UnexpectedCharacter.into());
        diag.set_message("予期しない文字".to_string());
        diag.set_label(span, format!("不正な文字`{}`が入力されました", unexpected));

        diag
    }

    pub fn invalid_float_format (
        dcx: DiagCtxtHandle<'dcx>,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(ErrorCode::InvalidFloatFormat.into());
        diag.set_message("無効な浮動小数点数の表記".to_string());
        diag.set_label(span, "二個目の`.`が見つかりました".to_string());

        diag
    }

    pub fn missing_fractional_part (
        dcx: DiagCtxtHandle<'dcx>,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(ErrorCode::MissingFractionalPart.into());
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
        diag.set_code(ErrorCode::InvalidEscapeSequence.into());
        diag.set_message("無効なエスケープシーケンス".to_string());
        diag.set_label(span, format!("`{}`は無効なエスケープシーケンス文字です", invalid_ch));

        diag
    }

    pub fn unterminated_string_literal (
        dcx: DiagCtxtHandle<'dcx>,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(ErrorCode::UnterminatedStringLiteral.into());
        diag.set_message("閉じられていない文字列リテラル".to_string());
        diag.set_label(span, "文字列が閉じられていません".to_string());

        diag
    }

    // 文字リテラルを期待したが、改行が見つかった
    pub fn unexpected_quote (
        dcx: DiagCtxtHandle<'dcx>,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(ErrorCode::UnexpectedQuote.into());
        diag.set_message("予期しない`'`".to_string());
        diag.set_label(span, "不正な`'`が見つかりました".to_string());

        diag
    }

    pub fn unterminated_char_literal (
        dcx: DiagCtxtHandle<'dcx>,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(ErrorCode::UnterminatedCharLiteral.into());
        diag.set_message("閉じられていない文字リテラル".to_string());
        diag.set_label(span, "文字リテラルが閉じられていません".to_string());

        diag
    }

    pub fn multiple_characters_in_char_literal (
        dcx: DiagCtxtHandle<'dcx>,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(ErrorCode::MultipleCharactersInCharLiteral.into());
        diag.set_message("無効な複数文字の文字リテラル".to_string());
        diag.set_label(span, "文字リテラルに複数の文字が含まれています".to_string());
        diag.set_help("一文字にするか、文字列に変更してください".to_string());

        diag
    }
}



#[repr(i32)]
enum ErrorCode {
    UnexpectedCharacter = 100,
    InvalidFloatFormat = 101,
    MissingFractionalPart = 102,
    InvalidEscapeSequence = 103,
    UnterminatedStringLiteral = 104,
    UnexpectedQuote = 105,
    UnterminatedCharLiteral = 106,
    MultipleCharactersInCharLiteral = 107,
}

impl From<ErrorCode> for i32 {
    fn from(value: ErrorCode) -> Self {
        value as i32
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::stelaro_common::create_default_session_globals_then;
    use crate::stelaro_diagnostic::emitter::SilentEmitter;
    use crate::stelaro_diagnostic::DiagCtxt;
    use crate::stelaro_lexer::{diagnostics::ErrorCode, Lexer};
    use crate::stelaro_session::ParseSess;
    use crate::stelaro_common::source_map::SourceMap;

    fn create_test_context() -> ParseSess {
        let source_map = Rc::new(SourceMap::new());
        let emitter = SilentEmitter::new();
        let dcx = DiagCtxt::new(Box::new(emitter));
        ParseSess::with_dcx(dcx, source_map)
    }

    fn get_sess_after_src_lex(src: &str) -> (ParseSess, bool) {
        create_default_session_globals_then(|| {
            let src = Rc::new(src.to_string());
            let psess = create_test_context();
            let mut lexer = Lexer::new(&psess, &src);
            let is_err = lexer.lex().is_err();
            (psess, is_err)
        })
    }

    #[test]
    fn test_unexpected_character() {
        let (sess, is_err) = get_sess_after_src_lex(
            "let # = 0;"
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::UnexpectedCharacter.into()));
    }

    #[test]
    fn test_invalid_float_format() {
        let (sess, is_err) = get_sess_after_src_lex(
            "123.456.789;"
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::InvalidFloatFormat.into()));
    }

    #[test]
    fn test_missing_fractional_part() {
        let (sess, is_err) = get_sess_after_src_lex(
            "123.;"
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::MissingFractionalPart.into()));
    }

    #[test]
    fn test_invalid_escape_sequence() {
        let (sess, is_err) = get_sess_after_src_lex(
            r#""Hello, World\r\q\""#
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::InvalidEscapeSequence.into()));
    }

    #[test]
    fn test_unterminated_string_literal() {
        let (sess, is_err) = get_sess_after_src_lex(
            r#""Hello, steralo!"#
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::UnterminatedStringLiteral.into()));
    }

    #[test]
    fn test_unexpected_quote() {
        let (sess, is_err) = get_sess_after_src_lex(
            "'\na'"
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::UnexpectedQuote.into()));
    }

    #[test]
    fn test_unterminated_char_literal() {
        let (sess, is_err) = get_sess_after_src_lex(
            "if 'a == ch {"
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::UnterminatedCharLiteral.into()));
    }

    #[test]
    fn test_multiple_characters_in_char_literal() {
        let (sess, is_err) = get_sess_after_src_lex(
            "'String'"
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::MultipleCharactersInCharLiteral.into()));
    }

}
