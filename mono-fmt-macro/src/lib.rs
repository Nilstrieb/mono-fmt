use std::str::Chars;

use parser::{Alignment, Error, FmtSpec, FmtType};
use peekmore::{PeekMore, PeekMoreIterator};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, Ident, LitStr, Token,
};

mod parser;

struct Input {
    crate_ident: Ident,
    format_str: String,
    str_span: Span,
    exprs: Punctuated<Expr, Token![,]>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let crate_ident = input.parse()?;
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
            crate_ident,
            format_str: first.value(),
            str_span: first.span(),
            exprs,
        })
    }
}

enum FmtPart {
    Literal(Ident, String),
    Spec(Ident, FmtSpec, Expr),
}

impl std::fmt::Debug for FmtPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Literal(_, arg0) => f.debug_tuple("Literal").field(arg0).finish(),
            Self::Spec(_, spec, _) => f.debug_tuple("Spec").field(spec).finish(),
        }
    }
}

impl PartialEq for FmtPart {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Literal(_, a), Self::Literal(_, b)) => a == b,
            (Self::Spec(_ ,a, _), Self::Spec(_, b, _)) => a == b,
            _ => false,
        }
    }
}

struct Formatter<'a, I> {
    string: PeekMoreIterator<Chars<'a>>,
    crate_ident: Ident,
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

    fn parse(mut self) -> Result<Vec<FmtPart>, Error> {
        let mut next_string = String::new();
        while let Some(char) = self.string.next() {
            match char {
                '{' => {
                    self.save_string(std::mem::take(&mut next_string));
                    let argument = self.fmt_spec()?;
                    let expr = self.expect_expr();
                    self.fmt_parts
                        .push(FmtPart::Spec(self.crate_ident.clone(), argument, expr));
                }
                other => {
                    next_string.push(other);
                }
            }
        }
        self.save_string(next_string);

        Ok(self.fmt_parts)
    }

    fn fmt_spec(&mut self) -> Result<parser::FmtSpec, Error> {
        let parser = parser::FmtSpecParser::new(&mut self.string);
        parser.parse()
    }

    fn save_string(&mut self, string: String) {
        if string.is_empty() {
            return;
        }
        self.fmt_parts
            .push(FmtPart::Literal(self.crate_ident.clone(), string));
    }
}

impl ToTokens for Alignment {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            Self::Center => quote! { WithCenterAlign },
            Self::Left => quote! { WithLeftAlign },
            Self::Right => quote! { WithRightAlign },
        })
    }
}

impl ToTokens for FmtPart {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let own_tokens = match self {
            FmtPart::Literal(crate_ident, lit) => {
                let literal = LitStr::new(lit, Span::call_site());
                quote! { #crate_ident::_private::Str(#literal) }
            }
            FmtPart::Spec(crate_ident, spec, expr) => {
                let mut opt_toks = quote! { () };

                // FIXME: Wait no we want `DebugArg::<WithUwu<WithOwo<()>>>(expr)`

                if let Some(align) = &spec.align {
                    if let Some(fill) = align.fill {
                        opt_toks = quote! { #crate_ident::_private::WithFill<#opt_toks, #fill> };
                    }
                    let alignment = align.kind;
                    opt_toks = quote! { #crate_ident::_private::#alignment<#opt_toks> };
                }

                if spec.alternate {
                    opt_toks = quote! { #crate_ident::_private::WithAlternate<_, #opt_toks };
                }

                if spec.zero {
                    todo!()
                }

                if let Some(_) = spec.width {
                    todo!()
                }

                if let Some(_) = spec.precision {
                    todo!()
                }

                match spec.kind {
                    FmtType::Default => quote! {
                        #crate_ident::_private::DisplayArg::<_, #opt_toks>(#expr, ::std::marker::PhantomData)
                    },
                    FmtType::Debug => quote! {
                        #crate_ident::_private::DebugArg::<_, #opt_toks>(#expr, ::std::marker::PhantomData)
                    },
                    _ => todo!(),
                }
            }
        };

        tokens.extend(own_tokens);
    }
}

#[proc_macro]
pub fn __format_args(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as Input);

    let formatter = Formatter {
        string: input.format_str.chars().peekmore(),
        crate_ident: input.crate_ident,
        exprs: input.exprs.into_iter(),
        fmt_parts: Vec::new(),
    };

    let fmt_parts = match formatter.parse() {
        Ok(parts) => parts,
        Err(error) => {
            return syn::Error::new(input.str_span, error.message)
                .to_compile_error()
                .into()
        }
    };

    quote! {
        (#(#fmt_parts),*,)
    }
    .into()
}

#[cfg(test)]
mod tests {
    use peekmore::PeekMore;
    use proc_macro2::{Ident, Span};
    use syn::Expr;

    use crate::{
        parser::{Align, Alignment, Argument, FmtSpec, FmtType},
        FmtPart,
    };

    fn fake_expr() -> Expr {
        syn::parse_str("1").unwrap()
    }

    fn fake_exprs(count: usize) -> Vec<Expr> {
        vec![fake_expr(); count]
    }

    fn crate_ident() -> Ident {
        Ident::new("mono_fmt", Span::call_site())
    }

    fn run_test(string: &str, expr_count: usize) -> Vec<FmtPart> {
        let fmt = super::Formatter {
            string: string.chars().peekmore(),
            crate_ident: crate_ident(),
            exprs: fake_exprs(expr_count).into_iter(),
            fmt_parts: Vec::new(),
        };
        fmt.parse().unwrap()
    }

    #[test]
    fn empty() {
        let parts = run_test("{}", 1);
        assert_eq!(
            parts,
            vec![FmtPart::Spec(
                crate_ident(),
                FmtSpec {
                    ..FmtSpec::default()
                },
                fake_expr()
            )]
        );
    }

    #[test]
    fn debug() {
        let parts = run_test("{:?}", 1);
        assert_eq!(
            parts,
            vec![FmtPart::Spec(
                crate_ident(),
                FmtSpec {
                    kind: FmtType::Debug,
                    ..FmtSpec::default()
                },
                fake_expr()
            )]
        );
    }

    #[test]
    fn many() {
        let parts = run_test("{uwu:-<?}", 1);
        assert_eq!(
            parts,
            vec![FmtPart::Spec(
                crate_ident(),
                FmtSpec {
                    arg: Argument::Keyword("uwu".to_string()),
                    align: Some(Align {
                        kind: Alignment::Left,
                        fill: Some('-'),
                    }),
                    kind: FmtType::Debug,
                    ..FmtSpec::default()
                },
                fake_expr()
            )]
        );
    }
}
