use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Expr, Lit, parse_macro_input};

pub fn derive_event(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    // Check for #[tyzen(name = "...")]
    let mut custom_name = None;
    for attr in &input.attrs {
        if attr.path().is_ident("tyzen") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    let value = meta.value()?;
                    let expr: Expr = value.parse()?;
                    if let Expr::Lit(expr_lit) = expr {
                        if let Lit::Str(lit_str) = expr_lit.lit {
                            custom_name = Some(lit_str.value());
                        }
                    }
                }
                Ok(())
            });
        }
    }

    let name = if let Some(n) = custom_name {
        n
    } else {
        let mut s = input.ident.to_string();
        if s.ends_with("Event") {
            s.truncate(s.len() - 5);
        }
        to_kebab_case(&s)
    };

    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #struct_name #ty_generics #where_clause {
            pub fn emit<R: ::tauri::Runtime>(&self, handle: &impl ::tauri::Emitter<R>) -> ::tauri::Result<()> {
                handle.emit(#name, self)
            }
        }

        ::tyzen::inventory::submit! {
            ::tyzen::EventMeta {
                name: #name,
                payload_type: <#struct_name #ty_generics as ::tyzen::TsType>::ts_name,
            }
        }
    };

    TokenStream::from(expanded)
}

fn to_kebab_case(s: &str) -> String {
    let mut kebab = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            kebab.push('-');
        }
        kebab.push(c.to_ascii_lowercase());
    }
    kebab
}
