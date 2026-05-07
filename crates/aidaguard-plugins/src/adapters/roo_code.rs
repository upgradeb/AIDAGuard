use std::path::PathBuf;
use crate::home_dir;
use crate::{Plugin, PluginManifest, ToolAdapter};

/// Roo Code VS Code extension storage directory (macOS)
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
        // Roo Code stores API provider config across multiple JSON files
        // V1: Only detect installation status; detailed config reading deferred
        None
    }

    fn current_model(&self) -> Option<String> {
        None
    }

    fn is_configured(&self) -> bool {
        false
    }

    fn backup(&self, _backup_dir: &std::path::Path) -> Result<(), String> {
        Err("One-click configuration for Roo Code will be supported in a future version".into())
    }

    fn configure(&self, _proxy_url: &str) -> Result<(), String> {
        Err("One-click configuration for Roo Code will be supported in a future version".into())
    }

    fn restore(&self, _backup_dir: &std::path::Path) -> Result<(), String> {
        Err("One-click configuration for Roo Code will be supported in a future version".into())
    }
}

impl Plugin for RooCode {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            id: "roo_code".into(),
            name: "Roo Code".into(),
            version: "1.0.0".into(),
            description: "VS Code extension for AI coding".into(),
            author: "Roo Code".into(),
            config_path_template: "VS Code storage".into(),
            categories: vec!["vscode-extension".into(), "openai-compatible".into()],
        }
    }
}
