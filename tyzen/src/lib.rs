/// Private re-exports for macro-generated code. Not part of the public API.
#[doc(hidden)]
pub mod __private {
    pub use inventory;
}

pub use tyzen_macro::{Event, Type, command, event, export};
pub mod meta;
pub mod utils;

use crate::utils::snake_to_camel;

pub mod ts_type;

type TypeFactory = fn() -> String;

/// Metadata describing a single parameter of a registered command.
///
/// Generated automatically by `#[tyzen::command]`. Not intended for manual use.
pub struct ParamMeta {
    /// The parameter name as it appears in the Rust function signature.
    pub name: &'static str,
    /// Returns the TypeScript type name for this parameter.
    pub ty: TypeFactory,
    /// Whether this parameter is a `tauri::ipc::Channel<T>`.
    ///
    /// When `true`, the TypeScript wrapper accepts a callback `(payload: T) => void`
    /// instead of a value.
    pub is_channel: bool,
}

/// Metadata describing a command registered for TypeScript generation.
///
/// Generated automatically by `#[tyzen::command]` or `#[tyzen_tauri::command]`.
/// Not intended for manual use.
pub struct CommandMeta {
    /// The Rust function name (snake_case). Converted to camelCase in generated TypeScript.
    pub name: &'static str,
    /// The command's parameters.
    pub params: &'static [ParamMeta],
    /// Returns the TypeScript return type for this command.
    pub return_type: TypeFactory,
}

/// Metadata describing a Rust type registered for TypeScript generation.
///
/// Generated automatically by `#[derive(tyzen::Type)]`. Not intended for manual use.
pub struct TypeMeta {
    /// The Rust type name, used as the TypeScript type name.
    pub name: &'static str,
    /// Generic parameter string (e.g. `"<T>"`) or `""` for non-generic types.
    pub generic_params: &'static str,
    /// Returns the structural description of this type used for code generation.
    pub structure: fn() -> meta::TypeStructure,
}

/// Metadata describing a typed event registered for TypeScript generation.
///
/// Generated automatically by `#[derive(tyzen::Event)]`. Not intended for manual use.
pub struct EventMeta {
    /// The event name emitted over the wire (e.g. `"user-joined"`).
    pub name: &'static str,
    /// Returns the TypeScript type name for the event payload.
    pub payload_type: fn() -> String,
}

/// Metadata describing a constant exported to TypeScript.
///
/// Generated automatically by `#[tyzen::export]` on a `pub const` item.
/// Not intended for manual use.
pub struct ConstMeta {
    /// The constant name as it appears in the generated TypeScript.
    pub name: &'static str,
    /// The constant value as a string literal for direct embedding in TypeScript.
    pub value: &'static str,
}

/// Trait for types that have a TypeScript type name representation.
///
/// Implemented automatically by `#[derive(tyzen::Type)]`. Built-in implementations
/// are provided for Rust primitives, `String`, `Vec<T>`, `Option<T>`,
/// `Result<T, E>`, and common standard library types.
///
/// Implement this manually only for external types not covered by Tyzen,
/// or wrap them in a newtype with `#[derive(tyzen::Type)]`.
///
/// # Examples
///
/// ```rust
/// use tyzen::TsType;
///
/// assert_eq!(u32::ts_name(), "number");
/// assert_eq!(String::ts_name(), "string");
/// assert_eq!(bool::ts_name(), "boolean");
/// assert_eq!(<Vec<u32>>::ts_name(), "number[]");
/// assert_eq!(<Option<String>>::ts_name(), "string | null");
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be used as a TypeScript type",
    label = "this type has no TypeScript representation",
    note = "add `#[derive(tyzen::Type)]` to your type, or implement `tyzen::TsType` manually",
    note = "for types from external crates, use a newtype: `struct MyWrapper(ExternalType);`"
)]
pub trait TsType {
    /// Returns the TypeScript type name for `Self`.
    fn ts_name() -> String;
}

inventory::collect!(CommandMeta);
inventory::collect!(TypeMeta);
inventory::collect!(EventMeta);
inventory::collect!(ConstMeta);

/// Generates TypeScript bindings for all registered types, commands, and constants.
///
/// This is the primary entry point for non-Tauri projects. For Tauri projects,
/// use `tyzen_tauri::generate` instead, which also emits typed command wrappers
/// and event listeners.
///
/// Creates parent directories as needed. The file is only written when content
/// has changed, avoiding unnecessary downstream rebuilds.
///
/// # Errors
///
/// Returns an error if the output directory cannot be created or the file
/// cannot be written.
///
/// # Examples
///
/// ```rust,no_run
/// fn main() {
///     tyzen::generate("../src/bindings.ts")
///         .expect("failed to generate TypeScript bindings");
/// }
/// ```
pub fn generate(output_path: &str) -> std::io::Result<()> {
    generate_full(output_path, write_command_declarations, |_| {})
}

/// Generates TypeScript bindings with a custom command writer.
///
/// Like [`generate`], but allows you to control how command declarations
/// are emitted. Useful when building framework-specific integrations on
/// top of Tyzen.
///
/// # Errors
///
/// Returns an error if the output file cannot be created or written.
pub fn generate_with_commands(
    output_path: &str,
    write_commands: fn(&mut String),
) -> std::io::Result<()> {
    generate_full(output_path, write_commands, |_| {})
}

/// Generates TypeScript bindings with full control over the output structure.
///
/// `write_before_types` runs before type definitions (use it for imports and
/// command declarations). `write_after_types` runs last (use it for event
/// helpers or custom utility code).
///
/// The generated file structure is:
/// 1. Header comment
/// 2. `write_before_types` output
/// 3. Exported constants (sorted alphabetically by name)
/// 4. Type definitions (sorted alphabetically by name)
/// 5. Built-in `Result<T, E>` type alias
/// 6. `write_after_types` output
///
/// # Errors
///
/// Returns an error if the output file cannot be created or written.
///
/// # Examples
///
/// ```rust,no_run
/// fn main() {
///     tyzen::generate_full(
///         "../src/bindings.ts",
///         tyzen::write_command_declarations,
///         |ts| ts.push_str("// custom footer\n"),
///     ).expect("failed to generate bindings");
/// }
/// ```
pub fn generate_full(
    output_path: &str,
    write_before_types: impl FnOnce(&mut String),
    write_after_types: impl FnOnce(&mut String),
) -> std::io::Result<()> {
    use std::fs;
    use std::path::PathBuf;

    let path = PathBuf::from(output_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut ts = String::new();
    ts.push_str("// auto-generated by tyzen, do not edit\n\n");
    write_before_types(&mut ts);

    let mut constants: Vec<_> = inventory::iter::<ConstMeta>().collect();
    constants.sort_by_key(|c| c.name);
    if !constants.is_empty() {
        ts.push_str("\n/** autogen constants **/\n");
        for c in constants {
            ts.push_str(&format!(
                "export const {} = {} as const;\n",
                c.name, c.value
            ));
        }
    }

    ts.push_str("\n/** autogen types **/\n");
    let mut metas: Vec<&TypeMeta> = inventory::iter::<TypeMeta>().collect();
    metas.sort_by_key(|m| m.name);
    for t in &metas {
        ts.push_str(&render_type(t, &metas));
        ts.push('\n');
    }

    ts.push('\n');
    ts.push_str("/** Generic Result type used for command returns **/\n");
    ts.push_str("export type Result<T, E = string> = { status: \"ok\"; data: T } | { status: \"error\"; error: E }\n");

    write_after_types(&mut ts);

    let current_content = fs::read_to_string(&path).ok();
    if current_content.as_ref() != Some(&ts) {
        fs::write(&path, ts)?;
        println!(
            "\x1b[32m\x1b[1m{:>12}\x1b[0m bindings updated -> \x1b[90m{}\x1b[0m",
            "Tyzen", output_path
        );
    } else {
        println!(
            "\x1b[33m\x1b[1m{:>12}\x1b[0m bindings are up to date",
            "Tyzen"
        );
    }
    Ok(())
}

fn render_type(meta: &TypeMeta, all_metas: &[&TypeMeta]) -> String {
    let structure = (meta.structure)();
    match structure {
        meta::TypeStructure::Struct(s) => {
            let mut fields = Vec::new();
            collect_fields_from_list(s.fields, all_metas, &mut fields);
            let fields_str = fields
                .iter()
                .map(|f| {
                    let label = if f.optional {
                        format!("{}?", f.name)
                    } else {
                        f.name.to_string()
                    };
                    format!("{}: {}", label, (f.ty_name)())
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("export type {}{} = {{ {} }}", meta.name, meta.generic_params, fields_str)
        }
        meta::TypeStructure::Tuple(types) => {
            if types.len() == 1 {
                format!("export type {}{} = {}", meta.name, meta.generic_params, (types[0])())
            } else {
                let inner = types.iter().map(|t| t()).collect::<Vec<_>>().join(", ");
                format!("export type {}{} = [{}]", meta.name, meta.generic_params, inner)
            }
        }
        meta::TypeStructure::Enum(e) => {
            if !e.untagged
                && e.variants
                    .iter()
                    .all(|v| matches!(v.fields, meta::VariantFields::Unit))
            {
                let variant_names: Vec<String> =
                    e.variants.iter().map(|v| v.name.to_string()).collect();
                let fields = variant_names
                    .iter()
                    .map(|n| format!("  {}: \"{}\"", n, n))
                    .collect::<Vec<_>>()
                    .join(",\n");
                format!(
                    "export const {} = {{\n{}\n}} as const;\nexport type {}{} = (typeof {})[keyof typeof {}]",
                    meta.name, fields, meta.name, meta.generic_params, meta.name, meta.name
                )
            } else {
                let variants: Vec<String> = e
                    .variants
                    .iter()
                    .map(|v| render_variant(v, &e, all_metas))
                    .collect();
                format!("export type {}{} = {}", meta.name, meta.generic_params, variants.join(" | "))
            }
        }
        meta::TypeStructure::Transparent(inner) => {
            format!("export type {}{} = {}", meta.name, meta.generic_params, inner)
        }
        meta::TypeStructure::Unit => {
            format!("export type {} = null", meta.name)
        }
    }
}

fn collect_fields_from_list(
    fields: &'static [meta::FieldMeta],
    all_metas: &[&TypeMeta],
    out: &mut Vec<&'static meta::FieldMeta>,
) {
    for field in fields {
        if field.flattened {
            if let Some(base_name) = field.flatten_base_name {
                match all_metas.iter().find(|m| m.name == base_name) {
                    Some(target) => {
                        if let meta::TypeStructure::Struct(inner_s) = (target.structure)() {
                            collect_fields_from_list(inner_s.fields, all_metas, out);
                        }
                    }
                    None => {
                        eprintln!(
                            "[tyzen] warning: could not resolve flattened type `{}` — its fields will be missing from generated TypeScript",
                            base_name
                        );
                    }
                }
            }
        } else {
            out.push(field);
        }
    }
}

fn render_variant(v: &meta::VariantMeta, e: &meta::EnumMeta, all_metas: &[&TypeMeta]) -> String {
    let tag_name = e.tag.unwrap_or("tag");
    let content_name = e.content;

    match &v.fields {
        meta::VariantFields::Unit => {
            if e.untagged {
                "null".to_string()
            } else {
                format!("{{ {}: \"{}\" }}", tag_name, v.name)
            }
        }
        meta::VariantFields::Unnamed(types) => {
            let values_ts = if types.len() == 1 {
                (types[0])()
            } else {
                let inner = types.iter().map(|t| t()).collect::<Vec<_>>().join(", ");
                format!("[{}]", inner)
            };

            if e.untagged {
                values_ts
            } else if let Some(content) = content_name {
                format!(
                    "{{ {}: \"{}\", {}: {} }}",
                    tag_name, v.name, content, values_ts
                )
            } else {
                format!("{{ {}: \"{}\", value: {} }}", tag_name, v.name, values_ts)
            }
        }
        meta::VariantFields::Named(fields) => {
            let mut all_fields = Vec::new();
            collect_fields_from_list(fields, all_metas, &mut all_fields);

            let fields_str = all_fields
                .iter()
                .map(|f| {
                    let label = if f.optional {
                        format!("{}?", f.name)
                    } else {
                        f.name.to_string()
                    };
                    format!("{}: {}", label, (f.ty_name)())
                })
                .collect::<Vec<_>>()
                .join(", ");

            if e.untagged {
                format!("{{ {} }}", fields_str)
            } else if let Some(content) = content_name {
                format!(
                    "{{ {}: \"{}\", {}: {{ {} }} }}",
                    tag_name, v.name, content, fields_str
                )
            } else {
                format!("{{ {}: \"{}\", {} }}", tag_name, v.name, fields_str)
            }
        }
    }
}

/// Writes framework-agnostic TypeScript command type declarations.
///
/// Emits a `Commands` type whose methods match each registered `#[tyzen::command]`,
/// mapping Rust snake_case names to TypeScript camelCase.
///
/// This is used internally by [`generate`] and can be passed to
/// [`generate_full`] as the `write_before_types` callback.
pub fn write_command_declarations(ts: &mut String) {
    let mut commands: Vec<_> = inventory::iter::<CommandMeta>().collect();
    if commands.is_empty() {
        return;
    }
    commands.sort_by_key(|c| c.name);

    ts.push_str("/** autogen commands **/\n");
    ts.push_str("export type Commands = {\n");
    for cmd in commands {
        let fn_name = snake_to_camel(cmd.name);
        let params_ts: Vec<String> = cmd
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, (p.ty)()))
            .collect();

        ts.push_str(&format!(
            "  {}({}): Promise<{}>\n",
            fn_name,
            params_ts.join(", "),
            (cmd.return_type)(),
        ));
    }
    ts.push_str("}\n");
}
