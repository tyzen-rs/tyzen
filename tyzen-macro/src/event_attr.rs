use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Expr, Lit, LitStr, parse_macro_input};

use crate::type_derive::case::{RenameRule, apply_rename_rule};

pub fn event(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as DeriveInput);
    let event_name = parse_macro_input!(attr as LitStr);

    input
        .attrs
        .push(syn::parse_quote! { #[tyzen(name = #event_name)] });
    input
        .attrs
        .push(syn::parse_quote! { #[derive(::tyzen::Event)] });
    input
        .attrs
        .push(syn::parse_quote! { #[event(#event_name)] });

    quote! {
        #input
    }
    .into()
}

pub fn derive_event(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    // Check for #[tyzen(name = "...", ns = "...")]
    let mut custom_name = None;
    let mut ns = None;
    for attr in &input.attrs {
        if attr.path().is_ident("tyzen") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    let value = meta.value()?;
                    let expr: Expr = value.parse()?;
                    if let Expr::Lit(expr_lit) = expr
                        && let Lit::Str(lit_str) = expr_lit.lit
                    {
                        custom_name = Some(lit_str.value());
                    }
                }
                if meta.path.is_ident("ns") || meta.path.is_ident("namespace") {
                    let value = meta.value()?;
                    let expr: Expr = value.parse()?;
                    if let Expr::Lit(expr_lit) = expr
                        && let Lit::Str(lit_str) = expr_lit.lit
                    {
                        ns = Some(lit_str.value());
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
        apply_rename_rule(&s, RenameRule::KebabCase)
    };

    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ns_val = match ns {
        Some(s) => quote! { Some(#s) },
        None => quote! { None },
    };

    let expanded = quote! {
        impl #impl_generics #struct_name #ty_generics #where_clause {
            pub fn emit<R: ::tauri::Runtime>(&self, handle: &impl ::tauri::Emitter<R>) -> ::tauri::Result<()> {
                handle.emit(#name, self)
            }
        }

        ::tyzen::__private::inventory::submit! {
            ::tyzen::EventMeta {
                name: #name,
                payload_type: <#struct_name #ty_generics as ::tyzen::TsType>::ts_name,
                ns: #ns_val,
                module_path: module_path!(),
            }
        }
    };

    TokenStream::from(expanded)
}
