#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct SharedOwner {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub active: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum CommonStatus {
    Ready,
    Busy,
    Failed { message: String },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelA {
    pub id: u64,
    pub key: String,
    pub title: String,
    pub owner: SharedOwner,
    pub status: CommonStatus,
    pub tags: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelB {
    pub id: u64,
    pub created_at: String,
    pub updated_at: String,
    pub retries: u32,
    pub active: bool,
    pub ratio_a: f64,
    pub ratio_b: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelC {
    pub value_a: i64,
    pub value_b: i64,
    pub value_c: i64,
    pub value_d: i64,
    pub flag_a: bool,
    pub flag_b: bool,
    pub flag_c: bool,
}

fn main() {
    // Baseline example for build-time comparison; no Tyzen macros.
    println!("baseline-ok");
}
