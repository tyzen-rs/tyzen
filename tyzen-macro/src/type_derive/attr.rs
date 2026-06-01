use super::case::{RenameRule, rename_rule};
use syn::{Attribute, GenericArgument, PathArguments, Type};

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
    pub binary: bool,
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

            if meta.path.is_ident("with") {
                let value = meta.value()?.parse::<syn::LitStr>()?;
                if value.value() == "serde_bytes" {
                    serde.binary = true;
                }
                return Ok(());
            }

            Ok(())
        });
    }

    serde
}

#[derive(Clone)]
pub enum VariantMetaValue {
    Lit(String),
    List(Vec<syn::Path>),
}

impl std::fmt::Debug for VariantMetaValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lit(s) => f.debug_tuple("Lit").field(s).finish(),
            Self::List(paths) => {
                let path_strs: Vec<String> = paths
                    .iter()
                    .map(|p| {
                        use quote::ToTokens;
                        p.to_token_stream().to_string().replace(" ", "")
                    })
                    .collect();
                f.debug_tuple("List").field(&path_strs).finish()
            }
        }
    }
}

#[derive(Default)]
pub struct TyzenAttrs {
    pub optional: bool,
    pub nullable: bool,
    pub meta_name: Option<String>,
    pub ns: Option<String>,
    pub apply: Option<syn::Path>,
    pub variant_meta: Vec<(String, VariantMetaValue)>,
    pub binary: bool,
    pub schema: bool,
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

            if meta.path.is_ident("nullable") {
                tyzen.nullable = true;
                return Ok(());
            }

            if meta.path.is_ident("binary") {
                tyzen.binary = true;
                return Ok(());
            }

            if meta.path.is_ident("schema") {
                tyzen.schema = true;
                return Ok(());
            }

            if meta.path.is_ident("meta") {
                let value = meta.value()?.parse::<syn::LitStr>()?;
                tyzen.meta_name = Some(value.value());
                return Ok(());
            }

            if meta.path.is_ident("ns") || meta.path.is_ident("namespace") {
                let value = meta.value()?.parse::<syn::LitStr>()?;
                tyzen.ns = Some(value.value());
                return Ok(());
            }

            if meta.path.is_ident("apply") {
                let value = meta.value()?.parse::<syn::Path>()?;
                tyzen.apply = Some(value);
                return Ok(());
            }

            // Catch-all for variant metadata: #[tyzen(code = "404", msg = "Not Found")]
            if let Some(ident) = meta.path.get_ident() {
                let key = ident.to_string();
                if meta.input.peek(syn::token::Paren) {
                    let mut list = Vec::new();
                    let _ = meta.parse_nested_meta(|nested| {
                        list.push(nested.path.clone());
                        Ok(())
                    });
                    tyzen.variant_meta.push((key, VariantMetaValue::List(list)));
                    return Ok(());
                } else {
                    let value = meta.value()?.parse::<syn::LitStr>()?;
                    tyzen
                        .variant_meta
                        .push((key, VariantMetaValue::Lit(value.value())));
                    return Ok(());
                }
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

pub fn parse_field_validation(attrs: &[syn::Attribute]) -> Option<proc_macro2::TokenStream> {
    let mut min_length = None;
    let mut min_length_message = None;
    let mut max_length = None;
    let mut max_length_message = None;
    let mut regex_pattern = None;
    let mut regex_message = None;
    let mut min_value = None;
    let mut min_value_message = None;
    let mut max_value = None;
    let mut max_value_message = None;
    let mut has_email = false;
    let mut email_message = None;
    let mut has_url = false;
    let mut url_message = None;
    let mut contains = None;
    let mut contains_message = None;
    let mut does_not_contain = None;
    let mut does_not_contain_message = None;
    let mut custom_fn = None;
    let mut custom_message = None;
    let mut has_any = false;

    for attr in attrs {
        if !attr.path().is_ident("validate") {
            continue;
        }

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("length") {
                has_any = true;
                let mut len_msg = None;
                let _ = meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("min") {
                        let value = nested.value()?.parse::<syn::LitInt>()?;
                        min_length = Some(value.base10_parse::<usize>()?);
                    } else if nested.path.is_ident("max") {
                        let value = nested.value()?.parse::<syn::LitInt>()?;
                        max_length = Some(value.base10_parse::<usize>()?);
                    } else if nested.path.is_ident("message") {
                        let value = nested.value()?.parse::<syn::LitStr>()?;
                        len_msg = Some(value.value());
                    }
                    Ok(())
                });
                if len_msg.is_some() {
                    min_length_message = len_msg.clone();
                    max_length_message = len_msg;
                }
                return Ok(());
            }

            if meta.path.is_ident("regex") {
                has_any = true;
                if meta.input.peek(syn::token::Paren) {
                    let _ = meta.parse_nested_meta(|nested| {
                        if nested.path.is_ident("path") {
                            let value_expr = nested.value()?.parse::<syn::Expr>()?;
                            let mut pattern = None;
                            if let syn::Expr::Unary(syn::ExprUnary { expr, .. }) = &value_expr {
                                if let syn::Expr::Call(syn::ExprCall { func, .. }) = &**expr {
                                    if let syn::Expr::Path(syn::ExprPath { path, .. }) = &**func {
                                        if let Some(segment) = path.segments.last() {
                                            if segment.ident == "re_code" {
                                                pattern = Some("^[A-Z0-9]{2,5}$".to_string());
                                            }
                                        }
                                    }
                                }
                            }
                            if pattern.is_none() {
                                if let syn::Expr::Lit(syn::ExprLit {
                                    lit: syn::Lit::Str(s),
                                    ..
                                }) = value_expr
                                {
                                    pattern = Some(s.value());
                                }
                            }
                            regex_pattern = pattern;
                        } else if nested.path.is_ident("message") {
                            let value = nested.value()?.parse::<syn::LitStr>()?;
                            regex_message = Some(value.value());
                        }
                        Ok(())
                    });
                } else {
                    let value = meta.value()?.parse::<syn::LitStr>()?;
                    regex_pattern = Some(value.value());
                }
                return Ok(());
            }

            if meta.path.is_ident("range") {
                has_any = true;
                let mut range_msg = None;
                let _ = meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("min") {
                        let val_lit = nested.value()?.parse::<syn::Lit>()?;
                        match val_lit {
                            syn::Lit::Float(f) => {
                                min_value = Some(f.base10_parse::<f64>()?);
                            }
                            syn::Lit::Int(i) => {
                                min_value = Some(i.base10_parse::<i64>()? as f64);
                            }
                            _ => {}
                        }
                    } else if nested.path.is_ident("max") {
                        let val_lit = nested.value()?.parse::<syn::Lit>()?;
                        match val_lit {
                            syn::Lit::Float(f) => {
                                max_value = Some(f.base10_parse::<f64>()?);
                            }
                            syn::Lit::Int(i) => {
                                max_value = Some(i.base10_parse::<i64>()? as f64);
                            }
                            _ => {}
                        }
                    } else if nested.path.is_ident("message") {
                        let value = nested.value()?.parse::<syn::LitStr>()?;
                        range_msg = Some(value.value());
                    }
                    Ok(())
                });
                if range_msg.is_some() {
                    min_value_message = range_msg.clone();
                    max_value_message = range_msg;
                }
                return Ok(());
            }

            if meta.path.is_ident("email") {
                has_any = true;
                has_email = true;
                if meta.input.peek(syn::token::Paren) {
                    let _ = meta.parse_nested_meta(|nested| {
                        if nested.path.is_ident("message") {
                            let value = nested.value()?.parse::<syn::LitStr>()?;
                            email_message = Some(value.value());
                        }
                        Ok(())
                    });
                }
                return Ok(());
            }

            if meta.path.is_ident("url") {
                has_any = true;
                has_url = true;
                if meta.input.peek(syn::token::Paren) {
                    let _ = meta.parse_nested_meta(|nested| {
                        if nested.path.is_ident("message") {
                            let value = nested.value()?.parse::<syn::LitStr>()?;
                            url_message = Some(value.value());
                        }
                        Ok(())
                    });
                }
                return Ok(());
            }

            if meta.path.is_ident("contains") {
                has_any = true;
                if meta.input.peek(syn::token::Paren) {
                    let _ = meta.parse_nested_meta(|nested| {
                        if nested.path.is_ident("pattern") {
                            let value = nested.value()?.parse::<syn::LitStr>()?;
                            contains = Some(value.value());
                        } else if nested.path.is_ident("message") {
                            let value = nested.value()?.parse::<syn::LitStr>()?;
                            contains_message = Some(value.value());
                        }
                        Ok(())
                    });
                } else {
                    let value = meta.value()?.parse::<syn::LitStr>()?;
                    contains = Some(value.value());
                }
                return Ok(());
            }

            if meta.path.is_ident("does_not_contain") {
                has_any = true;
                if meta.input.peek(syn::token::Paren) {
                    let _ = meta.parse_nested_meta(|nested| {
                        if nested.path.is_ident("pattern") {
                            let value = nested.value()?.parse::<syn::LitStr>()?;
                            does_not_contain = Some(value.value());
                        } else if nested.path.is_ident("message") {
                            let value = nested.value()?.parse::<syn::LitStr>()?;
                            does_not_contain_message = Some(value.value());
                        }
                        Ok(())
                    });
                } else {
                    let value = meta.value()?.parse::<syn::LitStr>()?;
                    does_not_contain = Some(value.value());
                }
                return Ok(());
            }

            if meta.path.is_ident("custom") {
                has_any = true;
                if meta.input.peek(syn::token::Paren) {
                    let _ = meta.parse_nested_meta(|nested| {
                        if nested.path.is_ident("function") {
                            let value = nested.value()?.parse::<syn::LitStr>()?;
                            custom_fn = Some(value.value());
                        } else if nested.path.is_ident("message") {
                            let value = nested.value()?.parse::<syn::LitStr>()?;
                            custom_message = Some(value.value());
                        }
                        Ok(())
                    });
                } else {
                    let value = meta.value()?.parse::<syn::LitStr>()?;
                    custom_fn = Some(value.value());
                }
                return Ok(());
            }

            Ok(())
        });
    }

    if has_any {
        let min_len_quote = match min_length {
            Some(l) => quote::quote! { Some(#l) },
            None => quote::quote! { None },
        };
        let min_len_msg_quote = match min_length_message {
            Some(ref m) => quote::quote! { Some(#m) },
            None => quote::quote! { None },
        };
        let max_len_quote = match max_length {
            Some(l) => quote::quote! { Some(#l) },
            None => quote::quote! { None },
        };
        let max_len_msg_quote = match max_length_message {
            Some(ref m) => quote::quote! { Some(#m) },
            None => quote::quote! { None },
        };
        let regex_quote = match regex_pattern {
            Some(ref r) => quote::quote! { Some(#r) },
            None => quote::quote! { None },
        };
        let regex_msg_quote = match regex_message {
            Some(ref m) => quote::quote! { Some(#m) },
            None => quote::quote! { None },
        };
        let min_val_quote = match min_value {
            Some(v) => quote::quote! { Some(#v) },
            None => quote::quote! { None },
        };
        let min_val_msg_quote = match min_value_message {
            Some(ref m) => quote::quote! { Some(#m) },
            None => quote::quote! { None },
        };
        let max_val_quote = match max_value {
            Some(v) => quote::quote! { Some(#v) },
            None => quote::quote! { None },
        };
        let max_val_msg_quote = match max_value_message {
            Some(ref m) => quote::quote! { Some(#m) },
            None => quote::quote! { None },
        };
        let email_msg_quote = match email_message {
            Some(ref m) => quote::quote! { Some(#m) },
            None => quote::quote! { None },
        };
        let url_msg_quote = match url_message {
            Some(ref m) => quote::quote! { Some(#m) },
            None => quote::quote! { None },
        };
        let contains_quote = match contains {
            Some(ref c) => quote::quote! { Some(#c) },
            None => quote::quote! { None },
        };
        let contains_msg_quote = match contains_message {
            Some(ref m) => quote::quote! { Some(#m) },
            None => quote::quote! { None },
        };
        let does_not_contain_quote = match does_not_contain {
            Some(ref c) => quote::quote! { Some(#c) },
            None => quote::quote! { None },
        };
        let does_not_contain_msg_quote = match does_not_contain_message {
            Some(ref m) => quote::quote! { Some(#m) },
            None => quote::quote! { None },
        };
        let custom_fn_quote = match custom_fn {
            Some(ref f) => quote::quote! { Some(#f) },
            None => quote::quote! { None },
        };
        let custom_msg_quote = match custom_message {
            Some(ref m) => quote::quote! { Some(#m) },
            None => quote::quote! { None },
        };

        Some(quote::quote! {
            Some(::tyzen::meta::ValidationRule {
                min_length: #min_len_quote,
                min_length_message: #min_len_msg_quote,
                max_length: #max_len_quote,
                max_length_message: #max_len_msg_quote,
                regex_pattern: #regex_quote,
                regex_message: #regex_msg_quote,
                min_value: #min_val_quote,
                min_value_message: #min_val_msg_quote,
                max_value: #max_val_quote,
                max_value_message: #max_val_msg_quote,
                email: #has_email,
                email_message: #email_msg_quote,
                url: #has_url,
                url_message: #url_msg_quote,
                contains: #contains_quote,
                contains_message: #contains_msg_quote,
                does_not_contain: #does_not_contain_quote,
                does_not_contain_message: #does_not_contain_msg_quote,
                custom_fn: #custom_fn_quote,
                custom_message: #custom_msg_quote,
            })
        })
    } else {
        None
    }
}
