//! a bunch of this code is adapted from [stylish](https://github.com/Nullus157/stylish-rs)
#![allow(dead_code, unreachable_code, unused_variables)]

use std::cell::Cell;

use format::Parse as _;
use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, ExprAssign, ExprPath, Ident, LitStr, PathArguments, Result, Token,
};
use to_tokens::Scoped;

mod format;
mod to_tokens;

struct Input {
    prefix: proc_macro2::TokenStream,
    format_str: LitStr,
    positional_args: Vec<Expr>,
    named_args: Vec<(Ident, Expr)>,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let crate_ident = input.parse::<syn::Path>()?;
        let prefix = quote! { #crate_ident::_private };

        let format_str = input.parse::<LitStr>()?;

        let mut positional_args = Vec::new();
        let mut named_args = Vec::new();
        let mut onto_named = false;
        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            let expr = input.parse::<Expr>()?;
            match expr {
                Expr::Assign(ExprAssign { left, right, .. })
                    if matches!(
                        &*left,
                        Expr::Path(ExprPath { path, .. })
                            if path.segments.len() == 1 && matches!(path.segments[0].arguments, PathArguments::None)
                    ) =>
                {
                    let ident = if let Expr::Path(ExprPath { mut path, .. }) = *left {
                        path.segments.pop().unwrap().into_value().ident
                    } else {
                        panic!()
                    };
                    named_args.push((ident, *right));
                    onto_named = true;
                }
                expr => {
                    if onto_named {
                        panic!("positional arg after named")
                    }
                    positional_args.push(expr);
                }
            }
        }
        Ok(Self {
            prefix,
            format_str,
            positional_args,
            named_args,
        })
    }
}

fn format_args_impl(input: Input) -> syn::Result<TokenStream> {
    let str = input.format_str.value();
    let (_, fmt_parts) = format::Format::parse(&str).unwrap();

    let current_position = Cell::new(0);

    Ok(Scoped::new(&input, &fmt_parts, &current_position).to_token_stream().into())
}

#[proc_macro]
pub fn __format_args(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as Input);

    match format_args_impl(input) {
        Ok(tt) => tt,
        Err(err) => err.to_compile_error().into(),
    }
}
