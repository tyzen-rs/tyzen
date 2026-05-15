#[allow(dead_code)]
use serde::Serialize;
use std::fs;
use tyzen::Type;

#[derive(Type, Serialize)]
#[tyzen(ns = "Task")]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub owner: User,
}

#[derive(Type, Serialize)]
#[tyzen(ns = "Auth")]
pub struct User {
    pub id: u32,
    pub name: String,
}

#[derive(Type, Serialize)]
#[tyzen(ns = "Task")]
pub struct TaskUpdate {
    pub title: Option<String>,
}

#[tyzen::command(ns = "Task")]
pub fn get_task(_id: u32) -> Task {
    unimplemented!()
}

#[test]
fn test_namespaces_generation() {
    let output_path = "target/test-bindings/namespaces.ts";
    let _ = fs::remove_file(output_path);

    // Provide __invoke and __listen as helpers in the test
    tyzen::generate_full(
        output_path,
        tyzen::GeneratorConfig::default(),
        |ts| {
            ts.push_str(
                "const __invoke = <T>(name: string, args: any) => Promise.resolve({} as any);\n",
            );
            ts.push_str("const __listen = (name: string, cb: any) => {};\n");
        },
        |_| {},
    )
    .expect("Failed to generate bindings");

    let output = fs::read_to_string(output_path).expect("Failed to read generated bindings");

    // Check Task namespace object (Model-Centric)
    assert!(output.contains("export const Task = {"));
    assert!(output.contains("getTask: (id: number) => __invoke<Task>(\"get_task\", { id })"));

    // Check global types
    assert!(output.contains("export type Task = {"));
    assert!(output.contains("owner: User")); // Global reference, no Auth. prefix
    assert!(output.contains("export type User = {"));

    // Check that they are properly ordered by checking the position of namespace markers
    assert!(output.find("Namespace: Auth").unwrap() < output.find("Namespace: Task").unwrap());
}
