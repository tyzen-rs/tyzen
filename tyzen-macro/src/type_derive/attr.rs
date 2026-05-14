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

#[derive(Default)]
pub struct TyzenAttrs {
    pub optional: bool,
    pub meta_name: Option<String>,
    pub variant_meta: Vec<(String, String)>,
}

pub fn tyzen_attrs(attrs: &[Attribute]) -> TyzenAttrs {
    let mut tyzen = TyzenAttrs::default();

    for attr in attrs {
        if !attr.path().is_ident("tyzen") {
            continue;
        }

        // Handle both #[tyzen(optional)] and #[tyzen(key = "val")]
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("optional") {
                tyzen.optional = true;
                return Ok(());
            }

            if meta.path.is_ident("meta") {
                let value = meta.value()?.parse::<syn::LitStr>()?;
                tyzen.meta_name = Some(value.value());
                return Ok(());
            }

            // Catch-all for variant metadata: #[tyzen(code = "404", msg = "Not Found")]
            if let Some(ident) = meta.path.get_ident() {
                let key = ident.to_string();
                let value = meta.value()?.parse::<syn::LitStr>()?;
                tyzen.variant_meta.push((key, value.value()));
                return Ok(());
            }

            Ok(())
        });
    }

    tyzen
}

pub fn has_tyzen_optional(attrs: &[Attribute]) -> bool {
    tyzen_attrs(attrs).optional
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
