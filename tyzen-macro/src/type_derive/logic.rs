use quote::quote;
use syn::{Data, DeriveInput, Field, Fields};
use super::attr::{SerdeAttrs, has_tyzen_optional, option_inner_type, serde_attrs};
use super::case::{RenameRule, apply_rename_rule};

pub fn structure_definition(input: &DeriveInput, generic_params: &[String]) -> proc_macro2::TokenStream {
    let serde = serde_attrs(&input.attrs);
    match &input.data {
        Data::Struct(data) => {
            if serde.transparent {
                let Some(field) = data.fields.iter().next() else {
                    return syn::Error::new_spanned(
                        &input.ident,
                        "#[serde(transparent)] requires exactly one field",
                    )
                    .into_compile_error();
                };
                let ty_name = ts_type_name(&field.ty, generic_params);
                quote! {
                    || ::tyzen::meta::TypeStructure::Transparent(#ty_name)
                }
            } else {
                match &data.fields {
                    Fields::Named(_) => {
                        let fields = struct_fields_meta(&data.fields, serde.rename_all, generic_params);
                        quote! {
                            || ::tyzen::meta::TypeStructure::Struct(::tyzen::meta::StructMeta {
                                fields: &[#(#fields),*]
                            })
                        }
                    }
                    Fields::Unnamed(f) => {
                        let types = f.unnamed.iter().map(|field| {
                            let ty_name = ts_type_name(&field.ty, generic_params);
                            quote! { || #ty_name }
                        });
                        quote! {
                            || ::tyzen::meta::TypeStructure::Tuple(&[#(#types),*])
                        }
                    }
                    Fields::Unit => {
                        quote! { || ::tyzen::meta::TypeStructure::Unit }
                    }
                }
            }
        }
        Data::Enum(data) => {
            let variants = enum_variants_meta(&data.variants, &serde, generic_params);
            let tag = serde.tag.as_deref();
            let tag_quote = match tag {
                Some(t) => quote! { Some(#t) },
                None => quote! { None },
            };
            let content = serde.content.as_deref();
            let content_quote = match content {
                Some(c) => quote! { Some(#c) },
                None => quote! { None },
            };
            let untagged = serde.untagged;
            quote! {
                || ::tyzen::meta::TypeStructure::Enum(::tyzen::meta::EnumMeta {
                    variants: &[#(#variants),*],
                    tag: #tag_quote,
                    content: #content_quote,
                    untagged: #untagged,
                })
            }
        }
        _ => quote! { || ::tyzen::meta::TypeStructure::Unit },
    }
}

fn struct_fields_meta(
    fields: &Fields,
    rename_all: Option<RenameRule>,
    generic_params: &[String],
) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .filter_map(|field| field_meta(field, rename_all, generic_params))
        .collect()
}

fn field_meta(field: &Field, rename_all: Option<RenameRule>, generic_params: &[String]) -> Option<proc_macro2::TokenStream> {
    let serde = serde_attrs(&field.attrs);
    if serde.skip {
        return None;
    }

    let field_name = field.ident.as_ref().map(|i| i.to_string()).unwrap_or_default();
    let renamed_name = ts_name(&field_name, serde.rename.clone(), rename_all);
    let optional = has_tyzen_optional(&field.attrs) || serde.default;
    let flattened = serde.flatten;
    
    let ty = if has_tyzen_optional(&field.attrs) {
        option_inner_type(&field.ty).unwrap()
    } else {
        &field.ty
    };

    let flatten_base_name = if flattened {
        let syn::Type::Path(p) = ty else {
            return Some(syn::Error::new_spanned(
                &field.ty,
                "#[serde(flatten)] field must be a named type path",
            )
            .into_compile_error());
        };
        let Some(last_seg) = p.path.segments.last() else {
            return Some(syn::Error::new_spanned(
                &field.ty,
                "#[serde(flatten)] field has an empty type path",
            )
            .into_compile_error());
        };
        let s = last_seg.ident.to_string();
        quote! { Some(#s) }
    } else {
        quote! { None }
    };

    let ty_name = ts_type_name(ty, generic_params);

    Some(quote! {
        ::tyzen::meta::FieldMeta {
            name: #renamed_name,
            ty_name: || #ty_name,
            optional: #optional,
            flattened: #flattened,
            flatten_base_name: #flatten_base_name,
        }
    })
}

fn enum_variants_meta(
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::Token![,]>,
    container_serde: &SerdeAttrs,
    generic_params: &[String],
) -> Vec<proc_macro2::TokenStream> {
    variants
        .iter()
        .filter_map(|variant| {
            let variant_serde = serde_attrs(&variant.attrs);
            if variant_serde.skip {
                return None;
            }

            let variant_name = ts_name(
                &variant.ident.to_string(),
                variant_serde.rename.clone(),
                container_serde.rename_all,
            );

            let fields = match &variant.fields {
                Fields::Unit => quote! { ::tyzen::meta::VariantFields::Unit },
                Fields::Unnamed(f) => {
                    let types = f.unnamed.iter().map(|field| {
                        let ty_name = ts_type_name(&field.ty, generic_params);
                        quote! { || #ty_name }
                    });
                    quote! { ::tyzen::meta::VariantFields::Unnamed(&[#(#types),*]) }
                }
                Fields::Named(_f) => {
                    let field_rename_all = variant_serde.rename_all.or(container_serde.rename_all_fields);
                    let fields = struct_fields_meta(&variant.fields, field_rename_all, generic_params);
                    quote! { ::tyzen::meta::VariantFields::Named(&[#(#fields),*]) }
                }
            };

            Some(quote! {
                ::tyzen::meta::VariantMeta {
                    name: #variant_name,
                    fields: #fields,
                }
            })
        })
        .collect()
}

fn ts_name(name: &str, rename: Option<String>, rename_all: Option<RenameRule>) -> String {
    if let Some(rename) = rename {
        rename
    } else if let Some(rule) = rename_all {
        apply_rename_rule(name, rule)
    } else {
        name.to_string()
    }
}

fn ts_type_name(ty: &syn::Type, generic_params: &[String]) -> proc_macro2::TokenStream {
    if let Some(inner) = channel_inner_type(ty) {
        let inner_ts = ts_type_name(inner, generic_params);
        return quote! { format!("Channel<{}>", #inner_ts) };
    }

    if let Some(name) = get_generic_aware_name(ty, generic_params) {
        quote! { #name }
    } else {
        quote! { <#ty as ::tyzen::TsType>::ts_name() }
    }
}

fn channel_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(type_path) = ty else {
        return None;
    };

    let segment = type_path.path.segments.last()?;
    if segment.ident != "Channel" {
        return None;
    }

    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };

    args.args.iter().find_map(|arg| match arg {
        syn::GenericArgument::Type(inner) => Some(inner),
        _ => None,
    })
}

fn get_generic_aware_name(
    ty: &syn::Type,
    generic_params: &[String],
) -> Option<proc_macro2::TokenStream> {
    match ty {
        syn::Type::Path(p) => {
            let segment = p.path.segments.last()?;
            let ident_str = segment.ident.to_string();

            if generic_params.contains(&ident_str) {
                return Some(quote! { #ident_str.to_string() });
            }

            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                let mut has_generic = false;
                let inner_names = args
                    .args
                    .iter()
                    .filter_map(|arg| {
                        if let syn::GenericArgument::Type(inner_ty) = arg {
                            if is_generic(inner_ty, generic_params) {
                                has_generic = true;
                                return Some(ts_type_name(inner_ty, generic_params));
                            }
                            Some(quote! { <#inner_ty as ::tyzen::TsType>::ts_name() })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                if !has_generic || inner_names.is_empty() {
                    return None;
                }

                return match ident_str.as_str() {
                    "Vec" | "VecDeque" | "LinkedList" => {
                        let inner = &inner_names[0];
                        Some(quote! { format!("{}[]", #inner) })
                    }
                    "Option" => {
                        let inner = &inner_names[0];
                        Some(quote! { format!("{} | null", #inner) })
                    }
                    "Result" => {
                        match inner_names.len() {
                            1 => {
                                let ok = &inner_names[0];
                                Some(quote! { format!("Result<{}, string>", #ok) })
                            }
                            2 => {
                                let ok = &inner_names[0];
                                let err = &inner_names[1];
                                Some(quote! { format!("Result<{}, {}>", #ok, #err) })
                            }
                            _ => None,
                        }
                    }
                    _ => Some(
                        quote! { format!("{}<{}>", #ident_str, vec![#(#inner_names),*].join(", ")) },
                    ),
                };
            }
            None
        }
        _ => None,
    }
}

fn is_generic(ty: &syn::Type, generic_params: &[String]) -> bool {
    match ty {
        syn::Type::Path(p) => {
            if let Some(ident) = p.path.get_ident()
                && generic_params.contains(&ident.to_string())
            {
                return true;
            }
            p.path.segments.iter().any(|s| {
                if let syn::PathArguments::AngleBracketed(args) = &s.arguments {
                    args.args.iter().any(|arg| {
                        if let syn::GenericArgument::Type(inner) = arg {
                            is_generic(inner, generic_params)
                        } else {
                            false
                        }
                    })
                } else {
                    false
                }
            })
        }
        _ => false,
    }
}
