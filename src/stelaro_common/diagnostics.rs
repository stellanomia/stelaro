use crate::stelaro_ast::token;
use crate::stelaro_common::Span;
use crate::stelaro_diagnostics::{Diag, DiagCtxtHandle};


pub struct Diags;

impl<'dcx> Diags {
    pub fn int_too_large(
        dcx: DiagCtxtHandle<'dcx>,
        _lit: token::Lit,
        span: Span,
    ) -> Diag<'dcx> {
        let mut diag = dcx.struct_err(span);
        diag.set_code(ErrorCode::IntTooLarge as i32);
        diag.set_message("大きすぎる整数リテラル".to_string());
        diag.set_label(
            span,
            // u128::MAX == 340282366920938463463374607431768211455
            "この値は制限である `340282366920938463463374607431768211455` を超えています".to_string()
        );
        diag
    }
}


#[repr(i32)]
enum ErrorCode {
    IntTooLarge = 900,
}
