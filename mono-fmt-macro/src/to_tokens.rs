use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::format::{Align, FormatTrait};

pub struct Scoped<'a, T> {
    export: &'a syn::Path,
    inner: &'a T,
}

impl<'a, T> Scoped<'a, T> {
    pub fn new(export: &'a syn::Path, inner: &'a T) -> Self {
        Self { export, inner }
    }

    fn scope<'b, U>(&self, inner: &'b U) -> Scoped<'b, U>
    where
        'a: 'b,
    {
        Scoped {
            inner,
            export: self.export,
        }
    }

    fn as_ref(&self) -> &'a T {
        self.inner
    }
}

impl ToTokens for Align {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            Self::Left => quote! { 1 },
            Self::Center => quote! { 2 },
            Self::Right => quote! { 3 },
        })
    }
}

impl ToTokens for FormatTrait {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            FormatTrait::Display => quote!(Display),
            FormatTrait::Debug => quote!(Debug),
            FormatTrait::Octal => quote!(Octal),
            FormatTrait::LowerHex => quote!(LowerHex),
            FormatTrait::UpperHex => quote!(UpperHex),
            FormatTrait::Pointer => quote!(Pointer),
            FormatTrait::Binary => quote!(Binary),
            FormatTrait::LowerExp => quote!(LowerExp),
            FormatTrait::UpperExp => quote!(UpperExp),
        }
        .to_tokens(tokens)
    }
}
