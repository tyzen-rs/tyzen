/// Private re-exports for macro-generated code. Not part of the public API.
#[doc(hidden)]
pub mod __private {
    pub use tyzen::__private::inventory;
}

pub use tyzen_macro::tauri_command as command;

use tyzen::{CommandMeta, EventMeta, __private::inventory, utils::snake_to_camel};


/// Metadata for a Tauri IPC handler registered via `#[tyzen_tauri::command]`.
pub struct HandlerMeta {
    pub name: &'static str,
    pub handler: fn(tauri::ipc::Invoke<tauri::Wry>) -> bool,
}

inventory::collect!(HandlerMeta);

pub fn generate(output_path: &str) -> std::io::Result<()> {
    tyzen::generate_full(output_path, write_tauri_commands, write_tauri_events)
}

pub fn write_tauri_commands(ts: &mut String) {
    let mut commands: Vec<_> = inventory::iter::<CommandMeta>().collect();
    if commands.is_empty() {
        return;
    }
    commands.sort_by_key(|c| c.name);

    ts.push_str("/** autogen commands **/\n");
    ts.push_str("import { invoke, Channel } from \"@tauri-apps/api/core\"\n");
    ts.push_str(
        "import { listen, once, emit, type EventCallback } from \"@tauri-apps/api/event\"\n\n",
    );
    ts.push_str("export const commands = {\n");
    for cmd in &commands {
        let fn_name = snake_to_camel(cmd.name);
        let params_ts: Vec<String> = cmd
            .params
            .iter()
            .map(|p| {
                let ty = (p.ty)();
                let param_name = snake_to_camel(p.name);
                if p.is_channel {
                    format!("{}: (payload: {}) => void", param_name, ty)
                } else {
                    format!("{}: {}", param_name, ty)
                }
            })
            .collect();

        let invoke_args: Vec<String> = cmd
            .params
            .iter()
            .map(|p| {
                let param_name = snake_to_camel(p.name);
                if p.is_channel {
                    format!("{}: new Channel({})", param_name, param_name)
                } else {
                    param_name
                }
            })
            .collect();

        let raw_return_type = (cmd.return_type)();
        let ts_return_type = if raw_return_type.starts_with("Result<") {
            raw_return_type
        } else {
            format!("Result<{}>", raw_return_type)
        };

        ts.push_str(&format!(
            "  async {}({}): Promise<{}> {{\n",
            fn_name,
            params_ts.join(", "),
            ts_return_type
        ));
        ts.push_str("    try {\n");
        ts.push_str(&format!(
            "      return {{ status: \"ok\", data: await invoke(\"{}\", {{ {} }}) }};\n",
            cmd.name,
            invoke_args.join(", ")
        ));
        ts.push_str("    } catch (e) {\n");
        ts.push_str("      if (e instanceof Error) throw e;\n");
        ts.push_str("      return { status: \"error\", error: e as any };\n");
        ts.push_str("    }\n");
        ts.push_str("  },\n");
    }
    ts.push_str("}\n");
}

pub fn write_tauri_events(ts: &mut String) {
    let events: Vec<_> = inventory::iter::<EventMeta>().collect();
    if events.is_empty() {
        return;
    }

    ts.push_str("\n/** autogen events **/\n");
    ts.push_str("export type Events = {\n");
    for event in &events {
        let js_name = snake_to_camel(event.name.replace(':', "_").replace('-', "_").as_str());
        ts.push_str(&format!("  {}: {},\n", js_name, (event.payload_type)()));
    }
    ts.push_str("}\n\n");

    ts.push_str("const __event_mappings__: Record<string, string> = {\n");
    for event in &events {
        let js_name = snake_to_camel(event.name.replace(':', "_").replace('-', "_").as_str());
        ts.push_str(&format!("  {}: \"{}\",\n", js_name, event.name));
    }
    ts.push_str("}\n\n");

    ts.push_str("export const events = __makeEvents__<Events>(__event_mappings__)\n\n");
    ts.push_str(concat!(
        "function __makeEvents__<T extends Record<string, any>>(mappings: Record<string, string>) {\n",
        "  return new Proxy({} as any, {\n",
        "    get: (_, prop: string) => {\n",
        "      const name = mappings[prop];\n",
        "      return {\n",
        "        listen: (cb: (payload: any) => void) => listen(name, (e) => cb(e.payload)),\n",
        "        once: (cb: (payload: any) => void) => once(name, (e) => cb(e.payload)),\n",
        "        emit: (payload: any) => emit(name, payload),\n",
        "      }\n",
        "    }\n",
        "  }) as { [K in keyof T]: {\n",
        "    listen: (cb: (payload: T[K]) => void) => Promise<any>,\n",
        "    once: (cb: (payload: T[K]) => void) => Promise<any>,\n",
        "    emit: (payload: T[K]) => Promise<void>\n",
        "  } };\n",
        "}\n"
    ));
}

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
