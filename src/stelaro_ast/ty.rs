use crate::stelaro_common::span::Span;

#[derive(Debug)]
pub struct Ty {
    kind: TyKind,
    span: Span,
}

#[derive(Debug)]
enum TyKind {

}