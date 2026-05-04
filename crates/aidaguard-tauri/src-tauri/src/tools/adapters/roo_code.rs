use std::path::PathBuf;
use crate::tools::home_dir;
use super::super::ToolAdapter;

/// Roo Code VS Code 扩展存储目录（macOS）
fn storage_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        home_dir().map(|h| {
            h.join("Library").join("Application Support").join("Code")
                .join("User").join("globalStorage")
                .join("rooveterinaryinc.roo-cline").join("settings")
        })
    }
    #[cfg(target_os = "linux")]
    {
        home_dir().map(|h| {
            h.join(".config").join("Code").join("User")
                .join("globalStorage")
                .join("rooveterinaryinc.roo-cline").join("settings")
        })
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA").ok().map(|p| {
            PathBuf::from(p).join("Code").join("User")
                .join("globalStorage")
                .join("rooveterinaryinc.roo-cline").join("settings")
        })
    }
}

pub struct RooCode;

impl RooCode {
    pub fn new() -> Self { Self }
}

impl ToolAdapter for RooCode {
    fn id(&self) -> &str { "roo_code" }
    fn name(&self) -> &str { "Roo Code" }

    fn config_path(&self) -> &str {
        #[cfg(target_os = "macos")]
        { "~/Library/Application Support/Code/User/globalStorage/rooveterinaryinc.roo-cline/settings/" }
        #[cfg(target_os = "linux")]
        { "~/.config/Code/User/globalStorage/rooveterinaryinc.roo-cline/settings/" }
        #[cfg(target_os = "windows")]
        { "%APPDATA%/Code/User/globalStorage/rooveterinaryinc.roo-cline/settings/" }
    }

    fn detect(&self) -> bool {
        storage_dir().map(|p| p.exists()).unwrap_or(false)
    }

    fn current_endpoint(&self) -> Option<String> {
        // Roo Code 将 API provider 配置存储在多个 JSON 文件中
        // V1: 仅检测安装状态，详细配置读取留待后续增强
        None
    }

    fn current_model(&self) -> Option<String> {
        None
    }

    fn is_configured(&self) -> bool {
        false
    }

    fn backup(&self, _backup_dir: &std::path::Path) -> Result<(), String> {
        Err("Roo Code 一键配置将在后续版本支持".into())
    }

    fn configure(&self, _proxy_url: &str) -> Result<(), String> {
        Err("Roo Code 一键配置将在后续版本支持".into())
    }

    fn restore(&self, _backup_dir: &std::path::Path) -> Result<(), String> {
        Err("Roo Code 一键配置将在后续版本支持".into())
    }
}
