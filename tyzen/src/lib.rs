/// Private re-exports for macro-generated code. Not part of the public API.
#[doc(hidden)]
pub mod __private {
    pub use inventory;
}

pub use tyzen_macro::{Event, Type, command, event, export};
pub mod ts_type;

use std::collections::{BTreeMap, BTreeSet};

/// Registers a default namespace for the current module and all its sub-modules.
///
/// Use this at the top of a module (usually `mod.rs`) to avoid setting `ns` on every item.
///
/// # Examples
///
/// ```rust
/// tyzen::module_ns!("Task");
/// ```
#[macro_export]
macro_rules! module_ns {
    ($ns:expr) => {
        ::tyzen::__private::inventory::submit! {
            ::tyzen::ModuleNamespaceMeta {
                path: module_path!(),
                ns: $ns,
            }
        }
    };
}
use crate::utils::snake_to_camel;

pub mod meta;
pub mod utils;

/// Groups all Tyzen metadata by their namespace for coordinated generation.
#[derive(Default)]
pub struct NamespaceMap<'a> {
    pub types: BTreeMap<Option<&'a str>, Vec<&'a TypeMeta>>,
    pub commands: BTreeMap<Option<&'a str>, Vec<&'a CommandMeta>>,
    pub events: BTreeMap<Option<&'a str>, Vec<&'a EventMeta>>,
    pub consts: BTreeMap<Option<&'a str>, Vec<&'a ConstMeta>>,
    pub namespaces: BTreeSet<&'a str>,
}

impl<'a> NamespaceMap<'a> {
    pub fn collect() -> Self {
        let mut map = Self::default();
        
        let mut module_ns: Vec<_> = inventory::iter::<ModuleNamespaceMeta>().collect();
        // Sort by path length descending so more specific paths (longer) match first
        module_ns.sort_by_key(|m| std::cmp::Reverse(m.path.len()));

        let resolve_ns = |item_ns: Option<&'static str>, module_path: &'static str| -> Option<&'static str> {
            if item_ns.is_some() {
                return item_ns;
            }
            for reg in &module_ns {
                if module_path.starts_with(reg.path) {
                    return Some(reg.ns);
                }
            }
            None
        };

        for m in inventory::iter::<TypeMeta>() {
            let ns = resolve_ns(m.ns, m.module_path);
            map.types.entry(ns).or_default().push(m);
            if let Some(ns) = ns { map.namespaces.insert(ns); }
        }
        for m in inventory::iter::<CommandMeta>() {
            let ns = resolve_ns(m.ns, m.module_path);
            map.commands.entry(ns).or_default().push(m);
            if let Some(ns) = ns { map.namespaces.insert(ns); }
        }
        for m in inventory::iter::<EventMeta>() {
            let ns = resolve_ns(m.ns, m.module_path);
            map.events.entry(ns).or_default().push(m);
            if let Some(ns) = ns { map.namespaces.insert(ns); }
        }
        for m in inventory::iter::<ConstMeta>() {
            let ns = resolve_ns(m.ns, m.module_path);
            map.consts.entry(ns).or_default().push(m);
            if let Some(ns) = ns { map.namespaces.insert(ns); }
        }

        // Sort everything for deterministic output
        for v in map.types.values_mut() { v.sort_by_key(|m| m.name); }
        for v in map.commands.values_mut() { v.sort_by_key(|m| m.name); }
        for v in map.events.values_mut() { v.sort_by_key(|m| m.name); }
        for v in map.consts.values_mut() { v.sort_by_key(|m| m.name); }

        map
    }
}

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
    /// The namespace this command belongs to.
    pub ns: Option<&'static str>,
    /// Optional rename for the command in the generated TypeScript.
    pub rename: Option<&'static str>,
    /// The Rust module path where this command is defined.
    pub module_path: &'static str,
    /// Whether the return type is binary.
    pub is_binary: bool,
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
    /// The namespace this type belongs to.
    pub ns: Option<&'static str>,
    /// The Rust module path where this type is defined.
    pub module_path: &'static str,
    /// Whether this type or its children contain binary data.
    pub has_binary: bool,
}

/// Metadata describing a typed event registered for TypeScript generation.
///
/// Generated automatically by `#[derive(tyzen::Event)]`. Not intended for manual use.
pub struct EventMeta {
    /// The event name emitted over the wire (e.g. `"user-joined"`).
    pub name: &'static str,
    /// Returns the TypeScript type name for the event payload.
    pub payload_type: fn() -> String,
    /// The namespace this event belongs to.
    pub ns: Option<&'static str>,
    /// The Rust module path where this event is defined.
    pub module_path: &'static str,
    /// Whether the payload type is binary.
    pub is_binary: bool,
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
    /// The namespace this constant belongs to.
    pub ns: Option<&'static str>,
    /// The Rust module path where this constant is defined.
    pub module_path: &'static str,
}

/// Metadata for registering a default namespace for a module path.
pub struct ModuleNamespaceMeta {
    pub path: &'static str,
    pub ns: &'static str,
}

inventory::collect!(ModuleNamespaceMeta);

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

/// Controls how function names are automatically cleaned when in a namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamingStrategy {
    /// Strips the namespace prefix (e.g., `task_create` -> `create`).
    Prefix,
    /// Strips the namespace postfix (e.g., `create_task` -> `create`).
    Postfix,
}

/// Configuration for the TypeScript generator.
#[derive(Debug, Clone, Copy)]
pub struct GeneratorConfig {
    /// How to automatically rename functions in namespaces if no explicit rename is provided.
    pub naming_strategy: NamingStrategy,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            naming_strategy: NamingStrategy::Prefix,
        }
    }
}

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
fn strip_naming(name: &str, ns: &str, is_type: bool, strategy: NamingStrategy) -> String {
    if is_type {
        let stripped = name.replace(ns, "");
        if stripped.is_empty() {
            return name.to_string();
        }
        return stripped;
    }

    let ns_norm = ns.to_lowercase();
    let separator = "_";
    
    match strategy {
        NamingStrategy::Prefix => {
            let prefix = format!("{}{}", ns_norm, separator);
            if name.starts_with(&prefix) && name.len() > prefix.len() {
                name[prefix.len()..].to_string()
            } else {
                name.to_string()
            }
        }
        NamingStrategy::Postfix => {
            let postfix = format!("{}{}", separator, ns_norm);
            if name.ends_with(&postfix) && name.len() > postfix.len() {
                name[..name.len() - postfix.len()].to_string()
            } else {
                name.to_string()
            }
        }
    }
}

pub fn generate(output_path: &str) -> std::io::Result<()> {
    generate_full(output_path, GeneratorConfig::default(), write_command_declarations, |_| {})
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
    generate_full(output_path, GeneratorConfig::default(), write_commands, |_| {})
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
    config: GeneratorConfig,
    write_before_types: impl FnOnce(&mut String),
    write_after_types: impl FnOnce(&mut String),
) -> std::io::Result<()> {
    use std::fs;
    use std::path::PathBuf;

    let path = PathBuf::from(output_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let map = NamespaceMap::collect();
    let all_types: Vec<&TypeMeta> = map.types.values().flatten().cloned().collect();

    let mut ts = String::new();
    ts.push_str("// auto-generated by tyzen, do not edit\n\n");
    write_before_types(&mut ts);

    // 1. Global Types (for ergonomics)
    ts.push_str("\n/** Global Types **/\n");
    let mut sorted_all_types = all_types.clone();
    sorted_all_types.sort_by_key(|t| t.name);
    for t in sorted_all_types {
        ts.push_str(&render_type(t, &all_types));
        ts.push('\n');
    }

    if let Some(root_consts) = map.consts.get(&None) {
        for c in root_consts {
            ts.push_str(&format!("export const {} = {} as const;\n", c.name, c.value));
        }
    }

    // 2. Namespaced Scopes
    for ns in &map.namespaces {
        ts.push_str(&format!("\n/** Namespace: {} **/\n", ns));

        // 2a. Types within namespace (using stripped names)
        if let Some(ns_types) = map.types.get(&Some(ns)) {
            ts.push_str(&format!("export namespace {} {{\n", ns));
            for t in ns_types {
                let stripped_name = strip_naming(t.name, ns, true, config.naming_strategy);
                // Avoid circular reference if stripping leaves the same name
                if stripped_name != t.name {
                   ts.push_str(&format!("  export type {} = {};\n", stripped_name, t.name));
                }
            }
            ts.push_str("}\n");
        }

        // 2b. Merged Action Object (Commands, Events, Constants)
        let ns_commands = map.commands.get(&Some(ns));
        let ns_consts = map.consts.get(&Some(ns));
        let ns_events = map.events.get(&Some(ns));

        if ns_commands.is_some() || ns_consts.is_some() || ns_events.is_some() {
            ts.push_str(&format!("export const {} = {{\n", ns));

            if let Some(consts) = ns_consts {
                for c in consts {
                    ts.push_str(&format!("  {}: {} as const,\n", c.name, c.value));
                }
            }

            if let Some(commands) = ns_commands {
                for cmd in commands {
                    let fn_name = cmd.rename.map(|r| r.to_string()).unwrap_or_else(|| {
                        let base = strip_naming(cmd.name, ns, false, config.naming_strategy);
                        snake_to_camel(&base)
                    });
                    let params: Vec<String> = cmd.params.iter().map(|p| format!("{}: {}", p.name, (p.ty)())).collect();
                    ts.push_str(&format!("  {}: ({}) => __invoke<{}>(\"{}\", {{ {} }}),\n", 
                        fn_name, 
                        params.join(", "), 
                        (cmd.return_type)(), 
                        cmd.name,
                        cmd.params.iter().map(|p| p.name).collect::<Vec<_>>().join(", ")
                    ));
                }
            }

            if let Some(events) = ns_events {
                for ev in events {
                    let camel_base = strip_naming(ev.name, ns, false, config.naming_strategy);
                    let camel_name = snake_to_camel(camel_base.replace("-", "_").replace(":", "_").as_str());
                    let ev_fn_name = format!("on{}{}", (&camel_name[0..1]).to_uppercase(), &camel_name[1..]);
                    ts.push_str(&format!("  {}: (cb: (payload: {}) => void) => __listen(\"{}\", cb),\n", 
                        ev_fn_name, (ev.payload_type)(), ev.name));
                }
            }

            // Also include Enum Metadata if a type with the same name exists
            if let Some(ns_types) = map.types.get(&Some(ns)) {
                if let Some(matching_type) = ns_types.iter().find(|t| t.name == *ns) {
                    if let meta::TypeStructure::Enum(e) = (matching_type.structure)() {
                        if !e.variants.iter().all(|v| v.attrs.is_empty()) {
                            ts.push_str("  Meta: {\n");
                            for v in e.variants {
                                let mut attrs = Vec::new();
                                for (k, val) in v.attrs {
                                    attrs.push(format!("{}: \"{}\"", snake_to_camel(k), val));
                                }
                                let attrs_str = attrs.join(", ");
                                if attrs_str.is_empty() {
                                    ts.push_str(&format!("    {}: {{}},\n", v.name));
                                } else {
                                    ts.push_str(&format!("    {}: {{ {} }},\n", v.name, attrs_str));
                                }
                            }
                            ts.push_str("  } as const,\n");
                        }
                    }
                }
            }

            ts.push_str("} as const;\n");
        }
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
    let def = match structure {
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
            let enum_def = if !e.untagged
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
                    "export const {} = ELECT_UNTIL_TUPLES_READY || {{\n{}\n}} as const;\nexport type {}{} = (typeof {})[keyof typeof {}]",
                    meta.name, fields, meta.name, meta.generic_params, meta.name, meta.name
                )
            } else {
                let variants: Vec<String> = e
                    .variants
                    .iter()
                    .map(|v| render_variant(v, &e, all_metas))
                    .collect();
                format!("export type {}{} = {}", meta.name, meta.generic_params, variants.join(" | "))
            };

            // Fix the ELECT_UNTIL_TUPLES_READY compilation string back to original
            let enum_def = enum_def.replace("ELECT_UNTIL_TUPLES_READY || ", "");

            if let Some(meta_obj) = render_enum_meta(meta, &e) {
                format!("{}\n\n{}", enum_def, meta_obj)
            } else {
                enum_def
            }
        }
        meta::TypeStructure::Transparent(inner) => {
            format!("export type {}{} = {}", meta.name, meta.generic_params, inner)
        }
        meta::TypeStructure::Unit => {
            format!("export type {} = null", meta.name)
        }
    };

    if let Some(transformer) = render_transformer(meta, all_metas) {
        format!("{}\n{}", def, transformer)
    } else {
        def
    }
}

#[allow(dead_code)]
fn has_binary_data(structure: &meta::TypeStructure, all_metas: &[&TypeMeta]) -> bool {
    match structure {
        meta::TypeStructure::Struct(s) => {
            let mut fields = Vec::new();
            collect_fields_from_list(s.fields, all_metas, &mut fields);
            fields.iter().any(|f| f.is_binary)
        }
        meta::TypeStructure::Enum(e) => {
            e.variants.iter().any(|v| match &v.fields {
                meta::VariantFields::Named(fields) => {
                   let mut all_fields = Vec::new();
                   collect_fields_from_list(fields, all_metas, &mut all_fields);
                   all_fields.iter().any(|f| f.is_binary)
                }
                _ => false, // Tuple hydration deferred to v0.3.0
            })
        }
        _ => false,
    }
}

fn render_transformer(meta: &TypeMeta, all_metas: &[&TypeMeta]) -> Option<String> {
    if !meta.has_binary {
        return None;
    }
    let structure = (meta.structure)();

    match structure {
        meta::TypeStructure::Struct(s) => {
            let mut fields = Vec::new();
            collect_fields_from_list(s.fields, all_metas, &mut fields);
            let binary_fields: Vec<_> = fields.iter().filter(|f| f.is_binary).collect();
            
            let mut transformations = Vec::new();
            for f in binary_fields {
                transformations.push(format!("  (res as any).{} = toBinary((res as any).{});", f.name, f.name));
            }

            Some(format!(
                "export function to{}(res: {}): {} {{\n{}\n  return res;\n}}",
                meta.name, meta.name, meta.name, transformations.join("\n")
            ))
        }
        _ => None, // Enum transformer deferred
    }
}

pub fn render_enum_meta(type_meta: &TypeMeta, e: &meta::EnumMeta) -> Option<String> {
    if e.variants.iter().all(|v| v.attrs.is_empty()) {
        return None;
    }

    let meta_name = e.meta_name.unwrap_or_else(|| {
        Box::leak(format!("{}Meta", type_meta.name).into_boxed_str())
    });

    let mut lines = Vec::new();
    for v in e.variants {
        let mut attrs = Vec::new();
        for (k, v) in v.attrs {
            attrs.push(format!("{}: \"{}\"", snake_to_camel(k), v));
        }
        let attrs_str = attrs.join(", ");
        if attrs_str.is_empty() {
            lines.push(format!("  {}: {{}}", v.name));
        } else {
            lines.push(format!("  {}: {{ {} }}", v.name, attrs_str));
        }
    }

    Some(format!(
        "export const {} = {{\n{}\n}} as const;",
        meta_name,
        lines.join(",\n")
    ))
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
