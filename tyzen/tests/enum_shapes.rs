#![allow(dead_code)]

use std::fs;

use tyzen::Type;

#[derive(Type)]
enum PayloadEnum {
    Unit,
    Single(String),
    Multi(u32, String, bool),
    Named { created_at: String, owner_id: u32 },
}

#[derive(Type)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
enum RenamedEnum {
    UnitVariant,
    #[serde(rename = "renamedTuple")]
    TupleVariant(u32, String),
    NamedVariant {
        created_at: String,
        #[serde(rename = "owner")]
        owner_id: u32,
    },
}

#[derive(Type)]
#[serde(rename_all_fields = "camelCase")]
enum VariantFieldRenameOverride {
    #[serde(rename_all = "kebab-case")]
    NamedVariant { created_at: String },
}

#[test]
fn enum_shapes_and_serde_names_are_generated() {
    let output_path = "target/test-bindings/enum-shapes.ts";

    tyzen::generate(output_path);

    let output = fs::read_to_string(output_path).unwrap();
    assert!(output.contains("\"Unit\""));
    assert!(output.contains("{ tag: \"Single\", value: string }"));
    assert!(output.contains("{ tag: \"Multi\", value: [number, string, boolean] }"));
    assert!(output.contains("{ tag: \"Named\", created_at: string, owner_id: number }"));
    assert!(output.contains("\"unitVariant\""));
    assert!(output.contains("{ tag: \"renamedTuple\", value: [number, string] }"));
    assert!(output.contains("{ tag: \"namedVariant\", createdAt: string, owner: number }"));
    assert!(output.contains("{ tag: \"NamedVariant\", created-at: string }"));
}
