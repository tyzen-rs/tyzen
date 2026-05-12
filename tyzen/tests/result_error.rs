#![allow(dead_code)]

use serde::Serialize;
use tyzen::TsType;

#[derive(tyzen::Type, Serialize)]
struct LocalResult<T> {
    data: T,
}

#[derive(tyzen::Type, Serialize)]
enum MyError {
    NotFound,
    Unauthorized(String),
}

type MyResult<T> = std::result::Result<T, MyError>;
type SimpleResult<T> = std::result::Result<T, String>;

#[test]
fn test_result_mapping() {
    assert_eq!(
        std::result::Result::<u32, String>::ts_name(),
        "Result<number, string>"
    );
    assert_eq!(
        std::result::Result::<u32, MyError>::ts_name(),
        "Result<number, MyError>"
    );
}

#[test]
fn test_alias_result_mapping() {
    assert_eq!(MyResult::<u32>::ts_name(), "Result<number, MyError>");
    assert_eq!(SimpleResult::<u32>::ts_name(), "Result<number, string>");
}

#[derive(tyzen::Type)]
struct Container {
    res1: std::result::Result<u32, String>,
    res2: std::result::Result<u32, MyError>,
    res3: std::result::Result<u32, Vec<String>>,
    res4: LocalResult<u32>,
}

#[derive(tyzen::Type)]
struct GenericContainer<T> {
    res: LocalResult<T>,
}

#[test]
fn test_single_arg_result_like() {
    assert_eq!(
        GenericContainer::<u32>::ts_name(),
        "GenericContainer<number>"
    );
}

#[test]
fn test_struct_with_results() {
    assert_eq!(Container::ts_name(), "Container");
}

#[test]
fn test_generation() {
    let output_path = "target/test-bindings/result-error.ts";
    tyzen::generate(output_path);
    let output = std::fs::read_to_string(output_path).unwrap();

    assert!(output.contains(
        "export type Result<T, E = string> = { status: \"ok\"; data: T } | { status: \"error\"; error: E }"
    ));

    assert!(output.contains(
        "export type MyError = { tag: \"NotFound\" } | { tag: \"Unauthorized\", value: string }"
    ));

    assert!(output.contains(
        "export type Container = { res1: Result<number, string>, res2: Result<number, MyError>, res3: Result<number, string[]>, res4: LocalResult<number> }"
    ));

    assert!(output.contains("export type LocalResult<T> = { data: T }"));
}

#[cfg(feature = "anyhow")]
#[test]
fn test_anyhow_support() {
    assert_eq!(
        std::result::Result::<u32, anyhow::Error>::ts_name(),
        "Result<number, string>"
    );
}
