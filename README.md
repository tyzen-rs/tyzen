# Tyzen ⚡

Tyzen is a high-performance developer tool that bridges the gap between **Rust** and **TypeScript**. Its primary goal is to provide a seamless, type-safe development experience (DX) by automating the synchronization of data structures and API definitions.

While Tyzen was built with **Tauri** in mind, its core is a generic, lightning-fast type generator that can be used in any Rust/TS project.

## ✨ Why Tyzen?

- 🚀 **Performance**: Generates complex TypeScript bindings in milliseconds.
- 🛠️ **Developer Experience (DX)**: Zero-config command registration and strongly-typed events.
- 📦 **Transparency**: Keeps your code clear and explicit.
- 🔌 **Tauri Integration**: First-class support for Tauri commands and events with automatic registration.

---

## 🚀 Quick Start

### 1. Add Dependencies

For core type generation:
```bash
cargo add tyzen
cargo add serde --features derive
```

For **Tauri** integration (Optional):
```bash
cargo add tyzen-tauri
```

### 2. Define Your Types (Rust)

Tyzen uses simple attributes to mark structs, enums, and functions.

```rust
use serde::{Serialize, Deserialize};

// 1. Convert any Rust struct/enum to TS
#[derive(tyzen::Type, Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub name: String,
}

// 2. Define a typed Event
#[derive(tyzen::Type, tyzen::Event, Serialize)]
pub struct WelcomeEvent {
    pub message: String,
}
```

### 3. Setup the Generator

In your `main.rs` (or any build/tool script), initialize the generator:

```rust
fn main() {
    tyzen::generate("../src/bindings.ts");
}
```

---

## 🔌 Tauri Integration

For Tauri projects, `tyzen-tauri` provides a wrapper that automates command registration and event handling.

### 1. Define Tauri Commands

No stacked macros needed! The `#[tyzen_tauri::command]` macro automatically expands and registers your commands with Tauri under the hood.

```rust
#[tyzen_tauri::command] // Marks for TS generation & auto-registers with Tauri
pub fn create_user(name: String) -> Result<User, String> {
    Ok(User { id: 1, name })
}
```

### 2. Setup Generator & Handler

```rust
fn main() {
    // 1. Generate TS bindings with Tauri support
    #[cfg(debug_assertions)]  
    tyzen_tauri::generate("../src/bindings.ts"); 

    // 2. Setup Tauri with auto-registration
    tauri::Builder::default()
        .invoke_handler(tyzen_tauri::handler!()) // Auto-registers all #[tyzen_tauri::command]
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## 💻 Frontend Usage (TypeScript/React)

Tyzen creates a clean, intuitive API for your frontend.

```tsx
import { useEffect, useState } from 'react';
import { commands, events } from "./bindings";

function App() {
  const [status, setStatus] = useState("Ready");

  useEffect(() => {
    // 1. Call a command (Type-safe Promise)
    commands.createUser("rzust").then(res => {
       if (res.status === 'ok') console.log("Success:", res.data);
    });

    // 2. Listen to an event (Fluent API)
    const unlisten = events.welcome.listen((payload) => {
      setStatus(`Message: ${payload.message}`);
    });

    return () => { unlisten.then(f => f()); };
  }, []);

  return <h1>{status}</h1>;
}
```

---

## 📖 Feature Guide

### Standard Type Conversion
Use `#[derive(tyzen::Type)]` on any Rust type. It supports primitives, `Vec`, `Option`, `HashMap`, and even complex **Generics**.

### Tauri Commands
The `#[tyzen_tauri::command]` macro is the only attribute you need to declare a type-safe Tauri command. It automatically handles code generation and expands `#[tauri::command]` so there is zero boilerplate or stacked-macro footguns.

### Typed Events
When you derive `tyzen::Event`, Tyzen adds a helper `.emit(&handle)` method to your struct:
```rust
let event = WelcomeEvent { message: "Hi!".into() };
event.emit(&handle).ok(); // Correctly types the payload for the frontend
```

---

## 🗺️ Roadmap & Status

| Feature | Importance | Notes |
| :--- | :--- | :--- |
| **Full Serde Parity** | Implemented | `flatten`, `alias`, `default`, and `rename_all` support. Requires inter-type metadata. |
| **Binary Data** | Implemented | Automatically maps `Vec<u8>` or fields marked with `#[tyzen(binary)]` to `Uint8Array` with transparent hydration. |
| **Result & Error** | Implemented | Deep support for custom Rust error types and enum variant metadata blocks in frontend. |
| **Constant Export** | Implemented | Sync `pub const` logic values from Rust to TS. |
| **Namespaces** | Implemented | Organize types and commands into logical Models (SDK style). |
| **Validation Sync** | Core | Bridge `validator` rules (Rust) directly into Zod schemas (TS). |
| **Zod Support** | Will implement | Generate Zod schemas alongside types for runtime frontend validation. |
| **Mock Client** | Will implement | Generate mock JS/TS clients for testing/UI prototyping without the backend. |
| **Doc Propagation** | | Transform Rust doc comments (`///`) into TSDoc (`/** ... */`). |

---

## 📦 Packages

- `tyzen`: The core engine for type conversion.
- `tyzen-macro`: Procedural macros for `Type` and `Event`.
- `tyzen-tauri`: Specialized integration for Tauri (commands, event emitters, and TS glue code).

## 📜 License

Distributed under the MIT / Apache-2.0 License.
