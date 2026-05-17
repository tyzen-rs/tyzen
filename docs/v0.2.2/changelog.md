# Changelog v0.2.2

> [!TIP]
> This release brings major type-safety advancements, high-performance binary serialization, extreme developer ergonomics, and structural code modularization.

## 🚀 Key Features & Enhancements

### 🛡️ TypeScript Hoisting & Compilation Safety
- **Sequential Code Generation**: Reordered the generated TypeScript sequence to `Type -> Meta -> Namespace -> Commands` with enums sorted to the top.
- **Block-Scoped Reference Safety**: Eliminates the TypeScript `block-scoped variable used before its declaration` errors (TS2448 / TS2454) when referencing enum values (e.g. `Status.Todo`) inside metadata blocks.

### 🎨 Path-Based Metadata Expressions
- **Unquoted Enum Attributes**: Upgraded the attribute parser to accept unquoted Rust path expressions and arrays of paths inside `#[tyzen(meta(...))]` blocks (e.g. `taskStatuses = [Status::Todo, Status::InProgress]`).
- **Constant References**: Emits raw unquoted TypeScript constants (e.g. `Status.Todo`) instead of hardcoded string literals inside metadata objects, preserving strict compiler synchronization.

### 🧹 Zero-Boilerplate Commands
- **Consolidated `#[tyzen_tauri::command]`**: The macro now automatically prepends and expands `#[tauri::command]` under the hood.
- **Footgun Elimination**: Halves boilerplate and eliminates compile mismatches or silent binding generation drift caused by forgetting to stack macros.

### ⚡ High-Performance Binary Data Transfers
- **Direct Byte Hydration**: Seamlessly maps binary fields (like `Vec<u8>`) to JavaScript `Uint8Array`.
- **Automatic Transformers**: Emits standalone file `helpers.ts` containing the standard Tauri helper suite (`__invoke`, `__listen`, `toBinary`, `parseError`) and automatically registers mapping transformations on commands returning binary fields.

---

## 🛠️ Refactoring & Code Quality

### 🧱 Modular Architecture Reorganization
- Monolithic `tyzen/src/lib.rs` split into specialized internal modules:
  - `lib.rs` (public API / macro exports)
  - `registry.rs` (metadata collection models)
  - `generator.rs` (orchestrates file output and namespace maps)
  - `renderer.rs` (printers for types, structs, enums, and metadata objects)

### 🧼 Parameter Formatting & Output Stability
- **Underscore Stripping**: Cleaned command wrapper signatures by automatically stripping leading underscores (e.g. `_id` becomes `id`) from frontend-facing API signatures.
- **Deterministic Metadata Outputs**: Enum variant metadata constants are now alphabetically sorted before output, preventing chaotic Git diff noise.

---
*For older releases, see [v0.2.1](../v0.2.1/changelog.md).*
