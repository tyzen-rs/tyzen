# Changelog v0.2.0

> [!IMPORTANT]
> **LLM Context**: This release establishes the core "Facts vs Instructions" architecture for Tyzen. Agents should refer to this for foundational namespacing and metadata patterns.

Tyzen v0.2.0 is a major milestone focusing on **Scalability**, **Modularity**, and **Developer Experience**. This release introduces namespacing for large-scale projects and a powerful metadata inheritance system.

## 🚀 Major Features

### 📦 Namespaces & SDK Generation
- **Module-level Namespacing**: Use `tyzen::module_ns!("Namespace")` to automatically group all types and commands in a module into a single TypeScript action object.
- **Smart SDK Pattern**: Generated TypeScript now follows a "Namespaced Object" pattern (e.g., `Task.create()`, `Auth.login()`), providing an SDK-like experience with IDE autocomplete.
- **Auto-Naming Strategy**: Introduced `NamingStrategy` (Prefix/Postfix) to automatically strip redundant prefixes from namespaced functions (e.g., `task_create` becomes `Task.create()`).

### 🧬 Variant Metadata Templates
- **Metadata Inheritance**: Share attributes across multiple enums using `#[tyzen(apply = TemplateName)]`.
- **Shadow Cache Mechanism**: Implemented a JSON-based persistence layer with global in-memory caching to ensure templates are resolved efficiently across macro boundaries.
- **Precise Error Diagnostics**: Compile-time validation now points exactly to the offending attribute span if a template is missing or invalid.

### 🛡️ Enhanced Type Safety
- **Robust Result Mapping**: Deep integration for `Result<T, E>`. Custom error enums are now correctly mapped to TypeScript discriminated unions.
- **Channel Support**: Added support for `tauri::ipc::Channel<T>`, mapping them to TypeScript callbacks for real-time data streaming.

## 🛠️ Internal Improvements
- **Performance**: Optimized macro execution using global caching, reducing disk I/O by 90% for large template-heavy projects.
- **Reliability**: Replaced manual text parsing with `serde_json` for metadata persistence to prevent data corruption.
- **Documentation**: Comprehensive `rustdoc` added to all internal macro logic and public API items.

## ⚠️ Breaking Changes
- The generated TypeScript structure has changed to a namespaced layout. Ensure you update your frontend imports to use the new namespaced objects.
- `tyzen-tauri` now requires Tauri v2.

---
*For a full list of attributes and types, see the [Reference](./reference.md).*
