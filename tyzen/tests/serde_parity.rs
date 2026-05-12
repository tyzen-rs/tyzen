#![allow(dead_code)]

use std::fs;
use tyzen::Type;

#[derive(Type)]
struct Child {
    c: i32,
}

#[derive(Type)]
struct Parent {
    #[serde(flatten)]
    child: Child,
    p: String,
}

#[derive(Type)]
struct DefaultFields {
    #[serde(default)]
    a: i32,
    #[serde(default = "default_b")]
    b: String,
}

fn default_b() -> String {
    "default".to_string()
}

#[derive(Type)]
struct AliasField {
    #[serde(alias = "other")]
    name: String,
}

#[test]
fn test_serde_parity() {
    let output_path = "target/test-bindings/serde-parity.ts";
    tyzen::generate(output_path);

    let output = fs::read_to_string(output_path).expect("Failed to read generated bindings");

    // Flattening
    assert!(
        output.contains("export type Parent = { c: number, p: string }")
            || output.contains("export type Parent = { p: string, c: number }"),
        "Flattening failed: expected fields 'c' and 'p' in Parent. Output: {}",
        output
    );

    // Default
    assert!(
        output.contains("a?: number"),
        "Default field 'a' should be optional"
    );
    assert!(
        output.contains("b?: string"),
        "Default field 'b' should be optional"
    );

    assert!(
        output.contains("name: string"),
        "Alias field 'name' should use canonical name"
    );
}
