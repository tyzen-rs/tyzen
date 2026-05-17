use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Field, parse_macro_input};

mod attr;
pub(crate) mod case;
mod logic;
mod metadata;

use attr::{has_tyzen_optional, option_inner_type, serde_attrs, tyzen_attrs};
use logic::structure_definition;
use crate::utils::is_known_binary_type;

pub fn derive_type(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let tyzen = attr::tyzen_attrs(&input.attrs);
    let ns_val = match &tyzen.ns {
        Some(s) => quote! { Some(#s) },
        None => quote! { None },
    };

    if let Some(error) = validate(&input) {
        return error.into_compile_error().into();
    }

    let name = &input.ident;
    let name_str = name.to_string();
    let generic_params: Vec<String> = input
        .generics
        .params
        .iter()
        .filter_map(|p| {
            if let syn::GenericParam::Type(t) = p {
                Some(t.ident.to_string())
            } else {
                None
            }
        })
        .collect();

    let has_binary = all_fields(&input).iter().any(|f| {
        let tyzen = tyzen_attrs(&f.attrs);
        let serde = serde_attrs(&f.attrs);
        is_known_binary_type(&f.ty) || tyzen.binary || serde.binary
    });

    let structure = structure_definition(&input, &generic_params);

    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut where_clause = where_clause
        .cloned()
        .unwrap_or_else(|| syn::parse_quote!(where));
    for param in &generics.params {
        if let syn::GenericParam::Type(type_param) = param {
            let ident = &type_param.ident;
            where_clause
                .predicates
                .push(syn::parse_quote!(#ident: ::tyzen::TsType));
        }
    }

    let ts_name_impl = if generics.params.is_empty() {
        quote! { #name_str.to_string() }
    } else {
        let param_names = generics.params.iter().filter_map(|p| {
            if let syn::GenericParam::Type(t) = p {
                let ident = &t.ident;
                Some(quote! { <#ident as ::tyzen::TsType>::ts_name() })
            } else {
                None
            }
        });
        quote! {
            format!("{}<{}>", #name_str, vec![#(#param_names),*].join(", "))
        }
    };

    let generic_params_str = if generic_params.is_empty() {
        "".to_string()
    } else {
        format!("<{}>", generic_params.join(", "))
    };

    quote! {
        impl #impl_generics ::tyzen::TsType for #name #ty_generics #where_clause {
            fn ts_name() -> String {
                #ts_name_impl
            }
        }

        ::tyzen::__private::inventory::submit! {
            ::tyzen::TypeMeta {
                name: #name_str,
                generic_params: #generic_params_str,
                structure: #structure,
                module_path: module_path!(),
                ns: #ns_val,
                has_binary: #has_binary,
            }
        }
    }
    .into()
}

fn validate(input: &DeriveInput) -> Option<syn::Error> {
    let mut error = None;

    for field in all_fields(input) {
        if has_tyzen_optional(&field.attrs) && option_inner_type(&field.ty).is_none() {
            push_error(
                &mut error,
                syn::Error::new_spanned(
                    field,
                    "#[tyzen(optional)] can only be used on Option<T> fields",
                ),
            );
        }
    }

    error
}

fn all_fields(input: &DeriveInput) -> Vec<&Field> {
    match &input.data {
        Data::Struct(data) => data.fields.iter().collect(),
        Data::Enum(data) => data
            .variants
            .iter()
            .flat_map(|variant| variant.fields.iter())
            .collect(),
        _ => Vec::new(),
    }
}

fn push_error(target: &mut Option<syn::Error>, error: syn::Error) {
    if let Some(existing) = target {
        existing.combine(error);
    } else {
        *target = Some(error);
    }
}
