use crate::stelaro_ast::token;
use crate::stelaro_common::{Span, Symbol};
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
                token::LitKind::Integer => todo!(),
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
    todo!()
}

// エスケープシーケンスを含む char である必要がある
#[inline]
fn unescape_char(ch: &str) -> char {
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
fn unescape_str(str: &str) -> Symbol {
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

#[cfg(test)]
mod tests {
    use crate::stelaro_common::create_default_session_globals_then;

    use super::*;

    fn test_unescape<T>(f: impl FnOnce() -> T) -> T {
        create_default_session_globals_then(f)
    }

    #[test]
    fn test_unescape_str() {
        test_unescape(|| {
            assert_eq!(
                "abcd	efgh\n	ijkl\\あ\0い\r⭐✨\"\'",
                unescape_str(r#"abcd\tefgh\n\tijkl\\あ\0い\r⭐✨\"\'"#).as_str(),
            );
        })
    }
}