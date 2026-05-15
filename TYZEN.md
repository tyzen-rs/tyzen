# Tyzen ⚡

```rust
use serde::Serialize;

#[derive(tyzen::Type, Serialize)]
pub struct User {
    pub id: u32,
    pub name: String,
}

#[tauri::command]
#[tyzen_tauri::command]
pub fn get_user(id: u32) -> User {
    User { id, name: "Alice".into() }
}

fn main() {
    tyzen_tauri::generate("../src/bindings.ts").unwrap();
}
```

Tyzen is a high-performance bridge that synchronizes your **Rust** types and **Tauri** commands with **TypeScript**. It eliminates the manual work of writing interfaces and API wrappers, ensuring your frontend is always in sync with your backend.

## 🚀 Quick Start

### 1. Add Dependencies
Add Tyzen to your `Cargo.toml`.

```bash
cargo add tyzen
cargo add tyzen-tauri
cargo add serde --features derive
```

### 2. Mark Your Types
Use `#[derive(tyzen::Type)]` on any struct or enum you want to use in the frontend.

```rust
use serde::{Serialize, Deserialize};

#[derive(tyzen::Type, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub is_completed: bool,
}
```

### 3. Define Commands (Tauri)
Mark your Tauri commands with `#[tyzen_tauri::command]`. This enables both TypeScript generation and automatic registration in the Tauri invoke handler.

```rust
#[tauri::command]
#[tyzen_tauri::command]
pub async fn create_task(title: String) -> Result<Task, String> {
    Ok(Task { id: 1, title, is_completed: false })
}
```

### 4. Generate & Register
In your `lib.rs` or `main.rs`, run the generator and use `tyzen_tauri::handler!()` to register all commands at once.

```rust
pub fn run() {
    // Generate bindings on startup (usually wrapped in debug cfg)
    #[cfg(debug_assertions)]
    tyzen_tauri::generate("../src/bindings.ts").expect("Failed to generate bindings");

    tauri::Builder::default()
        .invoke_handler(tyzen_tauri::handler!()) // Auto-registers all marked commands
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 5. Use in TypeScript
Tyzen generates a `commands` object with fully typed async methods.

```typescript
import { commands } from "./bindings";

async function addTask() {
    const res = await commands.createTask("Buy milk");
    if (res.status === "ok") {
        console.log("New task:", res.data.id);
    } else {
        console.error("Error:", res.error);
    }
}

async function handleTaskError(err: TaskError) {
    const meta = TaskErrorMeta[err];
    console.log(`Error Code: ${meta.code}, Message: ${meta.msg}`);
}
```

---

## 🛠️ Key Features

### Serde Parity
Tyzen respects most `#[serde]` attributes, allowing you to control the TypeScript output structure without changing your Rust code.
- `rename` / `rename_all`
- `tag` / `content` / `untagged` (for Enums)
- `flatten` (for nested structs)
- `skip`

### Typed Events
Define events in Rust and get a fluent, typed API in the frontend.

```rust
#[derive(tyzen::Type, tyzen::Event, Serialize)]
pub struct ProgressEvent {
    pub progress: f32,
}

// In Rust:
progress.emit(&window).ok();

// In TypeScript:
import { events } from "./bindings";
events.progress.listen((p) => console.log(p.progress));
```

### Streaming with Channels
Tyzen automatically detects `tauri::ipc::Channel` and maps it to a callback in TypeScript.

```rust
#[tyzen_tauri::command]
pub fn stream_logs(on_log: tauri::ipc::Channel<String>) {
    on_log.send("Connected".into()).ok();
}

// In TypeScript:
commands.streamLogs((log) => console.log("Log:", log));
```

### Variant Metadata
Attach arbitrary key-value pairs to enum variants. Tyzen generates a type-safe `[EnumName]Meta` object.

```rust
#[derive(tyzen::Type)]
pub enum TaskError {
    #[tyzen(code = "404", msg = "Not Found")]
    NotFound,
    #[tyzen(code = "500", msg = "Server Error")]
    Internal,
}
```

In TypeScript:
```typescript
// Autocomplete works for both keys and values!
const code = TaskErrorMeta.NotFound.code; // "404"

// Easily lookup metadata for a variant instance
const meta = TaskErrorMeta[errorVariant];
```

---

## ⚠️ Gotchas

- **Return Types**: All Tauri commands are wrapped in a `Result<T, E>` in TypeScript to ensure consistent error handling. If your Rust function doesn't return a `Result`, Tyzen will wrap it in one.
- **Explicit Attributes**: Both `#[tauri::command]` and `#[tyzen_tauri::command]` are required. This is intentional to keep the code explicit and avoid "magic" behavior that could conflict with other Tauri plugins.

## Keep this doc updated when:
- New Serde attributes are supported.
- The `handler!()` macro registration logic changes.
- New frontend frameworks (like Zod or Valibot) are supported.
