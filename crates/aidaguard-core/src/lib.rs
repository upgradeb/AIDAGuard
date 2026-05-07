pub mod config;
pub mod detector;
pub mod engine;
pub mod entity;
pub mod replacer;
pub mod storage;

pub use engine::DetectionEngine;
pub use entity::{EntityCategory, EntityType};
pub use storage::{AuditGroup, AuditStats, DetectionRecord, RuleCount};

use serde::{Deserialize, Serialize};

/// Detection event broadcast from proxy task to Tauri frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionEvent {
    pub timestamp_ms: i64,
    pub rule_id: String,
    pub strategy: String,
    pub placeholder: String,
    pub request_path: String,
    pub response_status: u16,
    pub tool_name: String,
}

/// Aidaguard core version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
