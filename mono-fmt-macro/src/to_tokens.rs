use std::cell::Cell;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};

use crate::{
    format::{
        Align, Count, DebugHex, Format, FormatArg, FormatArgRef, FormatTrait, FormatterArgs, Piece,
        Sign,
    },
    Input,
};

pub(crate) struct Scoped<'a, T> {
    input: &'a Input,
    current_position: &'a Cell<usize>,
    inner: &'a T,
}

fn pos_arg_ident(idx: usize) -> Ident {
    Ident::new(&format!("__pos_arg_{idx}"), Span::mixed_site())
}

fn named_arg_ident(name: impl std::fmt::Display) -> Ident {
    Ident::new(&format!("__named_arg_{name}"), Span::mixed_site())
}

impl<'a, T> Scoped<'a, T> {
    pub fn new(input: &'a Input, inner: &'a T, current_position: &'a Cell<usize>) -> Self {
        Self {
            input,
            inner,
            current_position,
        }
    }

    fn scope<'b, U>(&self, inner: &'b U) -> Scoped<'b, U>
    where
        'a: 'b,
    {
        Scoped {
            inner,
            input: self.input,
            current_position: self.current_position,
        }
    }

    fn as_ref(&self) -> &'a T {
        self.inner
    }
}

impl ToTokens for Scoped<'_, Format<'_>> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let parts = self.inner.pieces.iter().map(|piece| self.scope(piece));

        let input = &self.input;

        let pos_args = input.positional_args.iter();
        let named_args = input.named_args.iter().map(|(_, expr)| expr);

        let args = pos_args.chain(named_args);

        let pos_idents = input
            .positional_args
            .iter()
            .enumerate()
            .map(|(idx, _)| pos_arg_ident(idx));

        let named_idents = input
            .named_args
            .iter()
            .map(|(name, _)| named_arg_ident(name));

        let idents = pos_idents.chain(named_idents);

        tokens.extend(quote! {
            #[allow(unused_parens)]
            match (#(&#args),*) {
                (#(#idents),*) => (
                    #(#parts),*
                )
            }
        })
    }
}

impl ToTokens for Scoped<'_, Piece<'_>> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let prefix = &self.input.prefix;

        match self.inner {
            Piece::Lit(literal) => {
                let lit = syn::LitStr::new(literal, self.input.format_str.span());

                tokens.extend(quote! { #prefix::Str(#lit) });
            }
            Piece::Arg(arg) => self.scope(arg).to_tokens(tokens),
        }
    }
}

impl ToTokens for Scoped<'_, FormatArg<'_>> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let prefix = &self.input.prefix;

        let base = self.inner.format_spec.format_trait;

        let expr = match self.inner.arg {
            None => {
                let current_position = self.current_position.get();
                self.current_position.set(current_position + 1);

                pos_arg_ident(current_position).to_token_stream()
            }
            Some(FormatArgRef::Positional(idx)) => pos_arg_ident(idx).to_token_stream(),
            Some(FormatArgRef::Named(name)) => self
                .input
                .named_args
                .iter()
                .find(|(arg, _)| arg == name)
                .map(|(name, _)| named_arg_ident(name).to_token_stream())
                .unwrap_or_else(|| {
                    let ident = Ident::new(name, self.input.format_str.span());
                    quote! { &#ident }
                }),
        };

        let opt_ty = opt_ty_tokens(self.scope(&self.inner.format_spec.formatter_args));
        let opt_values = opt_value_tokens(self.scope(&self.inner.format_spec.formatter_args));

        tokens.extend(quote! { #prefix::#base::<_, #opt_ty>(#expr, #opt_values) })
    }
}

fn opt_value_tokens(scope: Scoped<'_, FormatterArgs<'_>>) -> TokenStream {
    let args = &scope.inner;
    let prefix = &scope.input.prefix;

    let mut opts = quote! { () };

    if args.alternate {
        opts = quote! { #prefix::WithAlternate(#opts) };
    }

    if let Some(width) = args.width {
        let width = match width {
            Count::Integer(int) => int,
            Count::Parameter(_) => panic!("parameter counts are not supported right now"),
        };
        opts = quote! { #prefix::WithWidth(#opts) };
    }

    if let Some(align) = args.align {
        opts = quote! { #prefix::WithAlign(#opts) };
    }

    if let Some(Sign::Plus) = args.sign {
        opts = quote! { #prefix::WithSignPlus(#opts) };
    }

    if let Some(Sign::Minus) = args.sign {
        opts = quote! { #prefix::WithMinus(#opts)};
    }

    if let Some(precision) = args.precision {
        let precision = match precision {
            Count::Integer(int) => int,
            Count::Parameter(_) => panic!("parameter counts are not supported right now"),
        };
        opts = quote! { #prefix::WithPrecision(#opts) };
    }

    if let Some(Sign::Plus) = args.sign {
        opts = quote! { #prefix::WithSignPlus(#opts) };
    }

    if args.zero {
        opts = quote! { #prefix::WithSignAwareZeroPad(#opts) };
    }

    if let Some(DebugHex::Lower) = args.debug_hex {
        opts = quote! { #prefix::WithDebugLowerHex(#opts) };
    }

    if let Some(DebugHex::Upper) = args.debug_hex {
        opts = quote! { #prefix::WithDebugUpperHex(#opts) };
    }

    opts
}

fn opt_ty_tokens(scope: Scoped<'_, FormatterArgs<'_>>) -> TokenStream {
    let args = &scope.inner;
    let prefix = &scope.input.prefix;

    let mut opts = quote! { () };

    if args.alternate {
        opts = quote! { #prefix::WithAlternate<#opts> };
    }

    if let Some(width) = args.width {
        let width = match width {
            Count::Integer(int) => int,
            Count::Parameter(_) => panic!("parameter counts are not supported right now"),
        };
        opts = quote! { #prefix::WithWidth<#opts, #width> };
    }

    if let Some(align) = args.align {
        opts = quote! { #prefix::WithAlign<#opts, #align> };
    }

    if let Some(Sign::Plus) = args.sign {
        opts = quote! { #prefix::WithSignPlus<#opts> };
    }

    if let Some(Sign::Minus) = args.sign {
        opts = quote! { #prefix::WithMinus<#opts> };
    }

    if let Some(precision) = args.precision {
        let precision = match precision {
            Count::Integer(int) => int,
            Count::Parameter(_) => panic!("parameter counts are not supported right now"),
        };
        opts = quote! { #prefix::WithPrecision<#opts, #precision> };
    }

    if let Some(Sign::Plus) = args.sign {
        opts = quote! { #prefix::WithSignPlus<#opts> };
    }

    if args.zero {
        opts = quote! { #prefix::WithSignAwareZeroPad<#opts> };
    }

    if let Some(DebugHex::Lower) = args.debug_hex {
        opts = quote! { #prefix::WithDebugLowerHex<#opts> };
    }

    if let Some(DebugHex::Upper) = args.debug_hex {
        opts = quote! { #prefix::WithDebugUpperHex<#opts> };
    }

    opts
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
            FormatTrait::Display => quote! { DisplayArg },
            FormatTrait::Debug => quote! { DebugArg },
            FormatTrait::Octal => quote! { OctalArg },
            FormatTrait::LowerHex => quote! { LowerHexArg },
            FormatTrait::UpperHex => quote! { UpperHexArg },
            FormatTrait::Pointer => quote! { PointerArg },
            FormatTrait::Binary => quote! { BinaryArg },
            FormatTrait::LowerExp => quote! { LowerExpArg },
            FormatTrait::UpperExp => quote! { UpperExpArg },
        }
        .to_tokens(tokens)
    }
}
