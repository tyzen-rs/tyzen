#![allow(dead_code)]

use std::fs;
use tyzen::Type;

#[derive(Type)]
#[tyzen(schema)]
struct GlobalOptionalCase {
    required_nullable: Option<String>,
    regular_string: String,
    
    #[tyzen(nullable)]
    explicitly_nullable: Option<String>,
}

#[test]
fn global_optional_generator_option_works() {
    let output_path = "target/test-bindings/global-optional.ts";

    tyzen::Generator::new(output_path)
        .option_fields_as_optional()
        .run()
        .expect("generate should succeed");

    let output = fs::read_to_string(output_path).unwrap();
    
    // Check standard Option mapping when config is active
    assert!(output.contains("required_nullable?: string"));
    assert!(output.contains("regular_string: string"));
    assert!(output.contains("required_nullable: z.string().optional()"));

    // Check that explicit #[tyzen(nullable)] overrides are honored (retains | null)
    assert!(output.contains("explicitly_nullable: string | null"));
    assert!(output.contains("explicitly_nullable: z.string().nullable().optional()"));
}
