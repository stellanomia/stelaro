use crate::stelaro_ast::token;
use crate::stelaro_common::Span;
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
                    todo!()
                },
                token::LitKind::Integer => todo!(),
                token::LitKind::Float => {
                    LitKind::Float(symbol)
                },
                token::LitKind::Str => {
                    todo!()
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
