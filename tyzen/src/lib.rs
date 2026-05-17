/// Private re-exports for macro-generated code. Not part of the public API.
#[doc(hidden)]
pub mod __private {
    pub use inventory;
}

pub use tyzen_macro::{Event, Type, command, event, export};
pub mod ts_type;
pub mod meta;
pub mod registry;
pub mod renderer;
pub mod generator;
pub mod utils;

// Re-export items for backwards compatibility and external usage
pub use registry::{
    ParamMeta, CommandMeta, TypeMeta, EventMeta, ConstMeta,
    ModuleNamespaceMeta, NamespaceMap, TypeFactory,
};
pub use generator::{
    NamingStrategy, GeneratorConfig, generate, generate_with_commands,
    generate_full, write_command_declarations, strip_naming,
};

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
