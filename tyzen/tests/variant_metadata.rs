#![allow(dead_code)]
use std::fs;
use tyzen::Type;

#[derive(Type)]
pub enum ErrorCode {
    #[tyzen(code = "404", msg = "Not Found")]
    NotFound,
    #[tyzen(code = "500", msg = "Internal Error")]
    Internal,
    #[tyzen(code = "403")]
    Forbidden,
    NoMeta,
}

#[derive(Type)]
#[serde(rename_all = "camelCase")]
pub enum RenamedMeta {
    #[tyzen(code = "1", error_message = "fail")]
    #[serde(rename = "custom_rename")]
    MyVariant,
}

#[derive(Type)]
pub enum GenericMeta<T> {
    #[tyzen(label = "Generic")]
    Value(T),
}

#[derive(Type)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Canceled,
}

#[derive(Type)]
pub enum ProjectStatus {
    #[tyzen(task_statuses(TaskStatus::Todo, TaskStatus::InProgress, TaskStatus::Done, TaskStatus::Canceled))]
    Active,
    #[tyzen(task_statuses(TaskStatus::Todo, TaskStatus::Canceled))]
    OnHold,
}

#[test]
fn variant_metadata_is_generated() {
    let output_path = "target/test-bindings/variant-metadata.ts";
    tyzen::generate(output_path).expect("generate should succeed");
    
    let output = fs::read_to_string(output_path).unwrap();

    // 1. Basic Metadata
    assert!(output.contains("export const ErrorCodeMeta = {"));
    assert!(output.contains("NotFound: { code: \"404\", msg: \"Not Found\" }"));
    assert!(output.contains("NoMeta: {}"));

    // 2. Renamed Variants (Meta key should use the TS variant name for consistency)
    // and keys should be camelCase
    assert!(output.contains("export const RenamedMetaMeta = {"));
    assert!(output.contains("custom_rename: { code: \"1\", errorMessage: \"fail\" }"));

    // 3. Generic Enums
    assert!(output.contains("export const GenericMetaMeta = {"));
    assert!(output.contains("Value: { label: \"Generic\" }"));

    // 4. Type-safe Path Lists
    assert!(output.contains("export const ProjectStatusMeta = {"));
    assert!(output.contains("Active: { taskStatuses: [TaskStatus.Todo, TaskStatus.InProgress, TaskStatus.Done, TaskStatus.Canceled] }"));
    assert!(output.contains("OnHold: { taskStatuses: [TaskStatus.Todo, TaskStatus.Canceled] }"));
}
