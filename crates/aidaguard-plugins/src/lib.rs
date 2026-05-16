use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Cross-platform home directory lookup
pub fn home_dir() -> Option<PathBuf> {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE").ok().map(PathBuf::from)
    }
}

pub mod abi;
pub mod adapters;
pub mod backup;
pub mod loader;
pub mod registry;

pub use registry::{Plugin, PluginManifest, PluginRegistry};
pub use loader::{DynamicPlugin, DynamicManifest, PluginLoader, PluginError};
pub use abi::{PluginMeta, PluginVTable, ABI_VERSION};

/// Tool configuration info (shared between frontend and backend)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolInfo {
    pub tool_id: String,
    pub tool_name: String,
    pub installed: bool,
    pub configured: bool,
    pub config_path: String,
    pub current_endpoint: Option<String>,
    pub current_model: Option<String>,
    pub preview_endpoint: Option<String>,
    /// Plugin metadata
    pub version: String,
    pub description: String,
    pub author: String,
    pub categories: Vec<String>,
    pub enabled: bool,
}

/// Tool adapter trait
pub trait ToolAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn config_path(&self) -> &str;
    fn detect(&self) -> bool;
    fn current_endpoint(&self) -> Option<String>;
    fn current_model(&self) -> Option<String>;
    fn backup(&self, backup_dir: &std::path::Path) -> Result<(), String>;
    fn configure(&self, proxy_url: &str) -> Result<(), String>;
    fn restore(&self, backup_dir: &std::path::Path) -> Result<(), String>;
    /// Check if already configured by Aidaguard
    fn is_configured(&self) -> bool;
}
