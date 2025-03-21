use crate::stelaro_ast::token::{Token, TokenKind};
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
        diag.set_code(ErrorCode::UnexpectedToken.into());
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
        diag.set_code(ErrorCode::ChainedComparison.into());
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
        diag.set_code(ErrorCode::MissingOperator.into());
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
        diag.set_code(ErrorCode::ExpectExpression.into());
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
        diag.set_code(ErrorCode::PrefixIncrement.into());
        diag.set_message("前置インクリメント".to_string());
        diag.set_label(span, "steraloは前置インクリメント演算子をもちません".to_string());
        diag.set_help("`lhs += 1;`を使用してください".to_string());

        diag
    }

}

#[repr(i32)]
enum ErrorCode {
    UnexpectedToken = 200,
    ChainedComparison = 201,
    MissingOperator = 202,
    ExpectExpression = 203,
    PrefixIncrement = 204,
}

impl From<ErrorCode> for i32 {
    fn from(value: ErrorCode) -> Self {
        value as i32
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{stelaro_common::source_map::SourceMap, stelaro_diagnostic::DiagCtxt, stelaro_parse::{diagnostics::ErrorCode, new_parser_from_src, parser::Parser}, stelaro_session::Session};

    fn create_sess(src: Rc<String>) -> Session {
        let dcx = DiagCtxt::new(Rc::clone(&src));
        let source_map = Rc::new(SourceMap::new());
        Session::new(dcx, source_map)
    }

    fn create_parser(sess: &Session, src: Rc<String>) -> Parser<'_> {
        new_parser_from_src(sess, src.to_string()).unwrap()
    }

    fn get_sess_after_parsed(src: &str) -> (Session, bool) {
        let src = Rc::new(src.to_string());
        let sess = create_sess(Rc::clone(&src));
        let mut parser = create_parser(&sess, src);
        let is_err = parser.parse_expr().is_err();
        (sess, is_err)
    }


    #[test]
    fn test_chained_comparison() {
        let (sess, is_err) = get_sess_after_parsed(
            "x = 1 < 2 < 3"
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::ChainedComparison.into()));
    }

    #[test]
    fn test_missing_operator() {
        let (sess, is_err) = get_sess_after_parsed(
            "x = 1 + 2 3"
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::MissingOperator.into()));
    }

    #[test]
    fn test_expect_expression() {
        let (sess, is_err) = get_sess_after_parsed(
            "1 + 2 - * 3"
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::ExpectExpression.into()));
    }

    #[test]
    fn prefix_increment() {
        let (sess, is_err) = get_sess_after_parsed(
            "y = ++x"
        );

        assert!(is_err);
        assert!(sess.dcx().has_err_code(ErrorCode::PrefixIncrement.into()));
    }
}