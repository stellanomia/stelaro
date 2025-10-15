//! rustc の `rustc_index_macros/newtype.rs` に基づいて設計されています。

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::*;
use syn::*;

struct Newtype(TokenStream);

impl Parse for Newtype {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attrs = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        input.parse::<Token![struct]>()?;
        let name: Ident = input.parse()?;


        let body;
        braced!(body in input);

        let mut derive_paths: Vec<Path> = Vec::new();
        let mut debug_format: Option<Lit> = None;
        let mut max = None;
        let mut consts = Vec::new();
        let mut ord = false;

        attrs.retain(|attr| match attr.path().get_ident() {
            Some(ident) => match &*ident.to_string() {
                "orderable" => {
                    ord = true;
                    false
                }
                "max" => {
                    let Meta::NameValue(MetaNameValue { value: Expr::Lit(lit), .. }) = &attr.meta
                    else {
                        panic!("#[max = NUMBER] attribute requires max value");
                    };

                    if let Some(old) = max.replace(lit.lit.clone()) {
                        panic!("Specified multiple max: {:?}", old.span().source_text());
                    }

                    false
                }
                "debug_format" => {
                    let Meta::NameValue(MetaNameValue { value: Expr::Lit(lit), .. }) = &attr.meta
                    else {
                        panic!("#[debug_format = FMT] attribute requires a format");
                    };

                    if let Some(old) = debug_format.replace(lit.lit.clone()) {
                        panic!("Specified multiple debug format options: {:?}", old.span().source_text());
                    }

                    false
                }
                _ => true,
            },
            _ => true,
        });


        loop {
            // ユーザーが提供したものをすべてパースした
            if body.is_empty() {
                break;
            }

            // それ以外の場合は、ユーザー定義の定数をパースします
            let const_attrs = body.call(Attribute::parse_outer)?;
            body.parse::<Token![const]>()?;
            let const_name: Ident = body.parse()?;
            body.parse::<Token![=]>()?;
            let const_val: Expr = body.parse()?;
            body.parse::<Token![;]>()?;
            consts.push(quote! { #(#const_attrs)* #vis const #const_name: #name = #name::from_u32(#const_val); });
        }

        let debug_format =
            debug_format.unwrap_or_else(|| Lit::Str(LitStr::new("{}", Span::call_site())));

        // これらのインデックスをenumにパックするためのスペースを確保するため、末尾の256個のインデックスを削る
        let max = max.unwrap_or_else(|| Lit::Int(LitInt::new("0xFFFF_FF00", Span::call_site())));

        if ord {
            derive_paths.push(parse_quote!(Ord));
            derive_paths.push(parse_quote!(PartialOrd));
        }

        let debug_impl = quote! {
            impl ::std::fmt::Debug for #name {
                fn fmt(&self, fmt: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    write!(fmt, #debug_format, self.as_u32())
                }
            }
        };

        Ok(Self(quote! {
            #(#attrs)*
            #[derive(Clone, Copy, PartialEq, Eq, Hash, #(#derive_paths),*)]
            #vis struct #name {
                private_use_as_methods_instead: u32,
            }

            #(#consts)*

            impl #name {
                /// インデックスが取りうる最大値（`u32`形式）。
                #vis const MAX_AS_U32: u32  = #max;

                /// インデックスが取りうる最大値。
                #vis const MAX: Self = Self::from_u32(#max);

                /// インデックスのゼロ値。
                #vis const ZERO: Self = Self::from_u32(0);

                /// 与えられた `usize` から新しいインデックスを生成する。
                ///
                /// # Panics
                ///
                /// `value` が `MAX` を超えた場合にPanicsする。
                #[inline]
                #vis const fn from_usize(value: usize) -> Self {
                    assert!(value <= (#max as usize));
                    // 安全性: `value <= max` であることは直前にチェック済み。
                    unsafe {
                        Self::from_u32_unchecked(value as u32)
                    }
                }

                /// 与えられた `u32` から新しいインデックスを生成する。
                ///
                /// # Panics
                ///
                /// `value` が `MAX` を超えた場合にパニックする。
                #[inline]
                #vis const fn from_u32(value: u32) -> Self {
                    assert!(value <= #max);
                    // 安全性: `value <= max` であることは直前にチェック済み。
                    unsafe {
                        Self::from_u32_unchecked(value)
                    }
                }

                /// 与えられた `u16` から新しいインデックスを生成する。
                ///
                /// # Panics
                ///
                /// `value` が `MAX` を超えた場合にパニックする。
                #[inline]
                #vis const fn from_u16(value: u16) -> Self {
                    let value = value as u32;
                    assert!(value <= #max);
                    // 安全性: `value <= max` であることは直前にチェック済み。
                    unsafe {
                        Self::from_u32_unchecked(value)
                    }
                }

                /// 与えられた `u32` から新しいインデックスを生成する。
                ///
                /// # 安全性
                ///
                /// 提供される値は、このnewtypeの最大値以下でなければならない。
                /// レイアウトの制約により、この範囲外の値を提供すると未定義動作となる。
                ///
                /// `from_u32` の使用を推奨する。
                #[inline]
                #vis const unsafe fn from_u32_unchecked(value: u32) -> Self {
                    Self { private_use_as_methods_instead: value }
                }

                /// このインデックスの値を `usize` として抽出する。
                #[inline]
                #vis const fn index(self) -> usize {
                    self.as_usize()
                }

                /// このインデックスの値を `u32` として抽出する。
                #[inline]
                #vis const fn as_u32(self) -> u32 {
                    self.private_use_as_methods_instead
                }

                /// このインデックスの値を `usize` として抽出する。
                #[inline]
                #vis const fn as_usize(self) -> usize {
                    self.as_u32() as usize
                }
            }

            impl std::ops::Add<usize> for #name {
                type Output = Self;

                #[inline]
                fn add(self, other: usize) -> Self {
                    Self::from_usize(self.index() + other)
                }
            }

            impl std::ops::AddAssign<usize> for #name {
                #[inline]
                fn add_assign(&mut self, other: usize) {
                    *self = *self + other;
                }
            }

            impl crate::stelaro_common::Idx for #name {
                #[inline]
                fn new(value: usize) -> Self {
                    Self::from_usize(value)
                }

                #[inline]
                fn index(self) -> usize {
                    self.as_usize()
                }
            }

            impl From<#name> for u32 {
                #[inline]
                fn from(v: #name) -> u32 {
                    v.as_u32()
                }
            }

            impl From<#name> for usize {
                #[inline]
                fn from(v: #name) -> usize {
                    v.as_usize()
                }
            }

            impl From<usize> for #name {
                #[inline]
                fn from(value: usize) -> Self {
                    Self::from_usize(value)
                }
            }

            impl From<u32> for #name {
                #[inline]
                fn from(value: u32) -> Self {
                    Self::from_u32(value)
                }
            }

            #debug_impl
        }))
    }
}

pub(crate) fn newtype(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Newtype);
    input.0.into()
}