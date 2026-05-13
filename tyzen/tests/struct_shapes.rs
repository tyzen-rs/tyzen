#![allow(dead_code)]

use std::fs;

use tyzen::Type;

#[derive(Type)]
struct NamedStruct {
    id: u32,
    name: String,
}

#[derive(Type)]
struct NewtypeId(String);

#[derive(Type)]
struct Pair(u32, String);

#[derive(Type)]
struct Marker;

#[test]
fn struct_shapes_are_generated() {
    let output_path = "target/test-bindings/struct-shapes.ts";

    tyzen::generate(output_path).expect("generate should succeed");

    let output = fs::read_to_string(output_path).unwrap();
    assert!(output.contains("export type NamedStruct = { id: number, name: string }"));
    assert!(output.contains("export type NewtypeId = string"));
    assert!(output.contains("export type Pair = [number, string]"));
    assert!(output.contains("export type Marker = null"));
}
