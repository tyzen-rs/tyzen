# Tyzen LLM Contract v0.2.1

> [!IMPORTANT]
> This contract extends v0.2.0. AI Agents MUST follow these rules when handling error metadata.

## 1. Error Parsing Rules (New)
- **Utility**: Always use the generated `parseError` function when handling Tyzen errors in the frontend.
- **Interpolation**: Ensure that error metadata templates use `{value}` or `{key}` placeholders that match the variant field names.
- **Contract**: The `parseError` function is the ONLY source of truth for mapping tags to human-readable messages.

## 2. Workspace Consistency
- **Versions**: All Tyzen crates (`tyzen`, `tyzen-macro`, `tyzen-tauri`) MUST be kept at the same version level (currently v0.2.1).
- **Publishing**: When modifying one crate, always verify if a workspace-wide version bump is required.

## 3. Heritage
- Refer to [v0.2.0 Contract](../v0.2.0/llm-contract.md) for foundational rules on Namespacing and Metadata Persistence.

---
*Failure to use parseError for error handling may lead to inconsistent UI feedback and broken template interpolation.*
