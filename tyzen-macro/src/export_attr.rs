use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemConst, parse_macro_input};

pub fn export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemConst);
    let name = &input.ident;
    let value = &input.expr;

    let value_str = quote!(#value).to_string();
    let ns = parse_ns(_attr);
    let ns_val = match ns {
        Some(s) => quote! { Some(#s) },
        None => quote! { None },
    };

    let expanded = quote! {
        #input

        tyzen::__private::inventory::submit! {
            tyzen::ConstMeta {
                name: stringify!(#name),
                value: #value_str,
                ns: #ns_val,
                module_path: module_path!(),
            }
        }
    };

    TokenStream::from(expanded)
}

fn parse_ns(attr: TokenStream) -> Option<String> {
    if attr.is_empty() {
        return None;
    }

    let mut ns = None;
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("ns") || meta.path.is_ident("namespace") {
            let value = meta.value()?.parse::<syn::LitStr>()?;
            ns = Some(value.value());
            return Ok(());
        }
        Ok(())
    });

    use syn::parse::Parser;
    let _ = parser.parse(attr);

    ns
}
