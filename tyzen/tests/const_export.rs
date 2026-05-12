use tyzen::export;
use std::fs;

#[export]
pub const VERSION: &str = "1.0.0";

#[export]
pub const MAX_RETRIES: u32 = 5;

#[export]
pub const IS_DEBUG: bool = true;

#[test]
fn test_const_export() {
    let output_path = "target/test-bindings/consts.ts";
    tyzen::generate(output_path);

    let content = fs::read_to_string(output_path).expect("failed to read output file");
    
    assert!(content.contains("export const VERSION = \"1.0.0\" as const;"));
    assert!(content.contains("export const MAX_RETRIES = 5 as const;"));
    assert!(content.contains("export const IS_DEBUG = true as const;"));
}
