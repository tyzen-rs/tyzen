use proc_macro::TokenStream;

mod command_attr;
mod event_attr;
mod type_derive;

#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    command_attr::command(attr, item)
}

#[proc_macro_attribute]
pub fn tauri_command(attr: TokenStream, item: TokenStream) -> TokenStream {
    command_attr::tauri_command(attr, item)
}

#[proc_macro_derive(Event, attributes(tyzen))]
pub fn derive_event(item: TokenStream) -> TokenStream {
    event_attr::derive_event(item)
}

#[proc_macro_derive(Type, attributes(tyzen, serde))]
pub fn derive_type(item: TokenStream) -> TokenStream {
    type_derive::derive_type(item)
}
