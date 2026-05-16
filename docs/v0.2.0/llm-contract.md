# Tyzen LLM Contract v0.2.0

This contract defines the strict rules for AI agents modifying or generating code within the Tyzen ecosystem.

## 1. Metadata Persistence (Shadow Cache)
- **Mechanism**: All `#[derive(Type)]` enums persist their variant metadata to `target/tyzen/metadata/<Ident>.json`.
- **Constraint**: NEVER attempt to read or write these files manually in code. Use `tyzen_macro::metadata` module functions.
- **Troubleshooting**: If metadata seems "stale", suggest `cargo clean`.

## 2. Namespacing Rules
- **Prefer Module-level**: Use `tyzen::module_ns!("Name")` in `mod.rs` or `lib.rs` instead of individual `ns` attributes for better scalability.
- **Naming Consistency**: When using namespaces, keep Rust function names in `snake_case`. Tyzen will handle the `camelCase` conversion and prefix stripping automatically.

## 3. Type Mapping
- **Generics**: Tyzen supports recursive generic mapping (e.g., `Vec<Option<T>>`). 
- **Framework Types**: Do NOT manually filter `State<T>`, `Window`, or `AppHandle` in your logic. The `tyzen-macro` already handles this in `is_framework_param`.
- **Channels**: Use `tauri::ipc::Channel<T>` for streaming. It maps to `(payload: T) => void` in TypeScript.

## 4. Documentation Standard
- **Rustdoc**: Every new macro feature or internal utility MUST have `///` documentation explaining its purpose and any non-obvious side effects.
- **Code Generation**: All generated TypeScript MUST be deterministic (alphabetically sorted).

## 5. Deployment Flow
1. Requirements Analysis.
2. Architecture Review.
3. TDD (Red-Green-Refactor).
4. Triple Quality Review (Code, Arch, Perf).
5. Document Update.

---
*Failure to follow this contract may lead to broken TypeScript bindings or non-deterministic build behavior.*
