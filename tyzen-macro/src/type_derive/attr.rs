use syn::{Attribute, GenericArgument, PathArguments, Type};
use super::case::{RenameRule, rename_rule};

#[derive(Default)]
pub struct SerdeAttrs {
    pub rename: Option<String>,
    pub rename_all: Option<RenameRule>,
    pub rename_all_fields: Option<RenameRule>,
    pub tag: Option<String>,
    pub content: Option<String>,
    pub untagged: bool,
    pub transparent: bool,
    pub skip: bool,
    pub flatten: bool,
    pub default: bool,
    pub alias: Vec<String>,
}

pub fn serde_attrs(attrs: &[Attribute]) -> SerdeAttrs {
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

            if meta.path.is_ident("flatten") {
                serde.flatten = true;
                return Ok(());
            }

            if meta.path.is_ident("default") {
                serde.default = true;
                return Ok(());
            }

            if meta.path.is_ident("alias") {
                let value = meta.value()?.parse::<syn::LitStr>()?;
                serde.alias.push(value.value());
                return Ok(());
            }

            Ok(())
        });
    }

    serde
}

pub fn has_tyzen_optional(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("tyzen")
            && attr
                .parse_args::<syn::Ident>()
                .map(|ident| ident == "optional")
                .unwrap_or(false)
    })
}

pub fn option_inner_type(ty: &Type) -> Option<&Type> {
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
