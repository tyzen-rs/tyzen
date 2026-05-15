//! Tauri integration for Tyzen.
//! 
//! This crate provides the glue between Tyzen's metadata system and Tauri's 
//! IPC mechanism. It handles the registration of command handlers and the
//! generation of TypeScript wrappers that use Tauri's `invoke` and `event` APIs.

/// Private re-exports for macro-generated code. Not part of the public API.
#[doc(hidden)]
pub mod __private {
    pub use tyzen::__private::inventory;
}

pub use tyzen_macro::tauri_command as command;

use tyzen::{__private::inventory, utils::snake_to_camel};

/// Metadata for a Tauri IPC handler registered via `#[tyzen_tauri::command]`.
/// 
/// This is used by the `handler!` macro to route incoming IPC calls to the
/// correct Rust function.
pub struct HandlerMeta {
    /// The original Rust function name (snake_case).
    pub name: &'static str,
    /// The generated wrapper function that calls `tauri::generate_handler!`.
    pub handler: fn(tauri::ipc::Invoke<tauri::Wry>) -> bool,
}

inventory::collect!(HandlerMeta);

/// Generates TypeScript bindings with default configuration.
/// 
/// This will generate type definitions, constants, and namespaced command wrappers.
/// The output includes Tauri-specific helpers like `invoke` and `listen`.
pub fn generate(output_path: &str) -> std::io::Result<()> {
    generate_with_config(output_path, tyzen::GeneratorConfig::default())
}

/// Generates TypeScript bindings with a custom configuration.
pub fn generate_with_config(output_path: &str, config: tyzen::GeneratorConfig) -> std::io::Result<()> {
    tyzen::generate_full(output_path, config, write_tauri_commands, write_tauri_events)
}

/// Internal helper to write the Tauri-specific command wrapper section.
/// 
/// This emits:
/// 1. Imports from `@tauri-apps/api`.
/// 2. `__invoke` and `__listen` internal helpers.
/// 3. The `commands` object for the root (default) namespace.
pub fn write_tauri_commands(ts: &mut String) {
    let map = tyzen::NamespaceMap::collect();
    
    ts.push_str("/** autogen helpers **/\n");
    ts.push_str("import { invoke } from \"@tauri-apps/api/core\"\n");
    ts.push_str("import { listen } from \"@tauri-apps/api/event\"\n\n");
    
    ts.push_str(concat!(
        "type InvokeResult<T> = T extends { status: \"ok\" } | { status: \"error\" } ? T : Result<T>;\n\n",
        "async function __invoke<T>(name: string, args: Record<string, any>): Promise<InvokeResult<T>> {\n",
        "  try {\n",
        "    const data = await invoke(name, args);\n",
        "    return { status: \"ok\", data } as InvokeResult<T>;\n",
        "  } catch (e) {\n",
        "    if (e instanceof Error) throw e;\n",
        "    return { status: \"error\", error: e as string } as InvokeResult<T>;\n",
        "  }\n",
        "}\n\n",
        "function __listen(name: string, cb: (payload: any) => void) {\n",
        "  return listen(name, (e) => cb(e.payload));\n",
        "}\n\n"
    ));

    if let Some(root_commands) = map.commands.get(&None) {
        ts.push_str("export const commands = {\n");
        for cmd in root_commands {
            let fn_name = cmd.rename.map(|r| r.to_string()).unwrap_or_else(|| snake_to_camel(cmd.name));
            let params_ts: Vec<String> = cmd.params.iter().map(|p| format!("{}: {}", p.name, (p.ty)())).collect();
            ts.push_str(&format!("  {}: ({}) => __invoke<{}>(\"{}\", {{ {} }}),\n", 
                fn_name, 
                params_ts.join(", "), 
                (cmd.return_type)(), 
                cmd.name,
                cmd.params.iter().map(|p| p.name).collect::<Vec<_>>().join(", ")
            ));
        }
        ts.push_str("}\n");
    }
}

/// Internal helper to write the Tauri-specific event listener section.
pub fn write_tauri_events(ts: &mut String) {
    let map = tyzen::NamespaceMap::collect();
    if let Some(root_events) = map.events.get(&None) {
        ts.push_str("\n/** Global Events **/\n");
        ts.push_str("export const events = {\n");
        for ev in root_events {
             let camel_name = snake_to_camel(ev.name.replace("-", "_").replace(":", "_").as_str());
             let ev_fn_name = format!("on{}{}", (&camel_name[0..1]).to_uppercase(), &camel_name[1..]);
             ts.push_str(&format!("  {}: (cb: (payload: {}) => void) => __listen(\"{}\", cb),\n", 
                ev_fn_name, (ev.payload_type)(), ev.name));
        }
        ts.push_str("}\n");
    }
}

/// Generates a Tauri IPC handler that routes calls to registered Tyzen commands.
/// 
/// This macro should be used within the `tauri::Builder::invoke_handler` call.
/// It automatically detects all functions marked with `#[tyzen_tauri::command]`.
/// 
/// In debug builds, it also performs a safety check to ensure every command 
/// has a corresponding handler registered.
/// 
/// # Example
/// ```rust
/// tauri::Builder::default()
///     .invoke_handler(tyzen_tauri::handler!())
///     .run(tauri::generate_context!())
///     .expect("error while running tauri application");
/// ```
#[macro_export]
macro_rules! handler {
    () => {{
        // In debug builds, warn about commands registered with #[tyzen::command]
        // instead of #[tyzen_tauri::command] — they have TS metadata but no handler.
        #[cfg(debug_assertions)]
        {
            let handler_names: ::std::collections::HashSet<&str> =
                ::tyzen::__private::inventory::iter::<::tyzen_tauri::HandlerMeta>()
                    .map(|h| h.name)
                    .collect();
            for cmd in ::tyzen::__private::inventory::iter::<::tyzen::CommandMeta>() {
                if !handler_names.contains(cmd.name) {
                    eprintln!(
                        "[tyzen] warning: command `{}` has TypeScript metadata but no Tauri \
                         handler. Did you use #[tyzen::command] instead of \
                         #[tyzen_tauri::command]?",
                        cmd.name
                    );
                }
            }
        }

        |invoke: ::tauri::ipc::Invoke<::tauri::Wry>| {
            let cmd = invoke.message.command().to_string();
            for meta in ::tyzen::__private::inventory::iter::<::tyzen_tauri::HandlerMeta>() {
                if meta.name == cmd {
                    return (meta.handler)(invoke);
                }
            }
            false
        }
    }};
}
