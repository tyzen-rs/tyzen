#![allow(dead_code)]

use std::fs;
use tyzen::Type;

// --- Rename Tests ---

#[derive(Type)]
#[serde(rename_all = "camelCase")]
struct CamelCaseFields {
    created_at: String,
    owner_id: u32,
}

#[derive(Type)]
#[serde(rename_all = "PascalCase")]
struct PascalCaseFields {
    created_at: String,
}

#[derive(Type)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct ScreamingSnakeCaseFields {
    created_at: String,
}

#[derive(Type)]
#[serde(rename_all = "kebab-case")]
struct KebabCaseFields {
    created_at: String,
}

#[derive(Type)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
struct ScreamingKebabCaseFields {
    created_at: String,
}

#[derive(Type)]
#[serde(rename_all = "lowercase")]
struct LowercaseFields {
    created_at: String,
}

#[derive(Type)]
#[serde(rename_all = "UPPERCASE")]
struct UppercaseFields {
    created_at: String,
}

#[derive(Type)]
struct FieldRenameCase {
    #[serde(rename = "display")]
    display_name: Option<String>,
}

// --- Skip Tests ---

#[derive(Type)]
struct SkippedFields {
    kept: String,
    #[serde(skip)]
    skipped: String,
    #[serde(skip_serializing)]
    skipped_serializing: String,
    #[serde(skip_deserializing)]
    skipped_deserializing: String,
}

#[derive(Type)]
enum SkippedVariants {
    Kept,
    #[serde(skip)]
    Skipped,
    WithFields {
        kept: String,
        #[serde(skip)]
        skipped: String,
    },
}

// --- Tagging Tests ---

#[derive(Type)]
#[serde(tag = "type", content = "data")]
enum AdjacentlyTagged {
    NotFound { id: u32 },
    Validation(String),
    Database(String),
    Unit,
}

#[derive(Type)]
#[serde(tag = "kind")]
enum InternallyTagged {
    A { x: u32 },
    B { y: String },
    C,
}

#[derive(Type)]
#[serde(untagged)]
enum Untagged {
    Number(u32),
    String(String),
    Struct { a: u32 },
    Unit,
}

#[derive(Type)]
#[serde(transparent)]
struct TransparentStruct(u32);

#[derive(Type)]
#[serde(transparent)]
struct TransparentNamed {
    inner: String,
}

#[test]
fn serde_attributes_are_obeyed() {
    let output_path = "target/test-bindings/serde-all.ts";

    tyzen::generate(output_path);

    let output = fs::read_to_string(output_path).unwrap();

    // Renames
    assert!(output.contains("createdAt: string"));
    assert!(output.contains("ownerId: number"));
    assert!(output.contains("CreatedAt: string"));
    assert!(output.contains("CREATED_AT: string"));
    assert!(output.contains("created-at: string"));
    assert!(output.contains("CREATED-AT: string"));
    assert!(output.contains("createdat: string"));
    assert!(output.contains("CREATEDAT: string"));
    assert!(output.contains("display: string | null"));

    // Skips
    assert!(output.contains("export type SkippedFields = { kept: string }"));
    assert!(!output.contains("skipped:"));
    assert!(!output.contains("skipped_serializing:"));
    assert!(!output.contains("skipped_deserializing:"));
    assert!(output.contains("\"Kept\""));
    assert!(!output.contains("\"Skipped\""));
    assert!(output.contains("{ tag: \"WithFields\", kept: string }"));

    // Tagging
    assert!(output.contains("export type AdjacentlyTagged = { type: \"NotFound\", data: { id: number } } | { type: \"Validation\", data: string } | { type: \"Database\", data: string } | { type: \"Unit\" }"));
    assert!(output.contains("export type InternallyTagged = { kind: \"A\", x: number } | { kind: \"B\", y: string } | { kind: \"C\" }"));
    assert!(output.contains("export type Untagged = number | string | { a: number } | null"));

    // Transparent
    assert!(output.contains("export type TransparentStruct = number"));
    assert!(output.contains("export type TransparentNamed = string"));
}
