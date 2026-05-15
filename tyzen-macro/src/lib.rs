use proc_macro::TokenStream;

mod command_attr;
mod event_attr;
mod type_derive;
mod export_attr;

#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    command_attr::command(attr, item)
}

#[proc_macro_attribute]
pub fn tauri_command(attr: TokenStream, item: TokenStream) -> TokenStream {
    command_attr::tauri_command(attr, item)
}

#[proc_macro_attribute]
pub fn event(attr: TokenStream, item: TokenStream) -> TokenStream {
    event_attr::event(attr, item)
}

#[proc_macro_derive(Event, attributes(tyzen, event))]
pub fn derive_event(item: TokenStream) -> TokenStream {
    event_attr::derive_event(item)
}

#[proc_macro_derive(Type, attributes(tyzen, serde))]
pub fn derive_type(item: TokenStream) -> TokenStream {
    type_derive::derive_type(item)
}

#[proc_macro_attribute]
pub fn export(attr: TokenStream, item: TokenStream) -> TokenStream {
    export_attr::export(attr, item)
}
