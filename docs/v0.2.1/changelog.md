# Changelog v0.2.1

> [!TIP]
> This is a maintenance release addressing feedback from the v0.2.0 rollout, specifically focusing on error handling and reliability.

## 🐛 Bug Fixes & Improvements

### 🛡️ Parse Error Integration
- **`parseError` Utility**: Added a new TypeScript utility to automatically map discriminated union errors back to their original metadata.
- **Template Interpolation**: `parseError` now supports dynamic interpolation for messages (e.g., `"Error: {value}"` becomes `"Error: 404"`).

### 🛠️ Internal Reliability
- **Diagnostic Pipeline**: Integrated `parseError` into the macro diagnostic pipeline to provide clearer error messages during build time.
- **Dependency Sync**: Bumped all internal workspace dependencies to v0.2.1 to ensure atomic updates.

---
*For a full list of features introduced in the previous major version, see [v0.2.0](../v0.2.0/changelog.md).*
