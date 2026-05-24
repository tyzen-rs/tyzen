#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(tyzen::Type, Serialize, Deserialize, Clone)]
pub struct SharedOwner {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub active: bool,
}

#[derive(tyzen::Type, Serialize, Deserialize, Clone)]
#[serde(tag = "kind", content = "data")]
pub enum CommonStatus {
    Ready,
    Busy,
    Failed { message: String },
}

tyzen::module_ns!("Fixture");

#[derive(tyzen::Type, Serialize, Deserialize, Clone)]
pub struct ModelA {
    pub id: u64,
    pub key: String,
    pub title: String,
    pub owner: SharedOwner,
    pub status: CommonStatus,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub retries: u32,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(tyzen::Type, Serialize, Deserialize, Clone)]
pub struct ModelB {
    pub id: u64,
    pub owner: SharedOwner,
    pub status: CommonStatus,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub retries: u32,
    pub active: bool,
    pub ratio_a: f64,
    pub ratio_b: f64,
    pub value_a: i64,
    pub value_b: i64,
    pub value_c: i64,
    pub value_d: i64,
    pub flag_a: bool,
    pub flag_b: bool,
    pub flag_c: bool,
}

#[derive(tyzen::Type, Serialize, Deserialize, Clone)]
pub struct ModelC {
    pub id: u64,
    pub key: String,
    pub title: String,
    pub owner: SharedOwner,
    pub status: CommonStatus,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub retries: u32,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
    pub ratio_a: f64,
    pub ratio_b: f64,
}

#[tyzen::command]
pub fn fixture_fetch(id: u64) -> Result<ModelA, String> {
    let _ = id;
    Err("not implemented".to_string())
}

#[tyzen::command]
pub fn fixture_list(owner_id: u64) -> Result<Vec<ModelB>, String> {
    let _ = owner_id;
    Err("not implemented".to_string())
}

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/bench-bindings/cold-generate-fixture.ts".to_string());
    tyzen::generate(&out).expect("failed to generate fixture bindings");
    println!("generated={}", out);
}
