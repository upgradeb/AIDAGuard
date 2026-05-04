use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 跨平台获取用户主目录
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

pub mod adapters;
pub mod backup;

/// 工具配置信息（前后端共享）
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
}

/// 工具适配器 trait
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
    // 检查是否已被 Aidaguard 配置过
    fn is_configured(&self) -> bool;
}

/// 获取所有已注册的适配器
pub fn all_adapters() -> Vec<Box<dyn ToolAdapter>> {
    vec![
        Box::new(adapters::RooCode::new()),
        Box::new(adapters::Cline::new()),
        Box::new(adapters::ContinueDev::new()),
        Box::new(adapters::Cursor::new()),
        Box::new(adapters::Windsurf::new()),
        Box::new(adapters::Zed::new()),
        Box::new(adapters::Aider::new()),
        Box::new(adapters::ClaudeCode::new()),
        Box::new(adapters::OpenClaw::new()),
        Box::new(adapters::HermesAgent::new()),
        Box::new(adapters::Codex::new()),
        Box::new(adapters::GeminiCli::new()),
        Box::new(adapters::OpenCode::new()),
    ]
}
