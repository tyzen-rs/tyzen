#![allow(dead_code)]

use std::fs;

use tyzen::Type;

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

#[test]
fn serde_skipped_fields_and_variants_are_omitted() {
    let output_path = "target/test-bindings/serde-skip.ts";

    tyzen::generate(output_path);

    let output = fs::read_to_string(output_path).unwrap();
    assert!(output.contains("export type SkippedFields = { kept: string }"));
    assert!(!output.contains("skipped:"));
    assert!(!output.contains("skipped_serializing:"));
    assert!(!output.contains("skipped_deserializing:"));
    assert!(output.contains("\"Kept\""));
    assert!(!output.contains("\"Skipped\""));
    assert!(output.contains("{ tag: \"WithFields\", kept: string }"));
}
