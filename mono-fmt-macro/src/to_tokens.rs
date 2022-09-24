use std::cell::Cell;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};

use crate::{
    format::{
        Align, Count, Format, FormatArg, FormatArgRef, FormatTrait, FormatterArgs, Piece, Sign,
    },
    Input,
};

pub(crate) struct Scoped<'a, T> {
    input: &'a Input,
    current_position: &'a Cell<usize>,
    inner: &'a T,
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

        tokens.extend(quote! {
            (
                #(#parts),*
            )
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

                self.input.positional_args[current_position].to_token_stream()
            }
            Some(FormatArgRef::Positional(idx)) => {
                self.input.positional_args[idx].to_token_stream()
            }
            Some(FormatArgRef::Named(name)) => self
                .input
                .named_args
                .iter()
                .find(|(arg, _)| arg == name)
                .map(|(_, expr)| expr.to_token_stream())
                .unwrap_or_else(|| Ident::new(name, Span::call_site()).to_token_stream()),
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
