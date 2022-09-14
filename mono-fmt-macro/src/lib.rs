//! a bunch of this code is adapted from [stylish](https://github.com/Nullus157/stylish-rs)
#![allow(dead_code, unreachable_code, unused_variables)]

use format::Parse as _;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, ExprAssign, ExprPath, Ident, LitStr, PathArguments, Result, Token,
};

mod format;
mod to_tokens;

struct Input {
    crate_ident: Ident,
    format_str: String,
    positional_args: Vec<Expr>,
    named_args: Vec<(Ident, Expr)>,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let crate_ident = input.parse()?;

        let format_str = input.parse::<LitStr>()?.value();

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
            crate_ident,
            format_str,
            positional_args,
            named_args,
        })
    }
}

fn format_args_impl(input: Input) -> syn::Result<TokenStream> {
    todo!();
    let (_, fmt_parts) = format::Format::parse(&input.format_str).unwrap();

    Ok(quote! {
        fmt_parts
    }
    .into())
}

#[proc_macro]
pub fn __format_args(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as Input);

    match format_args_impl(input) {
        Ok(tt) => tt,
        Err(err) => err.to_compile_error().into(),
    }
}
