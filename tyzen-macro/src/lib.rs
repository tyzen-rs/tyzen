//! Procedural macros for the Tyzen ecosystem.
//! 
//! This crate provides the bridge between Rust types/functions and TypeScript code generation.
//! It handles attribute parsing, type metadata collection, and code expansion for Tauri commands.

use proc_macro::TokenStream;

mod command_attr;
mod event_attr;
mod export_attr;
mod type_derive;

/// Marks a function as a Tyzen command.
/// 
/// This macro collects metadata about the function's parameters and return type to generate
/// matching TypeScript bindings. It does NOT emit `#[tauri::command]`.
/// 
/// # Example
/// ```rust
/// #[tyzen::command(namespace = "auth")]
/// fn login(username: String) -> Result<User, Error> { ... }
/// ```
#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    command_attr::command(attr, item)
}

/// Marks a function as both a Tyzen command and a Tauri command.
/// 
/// Shorthand for applying both `#[tyzen::command]` and `#[tauri::command]`.
#[proc_macro_attribute]
pub fn tauri_command(attr: TokenStream, item: TokenStream) -> TokenStream {
    command_attr::tauri_command(attr, item)
}

/// Marks a struct as a Tyzen Event.
/// 
/// Used to synchronize events emitted from Tauri to the frontend.
#[proc_macro_attribute]
pub fn event(attr: TokenStream, item: TokenStream) -> TokenStream {
    event_attr::event(attr, item)
}

/// Derives the `TsEvent` trait for a struct.
/// 
/// This allows the struct to be used with the `emit` utility on the frontend.
#[proc_macro_derive(Event, attributes(tyzen, event))]
pub fn derive_event(item: TokenStream) -> TokenStream {
    event_attr::derive_event(item)
}

/// Derives the `TsType` trait and registers metadata for TypeScript generation.
/// 
/// Supports both structs and enums. Correctly handles `serde` attributes
/// and Tyzen-specific metadata like `#[tyzen(optional)]` or `#[tyzen(apply = Template)]`.
#[proc_macro_derive(Type, attributes(tyzen, serde))]
pub fn derive_type(item: TokenStream) -> TokenStream {
    type_derive::derive_type(item)
}

/// Exports a constant value to TypeScript.
/// 
/// Useful for sharing configuration or magic numbers between the backend and frontend.
/// 
/// # Example
/// ```rust
/// #[tyzen::export]
/// pub const MAX_RETRIES: u32 = 5;
/// ```
#[proc_macro_attribute]
pub fn export(attr: TokenStream, item: TokenStream) -> TokenStream {
    export_attr::export(attr, item)
}
