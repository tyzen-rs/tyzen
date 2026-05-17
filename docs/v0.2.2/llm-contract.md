# Tyzen LLM Contract v0.2.2

> [!IMPORTANT]
> AI Agents MUST follow these rules when writing Tauri command decorators, metadata mappings, and event registration in Tyzen.

## 1. Single Attribute Rule for Tauri Commands
- **Shorthand**: AI Agents MUST only use the single `#[tyzen_tauri::command]` attribute.
- **No Redundancy**: Do NOT stack `#[tauri::command]` on top of it. The macro automatically handles the Tauri expansion under the hood.

## 2. Metadata Path Expressions
- **Path Reference**: AI Agents SHOULD use unquoted path-based expressions (e.g., `taskStatuses = [Status::Todo, Status::InProgress]`) inside `#[tyzen(meta(...))]` declarations.
- **No String Quotes**: Avoid wrapping enum references in string quotes when they map to a defined Rust path.

## 3. Optional vs. Nullable Fields
- When container-level `#[tyzen(optional)]` is active:
  - All `Option<T>` fields map to `field?: T` in TypeScript.
  - Fields marked with explicit `#[tyzen(nullable)]` map to `field: T | null`.
- When container-level `#[tyzen(optional)]` is NOT active:
  - All `Option<T>` fields map to `field: T | null`.

## 4. Binary Data Handling
- Fields containing raw byte arrays or marked with `#[tyzen(binary)]` map to TypeScript `Uint8Array`.
- Automated transformer wrappers are compiled for these structures; do not write manual frontend base64 conversions.

## 5. Event Naming
- When registering events via `#[tyzen_tauri::event]`, if an explicit `name` is omitted, the macro kebab-cases the struct name while stripping the `"Event"` suffix as a default fallback.

## 6. Heritage
- Refer to [v0.2.0 Contract](../v0.2.0/llm-contract.md) for core Namespacing and Metadata Registry fundamentals.
