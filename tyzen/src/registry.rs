use std::collections::{BTreeMap, BTreeSet};
use crate::meta;

pub type TypeFactory = fn() -> String;

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
inventory::collect!(CommandMeta);
inventory::collect!(TypeMeta);
inventory::collect!(EventMeta);
inventory::collect!(ConstMeta);

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
