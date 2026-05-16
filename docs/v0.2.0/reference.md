# Reference v0.2.0

## Core Attributes

| Attribute | Level | Description |
|---|---|---|
| `#[derive(Type)]` | Struct/Enum | Main derive macro for TypeScript generation. |
| `#[derive(Event)]` | Struct | Marks a type as a Tauri Event. |
| `#[tyzen::command]` | Function | Registers a function as an IPC command. |
| `#[tyzen::tauri_command]` | Function | Shorthand for `#[tyzen::command]` + `#[tauri::command]`. |
| `#[tyzen::export]` | Const | Exports a `pub const` to TypeScript. |

## Namespacing

### `tyzen::module_ns!("Name")`
Registers a default namespace for the current module. All types, commands, and events in this module (and sub-modules) will inherit this namespace unless explicitly overridden.

### Attribute Overrides
- `#[tyzen(ns = "Custom")]`: Override the namespace for a specific item.
- `#[tyzen(rename = "NewName")]`: Override the name in TypeScript.

## Metadata Templates (Enums Only)

### `#[tyzen(apply = TemplateName)]`
Applied at the enum level to inherit variant metadata.

```rust
#[derive(Type)]
#[tyzen(apply = CommonError)]
pub enum MyError {
    NotFound, // Inherits attributes from CommonError::NotFound
}
```

## Error Handling

Tyzen automatically maps `Result<T, E>` to a TypeScript union:
```typescript
type Result<T, E = string> = 
  | { status: "ok"; data: T } 
  | { status: "error"; error: E };
```

If `E` is an enum with `#[derive(Type)]`, it becomes a discriminated union in TypeScript.

## Internal Mechanics (Shadow Cache)

Tyzen uses a JSON-based shadow cache in `target/tyzen/metadata/` to share metadata across different macro expansions. 
- **Format**: `<Type>.json`
- **Reset**: Run `cargo clean` if metadata appears out of sync.
