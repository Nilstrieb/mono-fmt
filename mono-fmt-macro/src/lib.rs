use std::{iter::Peekable, str::Chars};

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, LitStr, Token,
};

struct Input {
    format_str: String,
    exprs: Punctuated<Expr, Token![,]>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let first = input.parse::<syn::LitStr>()?;

        let mut exprs = Punctuated::new();

        if !input.is_empty() {
            let _ = input.parse::<Token![,]>();
        }

        while !input.is_empty() {
            let punct = input.parse()?;
            exprs.push(punct);
            if input.is_empty() {
                break;
            }
            let value = input.parse()?;
            exprs.push(value);
        }

        Ok(Self {
            format_str: first.value(),
            exprs,
        })
    }
}

enum FmtPart {
    Literal(String),
    Debug(Expr),
    Display(Expr),
}

struct Formatter<'a, I> {
    string: Peekable<Chars<'a>>,
    exprs: I,
    fmt_parts: Vec<FmtPart>,
}

impl<'a, I> Formatter<'a, I>
where
    I: Iterator<Item = Expr>,
{
    fn parse(mut self) -> Vec<FmtPart> {
        let mut next_string = String::new();
        while let Some(char) = self.string.next() {
            match char {
                '{' => {
                    self.save_string(std::mem::take(&mut next_string));
                    if self.string.next() != Some('}') {
                        panic!("only supports display formatting!");
                    }
                    let expr = self
                        .exprs
                        .next()
                        .expect("missing argument for display formatting");
                    self.fmt_parts.push(FmtPart::Display(expr));
                }
                other => {
                    next_string.push(other);
                }
            }
        }
        self.save_string(next_string);

        self.fmt_parts
    }

    fn save_string(&mut self, string: String) {
        if string.is_empty() {
            return;
        }
        self.fmt_parts.push(FmtPart::Literal(string));
    }
}

impl ToTokens for FmtPart {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let own_tokens = match self {
            FmtPart::Literal(lit) => {
                let literal = LitStr::new(lit, Span::call_site());
                quote! { mono_fmt::_private::Str(#literal) }
            }
            FmtPart::Display(expr) => {
                quote! { mono_fmt::_private::DisplayArg(#expr) }
            }
            FmtPart::Debug(expr) => {
                quote! { mono_fmt::_private::DebugArg(#expr) }
            }
        };

        tokens.extend(own_tokens);
    }
}

#[proc_macro]
pub fn format_args(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as Input);

    let formatter = Formatter {
        string: input.format_str.chars().peekable(),
        exprs: input.exprs.into_iter(),
        fmt_parts: Vec::new(),
    };

    let fmt_parts = formatter.parse();

    quote! {
        (#(#fmt_parts),*,)
    }
    .into()
}
