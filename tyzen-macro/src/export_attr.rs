use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemConst, parse_macro_input};

pub fn export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemConst);
    let name = &input.ident;
    let value = &input.expr;

    let value_str = quote!(#value).to_string();

    let expanded = quote! {
        #input

        tyzen::__private::inventory::submit! {
            tyzen::ConstMeta {
                name: stringify!(#name),
                value: #value_str,
            }
        }
    };

    TokenStream::from(expanded)
}
