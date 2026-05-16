pub mod config;
pub mod detector;
pub mod engine;
pub mod entity;
pub mod error;
pub mod replacer;
pub mod storage_types;
pub mod storage_trait;

pub use engine::{DetectionEngine, EngineStats};
pub use entity::{EntityCategory, EntityType};
pub use error::{
    AidaGuardError, ConfigError, DetectionError, ErrorResponse,
    PluginError, ProxyError, StorageError,
};
pub use storage_types::{AuditFilter, AuditGroup, AuditStats, DetectionRecord, RuleCount};
pub use storage_trait::AuditStorage;

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
