//! rustc の `rustc_index_macros/newtype.rs` に基づいて設計されています。

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::*;
use syn::*;

struct Newtype(TokenStream);

impl Parse for Newtype {
    fn parse(input: ParseStream) -> Result<Self> {
        todo!()
    }
}

pub(crate) fn newtype(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Newtype);
    input.0.into()
}
