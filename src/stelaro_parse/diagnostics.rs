use crate::stelaro_ast::token::{Lit, Token, TokenKind};
use crate::stelaro_common::span::Span;
use crate::stelaro_diagnostic::diag::{Diag, DiagCtxtHandle, ErrorEmitted};

pub struct DiagsParser;

impl<'dcx> DiagsParser {
    pub fn unexpected_token (
        dcx: DiagCtxtHandle<'dcx>,
        unexpected: TokenKind,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(ErrorCode::UnexpectedToken.into());
        diag.set_message(format!("予期しないトークン: `{}`", unexpected));
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
        diag.set_code(ErrorCode::UnexpectedToken.into());
        diag.set_message(format!("予期しないトークン: `{}`", unexpected));
        diag.set_label(span, format!("`{}`を期待していましたが、`{}` は無効な入力です", expected, unexpected));

        diag
    }

    pub fn unexpected_token_with_expected_any (
        dcx: DiagCtxtHandle<'dcx>,
        unexpected: TokenKind,
        expected: &[TokenKind],
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(ErrorCode::UnexpectedToken.into());
        diag.set_message(format!("予期しないトークン: `{}`", unexpected));

        match expected.len() {
            0 => {
                diag.set_label(span, format!("`{}` は無効な入力です", unexpected));
            }
            1 => {
                diag.set_label(
                    span,
                    format!("`{}` を期待していましたが、`{}` は無効な入力です", expected[0], unexpected),
                );
            }
            _ => {
                let expected_list = expected
                    .iter()
                    .map(|t| format!("`{}`", t))
                    .collect::<Vec<_>>()
                    .join(", ");
                diag.set_label(
                    span,
                    format!(
                        "{} のいずれかを期待していましたが、`{}` は無効な入力です",
                        expected_list, unexpected
                    ),
                );
            }
        }

        diag
    }

    pub fn chained_comparison (
        dcx: DiagCtxtHandle<'dcx>,
        op1: Span,
        op2: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(op1.merge(&op2));
        diag.set_code(ErrorCode::ChainedComparison.into());
        diag.set_message("連鎖した比較演算子".to_string());
        diag.set_label(op1, "この比較は無効です".to_string());
        diag.set_label(op2, "比較演算子が連鎖しています".to_string());

        diag
    }

    pub fn expect_expression (
        dcx: DiagCtxtHandle<'dcx>,
        unexpected: Token,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(ErrorCode::ExpectExpression.into());
        diag.set_message("不正な式".to_string());
        diag.set_label(span, format!("`{}`は式ではありません", unexpected));
        diag.set_help("これを削除するか、間に値を追加してください".to_string());

        diag
    }

    pub fn prefix_increment (
        dcx: DiagCtxtHandle<'dcx>,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(ErrorCode::PrefixIncrement.into());
        diag.set_message("前置インクリメント".to_string());
        diag.set_label(span, "steraloは前置インクリメント演算子をもちません".to_string());
        diag.set_help("`lhs += 1;`を使用してください".to_string());

        diag
    }

    pub fn unexpected_closing_delimiter (
        dcx: DiagCtxtHandle<'dcx>,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(ErrorCode::UnexpectedClosingDelimiter.into());
        diag.set_message("余分な閉じ括弧".to_string());
        diag.set_label(span, "式の解析中に余分な閉じ括弧が見つかりました".to_string());
        diag.set_help("これを削除してください".to_string());

        diag
    }

    pub fn unexpected_token_for_identifier (
        dcx: DiagCtxtHandle<'dcx>,
        span: Span
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(ErrorCode::UnexpectedTokenForIdentifier.into());
        diag.set_message("無効な識別子".to_string());
        diag.set_label(span, "識別子でない予期しないトークンがあります".to_string());

        diag
    }

    pub fn unexpected_token_for_type (
        dcx: DiagCtxtHandle<'dcx>,
        span: Span
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(ErrorCode::UnexpectedTokenForType.into());
        diag.set_message("無効な型".to_string());
        diag.set_label(span, "これは型ではありません".to_string());

        diag
    }

    pub fn unexpected_numeric_literal_for_identifier (
        dcx: DiagCtxtHandle<'dcx>,
        unexpected_literal: Lit,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(ErrorCode::UnexpectedNumericLiteralForIdentifier.into());
        diag.set_message("識別子はリテラルから始めることができない".to_string());
        diag.set_label(span, format!("識別子を`{}`から始めることはできません", unexpected_literal.symbol.as_str()));
        diag.set_help("識別子に数値リテラルを使うことはできません".to_string());

        diag
    }

    // 式文解析時にセミコロンがない場合使用される
    pub fn missing_semicolon (
        dcx: DiagCtxtHandle<'dcx>,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(ErrorCode::MissingSemicolon.into());
        diag.set_message("セミコロンがありません".to_string());
        diag.set_label(span, "この文の末尾にセミコロンが必要です".to_string());
        diag.set_help("文の末尾に `;` を追加してください".to_string());

        diag
    }

    pub fn missing_function_body (
        dcx: DiagCtxtHandle<'dcx>,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(ErrorCode::MissingFunctionBody.into());
        diag.set_message("関数のボディがありません".to_string());
        diag.set_label(span, "関数のボディ`{...}`がありません".to_string());

        diag
    }

    pub fn cannot_use_underscore_as_identifier (
        dcx: DiagCtxtHandle<'dcx>,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(ErrorCode::CannotUseUnderscoreAsIdentifier.into());
        diag.set_message("無効な識別子: `_`".to_string());
        diag.set_label(span, "`_` を識別子として使用することはできません".to_string());
        diag.set_help("`_` は値の破棄や未使用変数の表現としてのみ利用可能です".to_string());

        diag
    }

    pub fn unexpected_token_for_item (
        dcx: DiagCtxtHandle<'dcx>,
        unexpected: TokenKind,
        span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(ErrorCode::UnexpectedTokenForItem.into());
        diag.set_message(format!("予期しないトークン: `{}`", unexpected));

        // Itemが最初にとりうるトークンが増えたとき、ここに追加する
        let expected_list = [TokenKind::Fn, TokenKind::Mod]
            .iter()
            .map(|t| format!("`{}`", t))
            .collect::<Vec<_>>()
            .join(", ");

        diag.set_label(
            span,
            format!(
                "{} のいずれかを期待していましたが、`{}` は無効な入力です",
                expected_list, unexpected
            ),
        );

        diag
    }

    pub fn unclosed_delimiter (
        dcx: DiagCtxtHandle<'dcx>,
        unexpected: Token,
        opening_delim_span: Span,
    ) -> Diag<'dcx, ErrorEmitted> {
        let mut diag = dcx.struct_err(unexpected.span);
        diag.set_code(ErrorCode::UnclosedDelimiter.into());
        diag.set_message("閉じられていない括弧".to_string());

        diag.set_label(
            unexpected.span,
            format!("閉じ括弧を期待していましたが、`{}` は無効な入力です", unexpected),
        );

        diag.set_label(
            opening_delim_span,
            "これに対応する括弧が見つかりません".to_string()
        );

        diag
    }
}

#[repr(i32)]
enum ErrorCode {
    UnexpectedToken = 200,
    ChainedComparison = 201,
    ExpectExpression = 202,
    PrefixIncrement = 203,
    UnexpectedClosingDelimiter = 204,
    UnexpectedTokenForIdentifier = 205,
    UnexpectedTokenForType = 206,
    UnexpectedNumericLiteralForIdentifier = 207,
    MissingSemicolon = 208,
    MissingFunctionBody = 209,
    CannotUseUnderscoreAsIdentifier = 210,
    UnexpectedTokenForItem = 211,
    UnclosedDelimiter = 212,
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
    use crate::stelaro_session::ParseSess;
    use crate::stelaro_parse::{diagnostics::ErrorCode, new_parser_from_source_str, parser::Parser};
    use crate::stelaro_diagnostic::DiagCtxt;
    use crate::stelaro_common::source_map::SourceMap;

    fn create_test_context() -> ParseSess {
        let source_map = Rc::new(SourceMap::new());
        let emitter = SilentEmitter::new();
        let dcx = DiagCtxt::new(Box::new(emitter));
        ParseSess::with_dcx(dcx, source_map)
    }

    fn src_to_parser<'a>(psess: &'a ParseSess, src: Rc<String>) -> Parser<'a> {
        new_parser_from_source_str(
            psess,
            "parse_test".into(),
            src.to_string()
        ).unwrap()
    }

    fn get_sess_after_expr_parse(src: &str) -> (ParseSess, bool) {
        create_default_session_globals_then(|| {
            let src = Rc::new(src.to_string());
            let psess = create_test_context();
            let is_err = src_to_parser(&psess, src).parse_expr().is_err();
            (psess, is_err)
        })
    }

    fn get_sess_after_stmt_parse(src: &str) -> (ParseSess, bool) {
        create_default_session_globals_then(|| {
            let src = Rc::new(src.to_string());
            let psess = create_test_context();
            let is_err = src_to_parser(&psess, src).parse_stmt().is_err();
            (psess, is_err)
        })
    }

    fn get_sess_after_item_parse(src: &str) -> (ParseSess, bool) {
        create_default_session_globals_then(|| {
            let src = Rc::new(src.to_string());
            let psess = create_test_context();
            let is_err = src_to_parser(&psess, src).parse_item().is_err();
            (psess, is_err)
        })
    }

    fn get_sess_after_stelo_parse(src: &str) -> (ParseSess, bool) {
        create_default_session_globals_then(|| {
            let src = Rc::new(src.to_string());
            let psess = create_test_context();
            let is_err = src_to_parser(&psess, src).parse_stelo().is_err();
            (psess, is_err)
        })
    }


    #[test]
    fn test_chained_comparison() {
        let (sess, is_err) = get_sess_after_expr_parse(
            "x = 1 < 2 < 3"
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::ChainedComparison.into()));
    }

    #[test]
    fn test_expect_expression() {
        let (sess, is_err) = get_sess_after_expr_parse(
            "1 + 2 - * 3"
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::ExpectExpression.into()));
    }

    #[test]
    fn test_prefix_increment() {
        let (sess, is_err) = get_sess_after_expr_parse(
            "y = ++x"
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::PrefixIncrement.into()));
    }

    #[test]
    fn test_unexpected_closing_delimiter() {
        let (sess, is_err) = get_sess_after_expr_parse(
            "(a + b)) + 1"
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::UnexpectedClosingDelimiter.into()));
    }

    #[test]
    fn test_unexpected_token_for_identifier() {
        let (sess, is_err) = get_sess_after_stmt_parse(
            "let if = 0;"
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::UnexpectedTokenForIdentifier.into()));
    }

    #[test]
    fn test_unexpected_token_for_type() {
        let (sess, is_err) = get_sess_after_item_parse(
            "fn main(x: i32, y:i32): {"
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::UnexpectedTokenForType.into()));
    }

    #[test]
    fn unexpected_numeric_literal_for_identifier() {
        let (sess, is_err) = get_sess_after_stmt_parse(
            "let 123abc = 0;"
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::UnexpectedNumericLiteralForIdentifier.into()));
    }

    #[test]
    fn test_missing_semicolon() {
        let (sess, is_err) = get_sess_after_item_parse(
            r#"
    fn f() {
        f(123, 456)
        return ;
    }
"#.trim()  // セミコロンなし
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::MissingSemicolon.into()));
    }

    #[test]
    fn test_missing_function_body() {
        let (sess, is_err) = get_sess_after_item_parse(
            "fn f(x: i32,)"
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::MissingFunctionBody.into()));
    }

    #[test]
    fn test_cannot_use_underscore_as_identifier() {
        let (sess, is_err) = get_sess_after_stelo_parse(
            r#"
    fn f(x: i32, y: i32) : i32 {
        let z = if x < y {
            y
        } else {
            x + y
        };

        return z;
    }

    fn _(x: i32) {
        return x;
    }
"#.trim()
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::CannotUseUnderscoreAsIdentifier.into()));
    }


    #[test]
    fn test_unexpected_token_for_item() {
        let (sess, is_err) = get_sess_after_stelo_parse(
            r#"
    fn f(x: i32, y: i32): bool {
        print("Hello");
        true
    }

    if f() {
        let x = 1 + 1;
    }
"#.trim()
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::UnexpectedTokenForItem.into()));
    }



    #[test]
    fn test_unclosed_delimiter() {
        let (sess, is_err) = get_sess_after_stelo_parse(
            r#"
    mod my_mod {
        fn f(): i32 {
            0


        fn a() {
            print("");
        }
    }
"#.trim()
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::UnclosedDelimiter.into()));
    }
}