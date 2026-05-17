use super::attr::{SerdeAttrs, option_inner_type, serde_attrs, tyzen_attrs};
use super::case::{RenameRule, apply_rename_rule};
use super::metadata::{load_enum_metadata, save_enum_metadata};
use crate::utils::is_known_binary_type;
use quote::quote;
use std::collections::HashMap;
use syn::{Data, DeriveInput, Field, Fields};

/// Generates a `TypeStructure` metadata block for a given Rust type.
///
/// This is the entry point for metadata collection during `#[derive(Type)]`.
/// It handles:
/// - Structs (Named, Unnamed, Unit, and Transparent)
/// - Enums (including Tagged/Untagged and Template inheritance)
pub fn structure_definition(
    input: &DeriveInput,
    generic_params: &[String],
) -> proc_macro2::TokenStream {
    let serde = serde_attrs(&input.attrs);
    match &input.data {
        Data::Struct(data) => {
            let tyzen = tyzen_attrs(&input.attrs);
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
                        let fields =
                            struct_fields_meta(&data.fields, serde.rename_all, &tyzen, generic_params);
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
            let tyzen = tyzen_attrs(&input.attrs);
            let variants = enum_variants_meta(
                &input.ident.to_string(),
                &data.variants,
                &serde,
                &tyzen,
                generic_params,
            );
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
            let meta_name = match tyzen.meta_name {
                Some(m) => quote! { Some(#m) },
                None => quote! { None },
            };

            quote! {
                || ::tyzen::meta::TypeStructure::Enum(::tyzen::meta::EnumMeta {
                    variants: &[#(#variants),*],
                    tag: #tag_quote,
                    content: #content_quote,
                    untagged: #untagged,
                    meta_name: #meta_name,
                })
            }
        }
        _ => quote! { || ::tyzen::meta::TypeStructure::Unit },
    }
}

/// Collects metadata for all fields in a struct or variant.
fn struct_fields_meta(
    fields: &Fields,
    rename_all: Option<RenameRule>,
    container_tyzen: &super::attr::TyzenAttrs,
    generic_params: &[String],
) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .filter_map(|field| field_meta(field, rename_all, container_tyzen, generic_params))
        .collect()
}

/// Extracts metadata for a single field, including Serde attributes and TypeScript type mapping.
fn field_meta(
    field: &Field,
    rename_all: Option<RenameRule>,
    container_tyzen: &super::attr::TyzenAttrs,
    generic_params: &[String],
) -> Option<proc_macro2::TokenStream> {
    let serde = serde_attrs(&field.attrs);
    if serde.skip {
        return None;
    }
    
    let is_option = option_inner_type(&field.ty).is_some();

    let field_name = field
        .ident
        .as_ref()
        .map(|i| i.to_string())
        .unwrap_or_default();
    let renamed_name = ts_name(&field_name, serde.rename.clone(), rename_all);
    
    let tyzen = tyzen_attrs(&field.attrs);
    let optional = (tyzen.optional || (container_tyzen.optional && is_option) || serde.default) && !tyzen.nullable;
    let flattened = serde.flatten;
    let is_binary = tyzen.binary || serde.binary || is_known_binary_type(&field.ty);

    let ty = if optional && is_option {
        option_inner_type(&field.ty).unwrap()
    } else {
        &field.ty
    };

    let flatten_base_name = if flattened {
        let syn::Type::Path(p) = ty else {
            return Some(
                syn::Error::new_spanned(
                    &field.ty,
                    "#[serde(flatten)] field must be a named type path",
                )
                .into_compile_error(),
            );
        };
        let Some(last_seg) = p.path.segments.last() else {
            return Some(
                syn::Error::new_spanned(
                    &field.ty,
                    "#[serde(flatten)] field has an empty type path",
                )
                .into_compile_error(),
            );
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
            is_binary: #is_binary,
        }
    })
}

/// Collects metadata for enum variants, supporting inheritance from templates.
///
/// The merging priority is:
/// 1. Attributes inherited from a template (via `#[tyzen(apply = TemplateName)]`)
/// 2. Local variant overrides (via `#[tyzen(key = "value")]`)
///
/// After collection, the metadata is persisted to a shadow file to allow other enums
/// in the same crate to use this enum as a template.
fn enum_variants_meta(
    enum_name: &str,
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::Token![,]>,
    container_serde: &SerdeAttrs,
    container_tyzen: &super::attr::TyzenAttrs,
    generic_params: &[String],
) -> Vec<proc_macro2::TokenStream> {
    let mut collected_meta = Vec::new();

    // Load template if specified
    let template_meta = if let Some(apply_path) = &container_tyzen.apply {
        let template_name = apply_path.segments.last().unwrap().ident.to_string();
        match load_enum_metadata(&template_name) {
            Some(meta) => Some(meta),
            None => {
                return vec![syn::Error::new_spanned(
                    apply_path,
                    format!(
                        "Tyzen Template '{}' not found. Ensure it derives 'Type' in this crate before being applied.", 
                        template_name
                    )
                ).into_compile_error()];
            }
        }
    } else {
        None
    };

    let tokens: Vec<_> = variants
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

            let tyzen = tyzen_attrs(&variant.attrs);
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
                    let field_rename_all = variant_serde
                        .rename_all
                        .or(container_serde.rename_all_fields);
                    let fields =
                        struct_fields_meta(&variant.fields, field_rename_all, &tyzen, generic_params);
                    quote! { ::tyzen::meta::VariantFields::Named(&[#(#fields),*]) }
                }
            };

            let mut final_attrs = HashMap::new();

            // 1. Apply from template
            if let Some(template) = &template_meta {
                if let Some(v_meta) = template.get(&variant.ident.to_string()) {
                    for (k, v) in v_meta {
                        final_attrs.insert(k.clone(), v.clone());
                    }
                }
            }

            // 2. Local overrides
            for (k, v) in &tyzen.variant_meta {
                final_attrs.insert(k.clone(), v.clone());
            }

            let attrs_vec: Vec<_> = final_attrs.into_iter().collect();
            collected_meta.push((variant.ident.to_string(), attrs_vec.clone()));

            let attrs_tokens = attrs_vec.iter().map(|(k, v)| {
                quote! { (#k, #v) }
            });

            Some(quote! {
                ::tyzen::meta::VariantMeta {
                    name: #variant_name,
                    fields: #fields,
                    attrs: &[#(#attrs_tokens),*],
                }
            })
        })
        .collect();

    // Save metadata for others to use
    save_enum_metadata(enum_name, &collected_meta);

    tokens
}

/// Applies renaming rules (Serde) to a name string.
fn ts_name(name: &str, rename: Option<String>, rename_all: Option<RenameRule>) -> String {
    if let Some(rename) = rename {
        rename
    } else if let Some(rule) = rename_all {
        apply_rename_rule(name, rule)
    } else {
        name.to_string()
    }
}

/// Resolves the TypeScript type name for a Rust type.
///
/// Handles:
/// - Generic parameters (maps to their names as strings)
/// - Common Rust types (Vec, Option, Result)
/// - Custom types (calls their `TsType::ts_name()` implementation)
fn ts_type_name(ty: &syn::Type, generic_params: &[String]) -> proc_macro2::TokenStream {
    if let Some(inner) = channel_inner_type(ty) {
        let inner_ts = ts_type_name(inner, generic_params);
        return quote! { format!("Channel<{}>", #inner_ts) };
    }

    if is_known_binary_type(ty) {
        return quote! { "Uint8Array".to_string() };
    }

    if let Some(name) = get_generic_aware_name(ty, generic_params) {
        quote! { #name }
    } else {
        quote! { <#ty as ::tyzen::TsType>::ts_name() }
    }
}

/// Detects if a type is a `tauri::ipc::Channel` and extracts its inner type.
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

/// Attempts to resolve a type name while considering generic parameters.
///
/// This function specializes common Rust types to their TypeScript equivalents:
/// - `Vec<T>` -> `T[]`
/// - `Option<T>` -> `T | null`
/// - `Result<T, E>` -> `Result<T, E>` (custom union)
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
                    "Result" => match inner_names.len() {
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
                    },
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

/// Recursively checks if a type or any of its generic arguments contain
/// a parameter from the current generic scope.
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

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_field_optional_logic() {
        let struct_tyzen_none = tyzen_attrs(&[]);
        let struct_tyzen_opt = tyzen_attrs(&[parse_quote!(#[tyzen(optional)])]);

        // 1. Option field, no struct-level optional -> required (null allowed)
        let field_opt: syn::Field = parse_quote!(pub status: Option<String>);
        let serde = serde_attrs(&field_opt.attrs);
        let tyzen = tyzen_attrs(&field_opt.attrs);
        let is_option = option_inner_type(&field_opt.ty).is_some();
        let optional = tyzen.optional || (struct_tyzen_none.optional && is_option) || serde.default;
        assert!(!optional);

        // 2. Option field, WITH struct-level optional -> optional (?)
        let optional = tyzen.optional || (struct_tyzen_opt.optional && is_option) || serde.default;
        assert!(optional);

        // 3. Non-option field, WITH struct-level optional -> still required
        let field_req: syn::Field = parse_quote!(pub title: String);
        let serde = serde_attrs(&field_req.attrs);
        let tyzen = tyzen_attrs(&field_req.attrs);
        let is_option = option_inner_type(&field_req.ty).is_some();
        let optional = tyzen.optional || (struct_tyzen_opt.optional && is_option) || serde.default;
        assert!(!optional);

        // 4. Non-option field, with explicit field-level optional -> optional (logic level)
        let field_req_opt: syn::Field = parse_quote!(#[tyzen(optional)] pub title: String);
        let tyzen = tyzen_attrs(&field_req_opt.attrs);
        let is_option = option_inner_type(&field_req_opt.ty).is_some();
        let optional = tyzen.optional || (struct_tyzen_opt.optional && is_option) || serde.default;
        assert!(optional);

        // 5. Option field, WITH struct-level optional, BUT WITH field-level nullable -> required (T | null)
        let field_nullable: syn::Field = parse_quote!(#[tyzen(nullable)] pub status: Option<String>);
        let serde = serde_attrs(&field_nullable.attrs);
        let tyzen = tyzen_attrs(&field_nullable.attrs);
        let is_option = option_inner_type(&field_nullable.ty).is_some();
        let optional = (tyzen.optional || (struct_tyzen_opt.optional && is_option) || serde.default) && !tyzen.nullable;
        assert!(!optional);
    }
}
