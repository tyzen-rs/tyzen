use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Field, Fields, GenericArgument, PathArguments, Type,
    parse_macro_input,
};

#[derive(Clone, Copy)]
enum RenameRule {
    Lowercase,
    Uppercase,
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    ScreamingKebabCase,
}

#[derive(Default)]
struct SerdeAttrs {
    rename: Option<String>,
    rename_all: Option<RenameRule>,
    rename_all_fields: Option<RenameRule>,
    tag: Option<String>,
    content: Option<String>,
    untagged: bool,
    transparent: bool,
    skip: bool,
}

pub fn derive_type(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    if let Some(error) = validate(&input) {
        return error.into_compile_error().into();
    }

    let name = &input.ident;
    let name_str = name.to_string();
    let ts_def = ts_definition(&input);

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

    quote! {
        impl #impl_generics ::tyzen::TsType for #name #ty_generics #where_clause {
            fn ts_name() -> String {
                #ts_name_impl
            }
        }

        ::tyzen::inventory::submit! {
            ::tyzen::TypeMeta {
                name: #name_str,
                ts_def: #ts_def,
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

fn ts_definition(input: &DeriveInput) -> proc_macro2::TokenStream {
    let name = input.ident.to_string();
    let serde = serde_attrs(&input.attrs);

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

    let generic_suffix = if generic_params.is_empty() {
        "".to_string()
    } else {
        format!("<{}>", generic_params.join(", "))
    };

    match &input.data {
        Data::Struct(data) => struct_definition(
            &name,
            &generic_suffix,
            &data.fields,
            &serde,
            &generic_params,
        ),
        Data::Enum(data) => enum_definition(
            &name,
            &generic_suffix,
            &data.variants,
            &serde,
            &generic_params,
        ),
        _ => quote! { || format!("export type {}{} = unknown", #name, #generic_suffix) },
    }
}

fn struct_definition(
    name: &str,
    generic_suffix: &str,
    fields: &Fields,
    serde: &SerdeAttrs,
    generic_params: &[String],
) -> proc_macro2::TokenStream {
    if serde.transparent {
        let field = fields.iter().next().expect("transparent struct must have one field");
        let ty = ts_type_name(&field.ty, generic_params);
        return quote! {
            || format!("export type {}{} = {}", #name, #generic_suffix, #ty)
        };
    }

    match fields {
        Fields::Named(fields) => {
            let fields = fields
                .named
                .iter()
                .filter_map(|field| named_field_definition(field, serde.rename_all, generic_params))
                .collect::<Vec<_>>();

            quote! {
                || {
                    let fields = vec![#(#fields),*];
                    format!("export type {}{} = {{ {} }}", #name, #generic_suffix, fields.join(", "))
                }
            }
        }
        Fields::Unnamed(fields) => {
            let types = fields
                .unnamed
                .iter()
                .map(|field| ts_type_name(&field.ty, generic_params))
                .collect::<Vec<_>>();

            if types.len() == 1 {
                quote! {
                    || format!("export type {}{} = {}", #name, #generic_suffix, #(#types),*)
                }
            } else {
                quote! {
                    || {
                        let fields = vec![#(#types),*];
                        format!("export type {}{} = [{}]", #name, #generic_suffix, fields.join(", "))
                    }
                }
            }
        }
        Fields::Unit => quote! {
            || format!("export type {}{} = null", #name, #generic_suffix)
        },
    }
}

fn enum_definition(
    name: &str,
    generic_suffix: &str,
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::Token![,]>,
    serde: &SerdeAttrs,
    generic_params: &[String],
) -> proc_macro2::TokenStream {
    let is_unit_enum = !serde.untagged && variants.iter().all(|v| matches!(v.fields, Fields::Unit));

    if is_unit_enum {
        let variant_names = variants
            .iter()
            .map(|v| {
                let variant_serde = serde_attrs(&v.attrs);
                ts_name(&v.ident.to_string(), variant_serde.rename, serde.rename_all)
            })
            .collect::<Vec<_>>();

        quote! {
            || {
                let names = vec![#(#variant_names),*];
                let fields = names.iter()
                    .map(|n| format!("  {}: \"{}\"", n, n))
                    .collect::<Vec<_>>()
                    .join(",\n");
                format!("export const {} = {{\n{}\n}} as const;\nexport type {}{} = (typeof {})[keyof typeof {}]", #name, fields, #name, #generic_suffix, #name, #name)
            }
        }
    } else {
        let variants = variants
            .iter()
            .filter_map(|variant| enum_variant_definition(variant, serde, generic_params))
            .collect::<Vec<_>>();

        quote! {
            || {
                let variants = vec![#(#variants),*];
                format!("export type {}{} = {}", #name, #generic_suffix, variants.join(" | "))
            }
        }
    }
}

fn enum_variant_definition(
    variant: &syn::Variant,
    container_serde: &SerdeAttrs,
    generic_params: &[String],
) -> Option<proc_macro2::TokenStream> {
    let variant_serde = serde_attrs(&variant.attrs);
    if variant_serde.skip {
        return None;
    }

    let variant_name = ts_name(
        &variant.ident.to_string(),
        variant_serde.rename,
        container_serde.rename_all,
    );
    let field_rename_all = variant_serde
        .rename_all
        .or(container_serde.rename_all_fields);
    let tag_name = container_serde.tag.as_deref().unwrap_or("tag");
    let content_name = container_serde.content.as_deref();

    Some(match &variant.fields {
        Fields::Unit => {
            if container_serde.untagged {
                quote! { "null".to_string() }
            } else {
                quote! {
                    format!("{{ {}: \"{}\" }}", #tag_name, #variant_name)
                }
            }
        }
        Fields::Unnamed(fields) => {
            let values = fields
                .unnamed
                .iter()
                .map(|field| ts_type_name(&field.ty, generic_params))
                .collect::<Vec<_>>();

            let values_ts = if values.len() == 1 {
                quote! { #(#values),* }
            } else {
                quote! {
                    {
                        let values = vec![#(#values),*];
                        format!("[{}]", values.join(", "))
                    }
                }
            };

            if container_serde.untagged {
                quote! { #values_ts }
            } else if let Some(content) = content_name {
                quote! {
                    format!("{{ {}: \"{}\", {}: {} }}", #tag_name, #variant_name, #content, #values_ts)
                }
            } else {
                quote! {
                    format!("{{ {}: \"{}\", value: {} }}", #tag_name, #variant_name, #values_ts)
                }
            }
        }
        Fields::Named(fields) => {
            let fields = fields
                .named
                .iter()
                .filter_map(|field| named_field_definition(field, field_rename_all, generic_params))
                .collect::<Vec<_>>();

            if container_serde.untagged {
                quote! {
                    format!("{{ {} }}", vec![#(#fields),*].join(", "))
                }
            } else if let Some(content) = content_name {
                quote! {
                    format!("{{ {}: \"{}\", {}: {{ {} }} }}", #tag_name, #variant_name, #content, vec![#(#fields),*].join(", "))
                }
            } else {
                quote! {
                    format!("{{ {}: \"{}\", {} }}", #tag_name, #variant_name, vec![#(#fields),*].join(", "))
                }
            }
        }
    })
}

fn named_field_definition(
    field: &Field,
    rename_all: Option<RenameRule>,
    generic_params: &[String],
) -> Option<proc_macro2::TokenStream> {
    let serde = serde_attrs(&field.attrs);
    if serde.skip {
        return None;
    }

    let field_name = field.ident.as_ref().unwrap().to_string();
    let field_name = ts_name(&field_name, serde.rename, rename_all);
    let optional = has_tyzen_optional(&field.attrs);
    let label = if optional {
        format!("{field_name}?")
    } else {
        field_name
    };
    let ty = if optional {
        option_inner_type(&field.ty).unwrap()
    } else {
        &field.ty
    };

    let ty_name = ts_type_name(ty, generic_params);

    Some(quote! {
        format!("{}: {}", #label, #ty_name)
    })
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
                    "Result" if inner_names.len() == 2 => {
                        let ok = &inner_names[0];
                        let err = &inner_names[1];
                        Some(quote! { format!("Result<{}, {}>", #ok, #err) })
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

fn ts_name(name: &str, rename: Option<String>, rename_all: Option<RenameRule>) -> String {
    if let Some(rename) = rename {
        rename
    } else if let Some(rule) = rename_all {
        apply_rename_rule(name, rule)
    } else {
        name.to_string()
    }
}

fn serde_attrs(attrs: &[Attribute]) -> SerdeAttrs {
    let mut serde = SerdeAttrs::default();

    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip")
                || meta.path.is_ident("skip_serializing")
                || meta.path.is_ident("skip_deserializing")
            {
                serde.skip = true;
                return Ok(());
            }

            if meta.path.is_ident("rename") {
                let value = meta.value()?.parse::<syn::LitStr>()?;
                serde.rename = Some(value.value());
                return Ok(());
            }

            if meta.path.is_ident("rename_all") {
                let value = meta.value()?.parse::<syn::LitStr>()?;
                serde.rename_all = rename_rule(&value.value());
                return Ok(());
            }

            if meta.path.is_ident("rename_all_fields") {
                let value = meta.value()?.parse::<syn::LitStr>()?;
                serde.rename_all_fields = rename_rule(&value.value());
                return Ok(());
            }

            if meta.path.is_ident("tag") {
                let value = meta.value()?.parse::<syn::LitStr>()?;
                serde.tag = Some(value.value());
                return Ok(());
            }

            if meta.path.is_ident("content") {
                let value = meta.value()?.parse::<syn::LitStr>()?;
                serde.content = Some(value.value());
                return Ok(());
            }

            if meta.path.is_ident("untagged") {
                serde.untagged = true;
                return Ok(());
            }

            if meta.path.is_ident("transparent") {
                serde.transparent = true;
                return Ok(());
            }

            Ok(())
        });
    }

    serde
}

fn rename_rule(rule: &str) -> Option<RenameRule> {
    Some(match rule {
        "lowercase" => RenameRule::Lowercase,
        "UPPERCASE" => RenameRule::Uppercase,
        "PascalCase" => RenameRule::PascalCase,
        "camelCase" => RenameRule::CamelCase,
        "snake_case" => RenameRule::SnakeCase,
        "SCREAMING_SNAKE_CASE" => RenameRule::ScreamingSnakeCase,
        "kebab-case" => RenameRule::KebabCase,
        "SCREAMING-KEBAB-CASE" => RenameRule::ScreamingKebabCase,
        _ => return None,
    })
}

fn apply_rename_rule(name: &str, rule: RenameRule) -> String {
    let words = words(name);

    match rule {
        RenameRule::Lowercase => words.join("").to_lowercase(),
        RenameRule::Uppercase => words.join("").to_uppercase(),
        RenameRule::PascalCase => words.iter().map(|word| capitalize(word)).collect(),
        RenameRule::CamelCase => {
            let mut out = String::new();
            for (index, word) in words.iter().enumerate() {
                if index == 0 {
                    out.push_str(&word.to_lowercase());
                } else {
                    out.push_str(&capitalize(word));
                }
            }
            out
        }
        RenameRule::SnakeCase => words.join("_").to_lowercase(),
        RenameRule::ScreamingSnakeCase => words.join("_").to_uppercase(),
        RenameRule::KebabCase => words.join("-").to_lowercase(),
        RenameRule::ScreamingKebabCase => words.join("-").to_uppercase(),
    }
}

fn words(name: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();

    for ch in name.chars() {
        if ch == '_' || ch == '-' {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            continue;
        }

        if ch.is_uppercase() && !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }

        current.push(ch);
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
        None => String::new(),
    }
}

fn has_tyzen_optional(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("tyzen")
            && attr
                .parse_args::<syn::Ident>()
                .map(|ident| ident == "optional")
                .unwrap_or(false)
    })
}

fn option_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };

    let segment = type_path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }

    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };

    args.args.iter().find_map(|arg| match arg {
        GenericArgument::Type(inner) => Some(inner),
        _ => None,
    })
}
