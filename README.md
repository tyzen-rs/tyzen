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

Run the following commands in your project directory:

```bash
cargo add tyzen
cargo add tyzen-tauri  # Optional: for Tauri specialized features
cargo add serde --features derive
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

// 3. Define a Tauri Command
#[tauri::command]
#[tyzen_tauri::command] // Marks for TS generation & auto-registration
pub fn create_user(name: String) -> Result<User, String> {
    Ok(User { id: 1, name })
}
```

### 3. Setup the Generator & Handler

In your `main.rs`, initialize the generator and use the `handler!` macro to register all commands automatically.

```rust
fn main() {
    // 1. Generate TS bindings (Run this in debug mode)
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
    commands.createUser("Silverian").then(res => {
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
Tyzen requires both `#[tauri::command]` and `#[tyzen_tauri::command]` for clarity. 
- `#[tauri::command]`: Tells Tauri this is an invocable command.
- `#[tyzen_tauri::command]`: Tells Tyzen to generate TS metadata and include it in the `handler!()` macro.

### Typed Events
When you derive `tyzen::Event`, Tyzen adds a helper `.emit(&handle)` method to your struct:
```rust
let event = WelcomeEvent { message: "Hi!".into() };
event.emit(&handle).ok(); // Correctly types the payload for the frontend
```

---

## 📦 Packages

- `tyzen`: The core engine for type conversion.
- `tyzen-macro`: Procedural macros for `Type` and `Event`.
- `tyzen-tauri`: Specialized integration for Tauri (commands, event emitters, and TS glue code).

## 📜 License

Distributed under the MIT / Apache-2.0 License.
