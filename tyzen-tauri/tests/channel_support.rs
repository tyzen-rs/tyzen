#![allow(dead_code)]

use std::fs;

#[tyzen::command]
pub fn stream_logs(on_log: tauri::ipc::Channel<String>) -> String {
    let _ = on_log;
    String::new()
}

#[test]
fn channel_params_generate_channel_wrappers() {
    let output_path = "target/test-bindings/tauri-channel-support.ts";
    let _ = fs::remove_file(output_path);

    tyzen_tauri::generate(output_path).expect("generate should succeed");

    let output = fs::read_to_string(output_path).expect("Failed to read generated bindings");

    assert!(output.contains("import { Channel } from \"@tauri-apps/api/core\""));
    assert!(output.contains(
        "streamLogs: (onLog: (payload: string) => void) => __invoke<string>(\"stream_logs\", { onLog: new Channel(onLog) })"
    ));
}
