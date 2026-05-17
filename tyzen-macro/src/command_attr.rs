use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ItemFn, PatType, ReturnType, parse_macro_input};
use crate::utils::is_known_binary_type;

/// Entry point for the `#[tyzen::command]` attribute.
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_parsed = parse_attr(attr);
    expand_command(item, false, attr_parsed)
}

/// Entry point for the `#[tyzen::tauri_command]` attribute.
pub fn tauri_command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_parsed = parse_attr(attr);
    expand_command(item, true, attr_parsed)
}

/// Main expansion logic for commands.
/// 
/// This function:
/// 1. Parses the function signature.
/// 2. Collects parameters, skipping Tauri framework types (State, AppHandle, etc.).
/// 3. Registers command metadata with the `tyzen` inventory.
/// 4. Optionally (if `emit_tauri` is true) generates a Tauri handler wrapper and submits it to `tyzen-tauri`.
fn expand_command(item: TokenStream, emit_tauri: bool, attr: (Option<String>, Option<String>, bool)) -> TokenStream {
    let (ns, rename, attr_binary) = attr;
    let func = parse_macro_input!(item as ItemFn);
    let ns_val = match ns {
        Some(s) => quote! { Some(#s) },
        None => quote! { None },
    };
    let rename_val = match rename {
        Some(s) => quote! { Some(#s) },
        None => quote! { None },
    };
    let sig = &func.sig;
    let fn_name = &sig.ident;
    let fn_name_str = fn_name.to_string();

    let params: Vec<_> = sig.inputs.iter().filter_map(command_param).collect();
    let params_ts: Vec<_> = params
        .iter()
        .map(|(pat, ty)| {
            let name = quote!(#pat).to_string();
            if let Some(inner) = channel_inner_type(ty) {
                quote! { ::tyzen::ParamMeta { name: #name, ty: <#inner as ::tyzen::TsType>::ts_name, is_channel: true } }
            } else {
                quote! { ::tyzen::ParamMeta { name: #name, ty: <#ty as ::tyzen::TsType>::ts_name, is_channel: false } }
            }
        })
        .collect();
    let is_binary = match &sig.output {
        ReturnType::Default => false,
        ReturnType::Type(_, ty) => is_known_binary_type(ty) || attr_binary,
    };

    let return_type_fn = return_type_fn(&sig.output);
    let handler_fn_name = quote::format_ident!("__tyzen_handler_{}", fn_name);
    let tauri_part = tauri_handler_submission(emit_tauri, &fn_name_str, &handler_fn_name, fn_name);

    let maybe_tauri_attr = if emit_tauri {
        quote! { #[tauri::command] }
    } else {
        quote! {}
    };

    quote! {
        #maybe_tauri_attr
        #func

        ::tyzen::__private::inventory::submit! {
            ::tyzen::CommandMeta {
                name: #fn_name_str,
                params: &[#(#params_ts),*],
                return_type: #return_type_fn,
                ns: #ns_val,
                rename: #rename_val,
                module_path: module_path!(),
                is_binary: #is_binary,
            }
        }

        #tauri_part
    }
    .into()
}

/// Filters function arguments to find those that should be exposed to TypeScript.
/// 
/// Skips `self` receivers and parameters that are internal to the Tauri framework.
fn command_param(arg: &FnArg) -> Option<(&syn::Pat, &syn::Type)> {
    match arg {
        FnArg::Typed(PatType { pat, ty, .. }) if is_framework_param(ty) => None,
        FnArg::Typed(PatType { pat, ty, .. }) => Some((pat, ty)),
        FnArg::Receiver(_) => None,
    }
}

/// Determines if a type is a "framework parameter" that should be hidden from TypeScript.
/// 
/// Types like `State<T>`, `Window`, or `AppHandle` are injected by Tauri and should not
/// appear in the generated frontend API.
fn is_framework_param(ty: &syn::Type) -> bool {
    let syn::Type::Path(type_path) = ty else {
        return false;
    };
    let Some(last) = type_path.path.segments.last() else {
        return false;
    };
    matches!(
        last.ident.to_string().as_str(),
        "State" | "AppHandle" | "Window" | "Webview" | "EventLoopProxy" | "Runtime" | "Scope"
    )
}

/// Generates a TokenStream for the return type of a command.
fn return_type_fn(output: &ReturnType) -> proc_macro2::TokenStream {
    match output {
        ReturnType::Default => quote! { <() as ::tyzen::TsType>::ts_name },
        ReturnType::Type(_, ty) => quote! { <#ty as ::tyzen::TsType>::ts_name },
    }
}

/// Generates the code to submit a Tauri command handler to the inventory.
fn tauri_handler_submission(
    emit_tauri: bool,
    fn_name_str: &str,
    handler_fn_name: &syn::Ident,
    fn_name: &syn::Ident,
) -> proc_macro2::TokenStream {
    if !emit_tauri {
        return quote! {};
    }

    quote! {
        #[allow(non_snake_case)]
        fn #handler_fn_name(invoke: ::tauri::ipc::Invoke<::tauri::Wry>) -> bool {
            let handlers: fn(::tauri::ipc::Invoke<::tauri::Wry>) -> bool =
                ::tauri::generate_handler![#fn_name];
            handlers(invoke)
        }

        ::tyzen_tauri::__private::inventory::submit! {
            ::tyzen_tauri::HandlerMeta {
                name: #fn_name_str,
                handler: #handler_fn_name,
            }
        }
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

/// Parses the attributes of the `command` macro (e.g. `ns`, `rename`).
fn parse_attr(attr: TokenStream) -> (Option<String>, Option<String>, bool) {
    if attr.is_empty() {
        return (None, None, false);
    }

    let mut ns = None;
    let mut rename = None;
    let mut binary = false;
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("ns") || meta.path.is_ident("namespace") {
            let value = meta.value()?.parse::<syn::LitStr>()?;
            ns = Some(value.value());
            return Ok(());
        }
        if meta.path.is_ident("rename") {
            let value = meta.value()?.parse::<syn::LitStr>()?;
            rename = Some(value.value());
            return Ok(());
        }
        if meta.path.is_ident("binary") {
            binary = true;
            return Ok(());
        }
        Ok(())
    });

    use syn::parse::Parser;
    let _ = parser.parse(attr);

    (ns, rename, binary)
}
