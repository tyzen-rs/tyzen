pub use tyzen::inventory;
pub use tyzen_macro::tauri_command as command;

use tyzen::{CommandMeta, EventMeta, utils::snake_to_camel};

pub struct HandlerMeta {
    pub name: &'static str,
    pub handler: fn(tauri::ipc::Invoke<tauri::Wry>) -> bool,
}

inventory::collect!(HandlerMeta);

pub fn generate(output_path: &str) {
    tyzen::generate_full(output_path, write_tauri_commands, write_tauri_events);
}

pub fn write_tauri_commands(ts: &mut String) {
    ts.push_str("import { invoke, Channel } from \"@tauri-apps/api/core\"\n");
    ts.push_str("import { listen, once, emit, type EventCallback } from \"@tauri-apps/api/event\"\n\n");
    ts.push_str("export const commands = {\n");
    for cmd in inventory::iter::<CommandMeta> {
        let fn_name = snake_to_camel(cmd.name);
        let params_ts: Vec<String> = cmd
            .params
            .iter()
            .map(|(name, ty_fn)| format!("{}: {}", name, ty_fn()))
            .collect();
        let param_names: Vec<&str> = cmd.params.iter().map(|(name, _)| *name).collect();

        let raw_return_type = (cmd.return_type)();
        let ts_return_type = if raw_return_type.starts_with("Result<") {
            raw_return_type
        } else {
            format!("Result<{}, any>", raw_return_type)
        };

        ts.push_str(&format!(
            "  async {}({}): Promise<{}> {{try {{return {{status: \"ok\",\n    data: await invoke(\"{}\", {{ {} }}),}};}}\ncatch (e) {{if (e instanceof Error) throw e;return {{ status: \"error\", error: e as any }};}}}},\n",
            fn_name,
            params_ts.join(", "),
            ts_return_type,
            cmd.name,
            param_names.join(", "),
        ));
    }
    ts.push_str("}\n");
}

pub fn write_tauri_events(ts: &mut String) {
    ts.push_str("\n/** autogen events **/\n");
    ts.push_str("export type Events = {\n");
    for event in inventory::iter::<EventMeta> {
        let js_name = snake_to_camel(event.name.replace(":", "_").replace("-", "_").as_str());
        ts.push_str(&format!("  {}: {},\n", js_name, (event.payload_type)()));
    }
    ts.push_str("}\n\n");

    ts.push_str("const __event_mappings__: Record<string, string> = {\n");
    for event in inventory::iter::<EventMeta> {
        let js_name = snake_to_camel(event.name.replace(":", "_").replace("-", "_").as_str());
        ts.push_str(&format!("  {}: \"{}\",\n", js_name, event.name));
    }
    ts.push_str("}\n\n");

    ts.push_str("export const events = __makeEvents__<Events>(__event_mappings__)\n\n");

    ts.push_str("function __makeEvents__<T extends Record<string, any>>(mappings: Record<string, string>) {\n");
    ts.push_str("  return new Proxy({} as any, {\n");
    ts.push_str("    get: (_, prop: string) => {\n");
    ts.push_str("      const name = mappings[prop];\n");
    ts.push_str("      return {\n");
    ts.push_str("        listen: (cb: (payload: any) => void) => listen(name, (e) => cb(e.payload)),\n");
    ts.push_str("        once: (cb: (payload: any) => void) => once(name, (e) => cb(e.payload)),\n");
    ts.push_str("        emit: (payload: any) => emit(name, payload),\n");
    ts.push_str("      }\n");
    ts.push_str("    }\n");
    ts.push_str("  }) as { [K in keyof T]: { \n");
    ts.push_str("    listen: (cb: (payload: T[K]) => void) => Promise<any>,\n");
    ts.push_str("    once: (cb: (payload: T[K]) => void) => Promise<any>,\n");
    ts.push_str("    emit: (payload: T[K]) => Promise<void>\n");
    ts.push_str("  } };\n");
    ts.push_str("}\n");
}

#[macro_export]
macro_rules! handler {
    () => {
        |invoke: ::tauri::ipc::Invoke<::tauri::Wry>| {
            let cmd = invoke.message.command().to_string();
            for meta in ::tyzen::inventory::iter::<::tyzen_tauri::HandlerMeta> {
                if meta.name == cmd {
                    return (meta.handler)(invoke);
                }
            }
            false
        }
    };
}
