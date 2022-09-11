use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Expr, parse_macro_input,
};

struct Input {
    format_str: String,
    items: Vec<Expr>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let first = input.parse::<syn::LitStr>()?;
        Ok(Self {
            format_str: first.value(),
            items: Vec::new(),
        })
    }
}

#[proc_macro]
pub fn format_args(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as Input);
    let str = input.format_str;
    quote! {
        (mono_fmt::_private::Str(#str),)
    }
    .into()
}
