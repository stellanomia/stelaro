use crate::stelaro_ast::token;
use crate::stelaro_common::{Span, Symbol, diagnostics::Diags};
use crate::stelaro_diagnostics::ErrorEmitted;
use crate::stelaro_session::ParseSess;
use crate::stelaro_sir::sir::LitKind;


pub enum LitError {
    IntTooLarge,
}

impl LitKind {
    /// トークンのリテラルをセマンティックなリテラルに変換する
    pub fn from_token_lit(lit: token::Lit) -> Result<LitKind, LitError> {
        let token::Lit { kind, symbol} = lit;

        Ok(
            match kind {
                token::LitKind::Bool(b) => {
                    LitKind::Bool(b)
                },
                token::LitKind::Char => {
                    let ch = symbol.as_str();
                    let ch = if ch.contains('\\') {
                        unescape_char(ch)
                    } else {
                        ch.parse::<char>()
                            .expect("bug: Lexer が適切に char リテラルを処理できていません。アンエスケープに失敗しました。")
                    };
                    LitKind::Char(
                        ch
                    )
                },
                token::LitKind::Integer => return integer_lit(symbol),
                token::LitKind::Float => {
                    LitKind::Float(symbol)
                },
                token::LitKind::Str => {
                    let str = symbol.as_str();
                    if str.contains('\\') {
                        LitKind::Str(unescape_str(str))
                    } else {
                        LitKind::Str(symbol)
                    }
                },
            }
        )
    }
}

pub fn report_lit_error(
    psess: &ParseSess,
    err: LitError,
    lit: token::Lit,
    span: Span,
) -> ErrorEmitted {
    match err {
        LitError::IntTooLarge => {
            Diags::int_too_large(psess.dcx(), lit, span).emit()
        }
    }
}

fn integer_lit(symbol: Symbol) -> Result<LitKind, LitError> {
    let s = symbol.as_str();
    s.parse::<u128>()
        .map(LitKind::Int)
        .map_err(|_| LitError::IntTooLarge)
}

// エスケープシーケンスを含む char である必要がある
#[inline]
pub fn unescape_char(ch: &str) -> char {
    let mut chars = ch.chars();
    if let Some('\\') = chars.next() && let Some(e) = chars.next() {
        match e {
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            '0' => '\0',
            '\'' => '\'',
            '"' => '\"',
            '\\' => '\\',
            _ => unreachable!("未定義のエスケープシーケンス: {e}")
        }
    } else {
        unreachable!("エスケープシーケンスを含む char である必要がある: {}", chars.as_str())
    }
}

// エスケープシーケンスを含む文字列である必要がある
#[inline]
pub fn unescape_str(str: &str) -> Symbol {
    let mut chars = str.chars();
    let mut buf = String::with_capacity(str.len());
    while let Some(ch) = chars.next() {
        // 入力ソースは UTF-8 を前提にしている
        let start = str.len() - chars.as_str().len() - ch.len_utf8();
        let end = str.len() - chars.as_str().len();
        if ch == '\\' {
            chars.next();
            buf.push(unescape_char(&str[start..=end]));
        } else {
            buf.push(ch);
        }
    }

    Symbol::intern(buf.leak())
}
