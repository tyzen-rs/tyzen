#[allow(dead_code)]
use std::collections::HashMap;
use std::fs;
use tyzen::Type;

// ─── Existing types ───────────────────────────────────────────────────────────

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

// ─── Enum schema types ────────────────────────────────────────────────────────

/// Internally tagged enum → z.discriminatedUnion
#[derive(Type)]
#[tyzen(schema)]
#[serde(tag = "type")]
enum StatusMessage {
    Success { message: String },
    Error { code: u32, message: String },
}

/// Untagged enum → z.union
#[derive(Type)]
#[tyzen(schema)]
#[serde(untagged)]
enum StringOrNumber {
    Str(String),
    Num(i64),
}

/// Adjacently tagged enum → z.discriminatedUnion with content wrapper
#[derive(Type)]
#[tyzen(schema)]
#[serde(tag = "kind", content = "data")]
enum AdjacentPayload {
    Ping,
    Pong { count: u32 },
}

/// External (default) tagged enum with payload → z.union of z.object wrappers
#[derive(Type)]
#[tyzen(schema)]
enum ExternalTagged {
    Text(String),
    Count(u32),
}

// ─── Structural types ─────────────────────────────────────────────────────────

/// Transparent newtype → delegates to inner type schema
#[derive(Type)]
#[tyzen(schema)]
#[serde(transparent)]
struct UserId(String);

/// Tuple struct → z.tuple([...])
#[derive(Type)]
#[tyzen(schema)]
struct Point(f64, f64);

/// HashMap field → z.record(...)
#[derive(Type)]
#[tyzen(schema)]
struct Config {
    pub settings: HashMap<String, String>,
}

// ─── Extra validator types ────────────────────────────────────────────────────

#[derive(Type)]
#[tyzen(schema)]
struct ContactInfo {
    #[validate(email)]
    pub email: String,

    #[validate(url)]
    pub website: String,
}

#[derive(Type)]
#[tyzen(schema)]
struct FilterConfig {
    #[validate(contains = "@")]
    pub required_char: String,

    #[validate(does_not_contain = "admin")]
    pub safe_name: String,
}

#[derive(Type)]
#[tyzen(schema)]
struct CustomAndPerFieldUser {
    #[validate(custom = "validate_username")]
    pub username: String,

    #[validate(custom(function = "validate_nickname", message = "Invalid nickname format"))]
    pub nickname: String,

    #[validate(email(message = "Not a valid email address"))]
    pub custom_email: String,

    #[validate(contains(pattern = "#", message = "Must contain hashtag"))]
    pub hashtag: String,
}

// ─── Tests ────────────────────────────────────────────────────────────────────

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
    assert!(output.contains("code: z.string().min(2, { message: \"Code must be between 2 and 5 characters\" }).max(5, { message: \"Code must be between 2 and 5 characters\" }).regex(/^[A-Z0-9]+$/)"));
    assert!(output.contains("age: z.number().min(10, { message: \"Age must be between 10 and 100\" }).max(100, { message: \"Age must be between 10 and 100\" })"));
    assert!(output.contains("is_active: z.boolean()"));

    // 3. Negative space: UnactivatedStruct should not generate a schema
    assert!(!output.contains("unactivatedStructSchema"));

    // 4. ComplexUser schema matching with nested unit enums and optionals
    assert!(output.contains("export const complexUserSchema = z.object({"));
    assert!(output.contains("priority: z.enum([\"Low\", \"Medium\", \"High\"])"));
    assert!(output.contains("target_date: z.union([z.string(), z.date()]).nullable().optional()"));
    assert!(output.contains("description: z.string().optional()"));
}

#[test]
fn zod_tagged_enum_generates_discriminated_union() {
    let output_path = "target/test-bindings/validation-sync-enums.ts";
    tyzen::generate(output_path).expect("generate should succeed");
    let output = fs::read_to_string(output_path).unwrap();

    // Internally tagged → z.discriminatedUnion("type", [...])
    assert!(
        output.contains("z.discriminatedUnion(\"type\","),
        "Expected discriminatedUnion for StatusMessage"
    );
    assert!(output.contains("z.literal(\"Success\")"));
    assert!(output.contains("z.literal(\"Error\")"));
}

#[test]
fn zod_untagged_enum_generates_union() {
    let output_path = "target/test-bindings/validation-sync-untagged.ts";
    tyzen::generate(output_path).expect("generate should succeed");
    let output = fs::read_to_string(output_path).unwrap();

    // Untagged → z.union([z.string(), z.number()])
    let schema_line = output
        .lines()
        .find(|l| l.contains("stringOrNumberSchema"))
        .unwrap_or("");
    assert!(
        schema_line.contains("z.union("),
        "Expected z.union for StringOrNumber, got: {}",
        schema_line
    );
    assert!(schema_line.contains("z.string()"));
    assert!(schema_line.contains("z.number()"));
}

#[test]
fn zod_adjacent_tagged_enum_generates_discriminated_union_with_content() {
    let output_path = "target/test-bindings/validation-sync-adjacent.ts";
    tyzen::generate(output_path).expect("generate should succeed");
    let output = fs::read_to_string(output_path).unwrap();

    // Adjacently tagged → z.discriminatedUnion("kind", [...])
    assert!(
        output.contains("z.discriminatedUnion(\"kind\","),
        "Expected discriminatedUnion for AdjacentPayload"
    );
    // Ping unit variant: just the tag field
    assert!(output.contains("z.literal(\"Ping\")"));
    // Pong named variant: wrapped in content key
    assert!(output.contains("data: z.object("));
}

#[test]
fn zod_transparent_newtype_generates_inner_schema() {
    let output_path = "target/test-bindings/validation-sync-transparent.ts";
    tyzen::generate(output_path).expect("generate should succeed");
    let output = fs::read_to_string(output_path).unwrap();

    // Transparent String wrapper → z.string()
    assert!(
        output.contains("export const userIdSchema = z.string();"),
        "Expected userIdSchema = z.string()"
    );
}

#[test]
fn zod_tuple_struct_generates_z_tuple() {
    let output_path = "target/test-bindings/validation-sync-tuple.ts";
    tyzen::generate(output_path).expect("generate should succeed");
    let output = fs::read_to_string(output_path).unwrap();

    // Point(f64, f64) → z.tuple([z.number(), z.number()])
    assert!(
        output.contains("export const pointSchema = z.tuple([z.number(), z.number()]);"),
        "Expected pointSchema = z.tuple"
    );
}

#[test]
fn zod_hashmap_field_generates_z_record() {
    let output_path = "target/test-bindings/validation-sync-record.ts";
    tyzen::generate(output_path).expect("generate should succeed");
    let output = fs::read_to_string(output_path).unwrap();

    // HashMap<String, String> → z.record(z.string(), z.string())
    assert!(
        output.contains("settings: z.record(z.string(), z.string())"),
        "Expected settings: z.record(z.string(), z.string())"
    );
}

#[test]
fn zod_email_url_validators() {
    let output_path = "target/test-bindings/validation-sync-emailurl.ts";
    tyzen::generate(output_path).expect("generate should succeed");
    let output = fs::read_to_string(output_path).unwrap();

    assert!(
        output.contains("email: z.string().email()"),
        "Expected .email() validator"
    );
    assert!(
        output.contains("website: z.string().url()"),
        "Expected .url() validator"
    );
}

#[test]
fn zod_contains_does_not_contain_validators() {
    let output_path = "target/test-bindings/validation-sync-contains.ts";
    tyzen::generate(output_path).expect("generate should succeed");
    let output = fs::read_to_string(output_path).unwrap();

    assert!(
        output.contains("required_char: z.string().includes(\"@\")"),
        "Expected .includes('@') for contains validator"
    );
    assert!(
        output.contains("safe_name: z.string().refine(v => !v.includes(\"admin\"),"),
        "Expected .refine for does_not_contain validator"
    );
}

#[test]
fn zod_custom_and_per_field_messages_validators() {
    let output_path = "target/test-bindings/validation-sync-custom.ts";
    tyzen::generate(output_path).expect("generate should succeed");
    let output = fs::read_to_string(output_path).unwrap();

    assert!(
        output.contains("username: z.string().refine(() => true, { message: \"validate with validate_username\" })"),
        "Expected refine no-op for custom function validate_username"
    );
    assert!(
        output.contains("nickname: z.string().refine(() => true, { message: \"Invalid nickname format\" })"),
        "Expected refine no-op for custom function validate_nickname with custom message"
    );
    assert!(
        output.contains("custom_email: z.string().email({ message: \"Not a valid email address\" })"),
        "Expected custom error message for email"
    );
    assert!(
        output.contains("hashtag: z.string().includes(\"#\", { message: \"Must contain hashtag\" })"),
        "Expected custom error message for includes (contains)"
    );
}

