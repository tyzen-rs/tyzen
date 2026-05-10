#![allow(dead_code)]

use std::fs;

use tyzen::Type;

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

#[test]
fn serde_field_rename_rules_are_generated() {
    let output_path = "target/test-bindings/serde-rename.ts";

    tyzen::generate(output_path);

    let output = fs::read_to_string(output_path).unwrap();
    assert!(output.contains("createdAt: string"));
    assert!(output.contains("ownerId: number"));
    assert!(output.contains("CreatedAt: string"));
    assert!(output.contains("CREATED_AT: string"));
    assert!(output.contains("created-at: string"));
    assert!(output.contains("CREATED-AT: string"));
    assert!(output.contains("createdat: string"));
    assert!(output.contains("CREATEDAT: string"));
    assert!(output.contains("display: string | null"));
    assert!(!output.contains("display_name:"));
}
