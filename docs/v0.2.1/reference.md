# Reference v0.2.1

## Core Attributes

| Attribute | Level | Description |
|---|---|---|
| `#[derive(Type)]` | Struct/Enum | Main derive macro for TypeScript generation. |
| `#[derive(Event)]` | Struct | Marks a type as a Tauri Event. |
| `#[tyzen::command]` | Function | Registers a function as an IPC command. |
| `#[tyzen::tauri_command]` | Function | Shorthand for `#[tyzen::command]` + `#[tauri::command]`. |
| `#[tyzen::export]` | Const | Exports a `pub const` to TypeScript. |

## Error Handling & Utilities

### `parseError<T>(error: T, meta: Record<string, any>)`
A TypeScript utility to map Tyzen errors (discriminated unions) to their human-readable metadata.

- **Status**: New in v0.2.1
- **Feature**: Supports template interpolation for `{value}` and `{key}` placeholders.
- **Usage**:
  ```typescript
  const result = await invoke_command();
  if (result.status === "error") {
      const { message, code } = parseError(result.error, MyErrorMeta);
      console.log(message);
  }
  ```

---
*For older documentation, see [v0.2.0](../v0.2.0/reference.md).*
