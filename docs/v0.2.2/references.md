# Reference v0.2.2

## Core Attributes

| Attribute | Level | Description |
|---|---|---|
| `#[derive(Type)]` | Struct/Enum | Main derive macro for TypeScript generation. Supports advanced `#[tyzen(meta(...))]` attributes. |
| `#[derive(Event)]` | Struct | Marks a type as a Tauri Event. |
| `#[tyzen::command]` | Function | Registers a function as an IPC command for non-Tauri or custom setups. |
| `#[tyzen_tauri::command]` | Function | Consolidates Tauri IPC registration. Automatically emits `#[tauri::command]`. |
| `#[tyzen::export]` | Const | Exports a `pub const` to TypeScript. |

---

## 🎨 Path-Based Enums in Metadata

You can define metadata fields in Rust enums using unquoted path-based expressions, allowing direct constant type synchronization with the frontend.

### Rust Backend
```rust
#[derive(tyzen::Type)]
#[tyzen(meta(
    taskStatuses = [Status::Todo, Status::InProgress, Status::Done, Status::Canceled]
))]
pub enum ProjectStatus {
    Active,
}
```

### TypeScript Generated Equivalent
```typescript
export const ProjectStatusMeta = {
  Active: { taskStatuses: [Status.Todo, Status.InProgress, Status.Done, Status.Canceled] }
} as const;
```

---

## ⚡ High-Performance Binary Data Transfers

Binary type mapping converts fields like `Vec<u8>` or fields decorated with `#[tyzen(binary)]` into standard JavaScript `Uint8Array`s and automatically hydrates them from raw base64/array strings on invocation return.

### Rust Backend
```rust
#[derive(tyzen::Type)]
pub struct ProjectAttachment {
    pub id: u32,
    #[tyzen(binary)]
    pub raw_bytes: Vec<u8>,
}
```

### TypeScript Generated Equivalent
```typescript
export interface ProjectAttachment {
  id: number;
  raw_bytes: Uint8Array;
}

export function toProjectAttachment(res: ProjectAttachment): ProjectAttachment {
  (res as any).raw_bytes = toBinary((res as any).raw_bytes);
  return res;
}
```

---

## 🧹 Consolidated Command Declaration

No extra stacked macros are required. Signatures are automatically formatted by stripping leading implementation-detail underscores (e.g. `_id` -> `id`).

### Rust Backend
```rust
#[tyzen_tauri::command]
pub fn task_create(payload: CreateTaskDto, _state: tauri::State<DbState>) -> TaskResult<()> {
    services::create(payload, &_state)
}
```

### TypeScript Frontend (auto-generated)
```typescript
export const commands = {
  taskCreate: (payload: CreateTaskDto) => __invoke<Result<null, TaskError>>("task_create", { payload }),
}
```

---
*For older documentation, see [v0.2.1](../v0.2.1/references.md) or [v0.2.0](../v0.2.0/reference.md).*
