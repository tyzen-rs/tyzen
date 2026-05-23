#![allow(dead_code)]

use std::fs;

mod tauri {
    pub mod ipc {
        #[allow(dead_code)]
        pub struct Channel<T>(pub T);
    }
}

#[tyzen::command]
pub fn stream_logs(on_log: tauri::ipc::Channel<String>) -> String {
    let _ = on_log;
    String::new()
}

#[test]
fn channel_params_generate_callback_types_in_declarations() {
    let output_path = "target/test-bindings/channel-support.ts";
    let _ = fs::remove_file(output_path);

    tyzen::generate(output_path).expect("generate should succeed");

    let output = fs::read_to_string(output_path).expect("Failed to read generated bindings");

    assert!(output.contains("streamLogs(onLog: (payload: string) => void): Promise<string>"));
    assert!(!output.contains("Channel<string>"));
}
