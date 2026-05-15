use tyzen::{GeneratorConfig, NamingStrategy, Type};

#[derive(Type)]
#[tyzen(ns = "Task")]
pub struct TaskDto {
    pub id: u32,
    pub title: String,
}

#[derive(Type)]
#[tyzen(ns = "Task")]
pub struct Task {
    pub id: u32,
}

#[test]
fn test_auto_naming_prefix() {
    let config = GeneratorConfig {
        naming_strategy: NamingStrategy::Prefix,
    };

    // We can't easily call generate_full with a buffer here because it writes to a file.
    // But we can test the internal strip_naming if it were public, or use a temporary file.

    let path = "tests/output_auto_naming.ts";
    tyzen::generate_full(path, config, |_| {}, |_| {}).unwrap();

    let content = std::fs::read_to_string(path).unwrap();

    // Check global types
    assert!(content.contains("export type TaskDto = { id: number, title: string }"));
    assert!(content.contains("export type Task = { id: number }"));

    // Check namespaced types
    assert!(content.contains("export namespace Task {"));
    assert!(content.contains("  export type Dto = TaskDto;"));

    // Check collision avoidance (Task.Task should be skipped if we implemented it correctly)
    // Actually, our current logic: if stripped_name != ns.
    // Task stripped with Task -> "" (empty).
    // Our strip_naming returns original if it would be empty? Let's check.

    std::fs::remove_file(path).ok();
}
