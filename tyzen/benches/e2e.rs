use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const WARMUP_RUNS: usize = 5;
const MEASURED_RUNS: usize = 20;

#[derive(tyzen::Type, Serialize, Deserialize, Clone)]
pub struct UserProfile {
    pub display_name: String,
    #[serde(default)]
    pub bio: String,
    pub avatar_url: Option<String>,
}

#[derive(tyzen::Type, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: u64,
    pub email: String,
    #[serde(flatten)]
    pub profile: UserProfile,
}

#[derive(tyzen::Type, Serialize, Deserialize, Clone)]
#[serde(tag = "kind", content = "data")]
pub enum AuthEventPayload {
    LoggedIn(User),
    LoggedOut,
    TokenRefreshed { expires_in_secs: u64 },
}

mod auth {
    tyzen::module_ns!("Auth");
    use super::*;

    #[derive(tyzen::Type, Serialize, Deserialize, Clone)]
    pub struct LoginRequest {
        pub email: String,
        pub password: String,
    }

    #[derive(tyzen::Type, Serialize, Deserialize, Clone)]
    pub struct LoginResponse {
        pub token: String,
        pub user: User,
    }

    #[tyzen::command]
    pub fn auth_login(_input: LoginRequest) -> Result<LoginResponse, String> {
        Err("not implemented".to_string())
    }

    #[tyzen::command]
    pub fn auth_me() -> Result<User, String> {
        Err("not implemented".to_string())
    }
}

mod task {
    tyzen::module_ns!("Task");
    use super::*;

    #[derive(tyzen::Type, Serialize, Deserialize, Clone)]
    pub struct TaskMeta {
        pub priority: u8,
        #[serde(default)]
        pub archived: bool,
    }

    #[derive(tyzen::Type, Serialize, Deserialize, Clone)]
    pub struct Task {
        pub id: u64,
        #[serde(rename = "createdAt")]
        pub created_at: String,
        pub title: String,
        pub owner: User,
        #[serde(flatten)]
        pub meta: TaskMeta,
    }

    #[derive(tyzen::Type, Serialize, Deserialize, Clone)]
    pub struct CreateTaskInput {
        pub title: String,
        pub owner_id: u64,
        pub priority: Option<u8>,
    }

    #[derive(tyzen::Type, Serialize, Deserialize, Clone)]
    #[serde(tag = "status", content = "payload")]
    pub enum TaskSyncState {
        Idle,
        Syncing { queued: usize },
        Failed { message: String },
    }

    #[tyzen::command]
    pub fn task_create(_input: CreateTaskInput) -> Result<Task, String> {
        Err("not implemented".to_string())
    }

    #[tyzen::command]
    pub fn task_list(_owner_id: u64) -> Result<Vec<Task>, String> {
        Err("not implemented".to_string())
    }
}

mod billing {
    tyzen::module_ns!("Billing");
    use super::*;

    #[derive(tyzen::Type, Serialize, Deserialize, Clone)]
    pub struct Money {
        pub currency: String,
        pub amount_cents: u64,
    }

    #[derive(tyzen::Type, Serialize, Deserialize, Clone)]
    pub struct Plan {
        pub code: String,
        pub name: String,
        pub price: Money,
    }

    #[derive(tyzen::Type, Serialize, Deserialize, Clone)]
    pub struct Subscription {
        pub id: String,
        pub user_id: u64,
        pub plan: Plan,
        pub active: bool,
    }

    #[tyzen::command]
    pub fn billing_plans() -> Result<Vec<Plan>, String> {
        Err("not implemented".to_string())
    }

    #[tyzen::command]
    pub fn billing_subscription(_user_id: u64) -> Result<Subscription, String> {
        Err("not implemented".to_string())
    }
}

fn temp_output_path() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!(
        "target/bench-bindings/e2e-{}-{}.ts",
        std::process::id(),
        nanos
    )
}

fn median_ms(samples: &mut [Duration]) -> f64 {
    samples.sort_unstable();
    let mid = samples.len() / 2;
    samples[mid].as_secs_f64() * 1000.0
}

fn p95_ms(samples: &mut [Duration]) -> f64 {
    samples.sort_unstable();
    let idx = ((samples.len() as f64) * 0.95).ceil() as usize - 1;
    samples[idx].as_secs_f64() * 1000.0
}

fn write_results_json(median: f64, p95: f64, output_size_bytes: usize) -> std::io::Result<()> {
    std::fs::create_dir_all("bench/results")?;
    let json = format!(
        "{{\n  \"warmup_runs\": {},\n  \"measured_runs\": {},\n  \"benchmark\": \"e2e-generate\",\n  \"total_ms_median\": {:.3},\n  \"total_ms_p95\": {:.3},\n  \"output_size_bytes\": {}\n}}\n",
        WARMUP_RUNS, MEASURED_RUNS, median, p95, output_size_bytes
    );
    std::fs::write("bench/results/e2e-latest.json", json)
}

fn main() {
    let mut samples = Vec::with_capacity(MEASURED_RUNS);
    let mut last_output_size = 0usize;

    for _ in 0..WARMUP_RUNS {
        let path = temp_output_path();
        let _ = tyzen::generate(&path);
        let _ = std::fs::remove_file(path);
    }

    for _ in 0..MEASURED_RUNS {
        let path = temp_output_path();
        let start = Instant::now();
        tyzen::generate(&path).expect("failed to generate bindings");
        let elapsed = start.elapsed();

        let metadata = std::fs::metadata(&path).expect("failed to read generated file metadata");
        last_output_size = metadata.len() as usize;
        std::fs::remove_file(path).expect("failed to cleanup generated bindings");

        samples.push(elapsed);
    }

    let mut median_samples = samples.clone();
    let mut p95_samples = samples;

    let median = median_ms(&mut median_samples);
    let p95 = p95_ms(&mut p95_samples);

    write_results_json(median, p95, last_output_size).expect("failed to write e2e benchmark json");

    println!(
        "benchmark=e2e-generate total_median_ms={:.3} total_p95_ms={:.3} output_size_bytes={}",
        median, p95, last_output_size
    );
}
