use std::path::PathBuf;
use crate::home_dir;
use crate::{Plugin, PluginManifest, ToolAdapter};

fn storage_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        home_dir().map(|h| {
            h.join("Library").join("Application Support").join("Code")
                .join("User").join("globalStorage")
                .join("saoudrizwan.claude-dev").join("settings")
        })
    }
    #[cfg(target_os = "linux")]
    {
        home_dir().map(|h| {
            h.join(".config").join("Code").join("User")
                .join("globalStorage")
                .join("saoudrizwan.claude-dev").join("settings")
        })
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA").ok().map(|p| {
            PathBuf::from(p).join("Code").join("User")
                .join("globalStorage")
                .join("saoudrizwan.claude-dev").join("settings")
        })
    }
}

pub struct Cline;

impl Cline {
    pub fn new() -> Self { Self }
}

impl ToolAdapter for Cline {
    fn id(&self) -> &str { "cline" }
    fn name(&self) -> &str { "Cline" }

    fn config_path(&self) -> &str {
        #[cfg(target_os = "macos")]
        { "~/Library/Application Support/Code/User/globalStorage/saoudrizwan.claude-dev/settings/" }
        #[cfg(target_os = "linux")]
        { "~/.config/Code/User/globalStorage/saoudrizwan.claude-dev/settings/" }
        #[cfg(target_os = "windows")]
        { "%APPDATA%/Code/User/globalStorage/saoudrizwan.claude-dev/settings/" }
    }

    fn detect(&self) -> bool {
        storage_dir().map(|p| p.exists()).unwrap_or(false)
    }

    fn current_endpoint(&self) -> Option<String> {
        None
    }

    fn current_model(&self) -> Option<String> {
        None
    }

    fn is_configured(&self) -> bool {
        false
    }

    fn backup(&self, _backup_dir: &std::path::Path) -> Result<(), String> {
        Err("One-click configuration for Cline will be supported in a future version".into())
    }

    fn configure(&self, _proxy_url: &str) -> Result<(), String> {
        Err("One-click configuration for Cline will be supported in a future version".into())
    }

    fn restore(&self, _backup_dir: &std::path::Path) -> Result<(), String> {
        Err("One-click configuration for Cline will be supported in a future version".into())
    }
}

impl Plugin for Cline {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            id: "cline".into(),
            name: "Cline".into(),
            version: "1.0.0".into(),
            description: "VS Code extension for autonomous coding".into(),
            author: "Cline".into(),
            config_path_template: "VS Code storage".into(),
            categories: vec!["vscode-extension".into(), "openai-compatible".into()],
        }
    }
}
