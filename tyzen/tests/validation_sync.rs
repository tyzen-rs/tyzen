#[allow(dead_code)]

use std::fs;
use tyzen::Type;

// Mock validator crate elements for testing purposes
// This ensures we can compile without depending on the heavy validator crate in the tyzen library itself.
// When using the macro, syn just parses the attribute tokens, so these definitions are perfectly clean and compatible!
#[derive(Type)]
#[tyzen(schema)]
struct SimpleUser {
    #[validate(length(min = 1, message = "Title cannot be empty"))]
    pub title: String,

    #[validate(length(min = 2, max = 5, message = "Code must be between 2 and 5 characters"))]
    #[validate(regex = "^[A-Z0-9]+$")]
    pub code: String,

    #[validate(range(min = 10, max = 100, message = "Age must be between 10 and 100"))]
    pub age: u32,

    pub is_active: bool,
}

#[derive(Type)]
struct UnactivatedStruct {
    pub value: String,
}

#[derive(Type)]
pub enum SimplePriority {
    Low,
    Medium,
    High,
}

#[derive(Type)]
#[tyzen(schema)]
struct ComplexUser {
    pub priority: SimplePriority,
    
    pub target_date: Option<String>,
    
    #[tyzen(optional)]
    pub description: Option<String>,
}

#[test]
fn validation_sync_generates_correct_zod_schemas() {
    let output_path = "target/test-bindings/validation-sync.ts";

    tyzen::generate(output_path).expect("generate should succeed");

    let output = fs::read_to_string(output_path).unwrap();

    // 1. Zod Import injected
    assert!(output.contains("import { z } from \"zod\";"));

    // 2. SimpleUser schema matching
    assert!(output.contains("export const simpleUserSchema = z.object({"));
    assert!(output.contains("title: z.string().min(1, { message: \"Title cannot be empty\" })"));
    assert!(output.contains("code: z.string().min(2, { message: \"Code must be between 2 and 5 characters\" }).max(5, { message: \"Code must be between 2 and 5 characters\" }).regex(/^[A-Z0-9]+$/, { message: \"Code must be between 2 and 5 characters\" })"));
    assert!(output.contains("age: z.number().min(10, { message: \"Age must be between 10 and 100\" }).max(100, { message: \"Age must be between 10 and 100\" })"));
    assert!(output.contains("is_active: z.boolean()"));

    // 3. Negative space: UnactivatedStruct should not generate a schema
    assert!(!output.contains("unactivatedStructSchema"));

    // 4. ComplexUser schema matching with nested unit enums and optionals
    assert!(output.contains("export const complexUserSchema = z.object({"));
    assert!(output.contains("priority: z.enum([\"Low\", \"Medium\", \"High\"])"));
    assert!(output.contains("target_date: z.string().nullable().optional()"));
    assert!(output.contains("description: z.string().optional()"));
}
