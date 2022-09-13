use core::panic;
use std::{iter::Peekable, str::Chars};

use parser::FmtSpec;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, LitStr, Token,
};

mod parser;

// TODO: Rewrite using state machine please

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

#[derive(Debug, PartialEq)]
struct Advanced {
    width: Option<usize>,
    fill: Option<char>,
    align: Option<Alignment>,
}

enum FmtPart {
    Literal(String),
    Spec(FmtSpec, Expr),
}

impl std::fmt::Debug for FmtPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Literal(arg0) => f.debug_tuple("Literal").field(arg0).finish(),
            Self::Spec(spec, _) => f.debug_tuple("Spec").field(spec).finish(),
        }
    }
}

impl PartialEq for FmtPart {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Literal(a), Self::Literal(b)) => a == b,
            (Self::Spec(a, _), Self::Spec(b, _)) => a == b,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Alignment {
    Left,
    Center,
    Right,
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
    fn expect_expr(&mut self) -> Expr {
        self.exprs
            .next()
            .expect("missing argument for display formatting")
    }

    fn parse(mut self) -> Vec<FmtPart> {
        let mut next_string = String::new();
        while let Some(char) = self.string.next() {
            match char {
                '{' => {
                    let argument = self.fmt_spec().unwrap();
                    let expr = self.expect_expr();
                    self.fmt_parts.push(FmtPart::Spec(argument, expr));
                }
                other => {
                    next_string.push(other);
                }
            }
        }
        self.save_string(next_string);

        self.fmt_parts
    }

    fn fmt_spec(&mut self) -> Result<parser::FmtSpec, ()> {
        let parser = parser::FmtSpecParser::new(&mut self.string);
        parser.parse()
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
                quote! { ::mono_fmt::_private::Str(#literal) }
            }
            FmtPart::Spec(_, _) => todo!(),
        };

        tokens.extend(own_tokens);
    }
}

#[proc_macro]
pub fn format_args(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as Input);

    if false {
        parser::FmtSpecParser::new(&mut input.format_str.chars().peekable())
            .parse()
            .unwrap();
    }

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

#[cfg(test)]
mod tests {
    use syn::Expr;

    use crate::{Advanced, Alignment, FmtPart};

    fn fake_expr() -> Expr {
        syn::parse_str("1").unwrap()
    }

    fn fake_exprs(count: usize) -> Vec<Expr> {
        vec![fake_expr(); count]
    }

    fn run_test(string: &str, expr_count: usize) -> Vec<FmtPart> {
        let fmt = super::Formatter {
            string: string.chars().peekable(),
            exprs: fake_exprs(expr_count).into_iter(),
            fmt_parts: Vec::new(),
        };
        fmt.parse()
    }

    #[test]
    fn parse_fmt() {
        let string = "{:<5}";
        let parts = run_test(string, 1);
        assert_eq!(
            parts,
            vec![FmtPart::Advanced(
                Advanced {
                    width: Some(5),
                    fill: None,
                    align: Some(Alignment::Left),
                },
                fake_expr()
            )]
        );
    }
}
