#![allow(dead_code)]

use std::fs;

use tyzen::Type;

#[derive(Type)]
struct OptionalFieldCase {
    required_nullable: Option<String>,
    #[tyzen(optional)]
    optional_string: Option<String>,
    #[tyzen(optional)]
    optional_number: Option<u32>,
}

#[test]
fn selected_fields_can_generate_typescript_optional_properties() {
    let output_path = "target/test-bindings/optional-fields.ts";

    tyzen::generate(output_path);

    let output = fs::read_to_string(output_path).unwrap();
    assert!(output.contains("required_nullable: string | null"));
    assert!(output.contains("optional_string?: string"));
    assert!(output.contains("optional_number?: number"));
}
