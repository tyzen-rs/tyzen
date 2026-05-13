use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ItemFn, PatType, ReturnType, parse_macro_input};

pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return quote! {
            compile_error!("unsupported tyzen::command argument; use #[tyzen::command] for framework-agnostic TS metadata or #[tyzen_tauri::command] for Tauri handlers");
        }
        .into();
    }

    expand_command(item, false)
}

pub fn tauri_command(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return quote! {
            compile_error!("unsupported tyzen_tauri::command argument; use #[tyzen_tauri::command]");
        }
        .into();
    }

    expand_command(item, true)
}

fn expand_command(item: TokenStream, emit_tauri: bool) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
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
    let return_type_fn = return_type_fn(&sig.output);
    let handler_fn_name = quote::format_ident!("__tyzen_handler_{}", fn_name);
    let tauri_part = tauri_handler_submission(emit_tauri, &fn_name_str, &handler_fn_name, fn_name);

    quote! {
        #func

        ::tyzen::inventory::submit! {
            ::tyzen::CommandMeta {
                name: #fn_name_str,
                params: &[#(#params_ts),*],
                return_type: #return_type_fn,
            }
        }

        #tauri_part
    }
    .into()
}

fn command_param(arg: &FnArg) -> Option<(&syn::Pat, &syn::Type)> {
    match arg {
        FnArg::Typed(PatType { pat, ty, .. }) if is_framework_param(ty) => None,
        FnArg::Typed(PatType { pat, ty, .. }) => Some((pat, ty)),
        FnArg::Receiver(_) => None,
    }
}

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

fn return_type_fn(output: &ReturnType) -> proc_macro2::TokenStream {
    match output {
        ReturnType::Default => quote! { <() as ::tyzen::TsType>::ts_name },
        ReturnType::Type(_, ty) => quote! { <#ty as ::tyzen::TsType>::ts_name },
    }
}

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

        ::tyzen_tauri::inventory::submit! {
            ::tyzen_tauri::HandlerMeta {
                name: #fn_name_str,
                handler: #handler_fn_name,
            }
        }
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
