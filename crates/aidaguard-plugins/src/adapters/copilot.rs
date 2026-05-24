//! GitHub Copilot adapter
//!
//! GitHub Copilot uses VS Code settings and GitHub OAuth authentication.
//! Configuration is done via VS Code's settings.json.

use std::fs;
use std::path::PathBuf;
use crate::home_dir;
use crate::{Plugin, PluginManifest, ToolAdapter};

fn vscode_settings_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        home_dir().map(|h| {
            h.join("Library").join("Application Support")
                .join("Code").join("User").join("settings.json")
        })
    }
    #[cfg(target_os = "linux")]
    {
        home_dir().map(|h| h.join(".config").join("Code").join("User").join("settings.json"))
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA").ok().map(|p| {
            PathBuf::from(p).join("Code").join("User").join("settings.json")
        })
    }
}

fn copilot_extension_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        home_dir().map(|h| h.join(".vscode").join("extensions"))
    }
    #[cfg(target_os = "linux")]
    {
        home_dir().map(|h| h.join(".vscode").join("extensions"))
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE").ok().map(|p| {
            PathBuf::from(p).join(".vscode").join("extensions")
        })
    }
}

pub struct Copilot;

impl Copilot {
    pub fn new() -> Self { Self }
}

impl ToolAdapter for Copilot {
    fn id(&self) -> &str { "copilot" }
    fn name(&self) -> &str { "GitHub Copilot" }

    fn config_path(&self) -> &str {
        #[cfg(target_os = "macos")]
        { "~/Library/Application Support/Code/User/settings.json" }
        #[cfg(target_os = "linux")]
        { "~/.config/Code/User/settings.json" }
        #[cfg(target_os = "windows")]
        { "%APPDATA%/Code/User/settings.json" }
    }

    fn detect(&self) -> bool {
        // Check if Copilot extension is installed
        if let Some(ext_dir) = copilot_extension_path() {
            if ext_dir.exists() {
                if let Ok(entries) = fs::read_dir(&ext_dir) {
                    for entry in entries.flatten() {
                        let name = entry.file_name();
                        let name = name.to_string_lossy();
                        if name.starts_with("github.copilot") {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn current_endpoint(&self) -> Option<String> {
        // GitHub Copilot uses fixed endpoints
        Some("https://api.githubcopilot.com".to_string())
    }

    fn current_model(&self) -> Option<String> {
        // Copilot uses gpt-4o-copilot by default
        Some("gpt-4o-copilot".to_string())
    }

    fn is_configured(&self) -> bool {
        // Check if proxy is configured in VS Code settings
        let path = match vscode_settings_path() {
            Some(p) => p,
            None => return false,
        };
        if !path.exists() {
            return false;
        }
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return false,
        };
        let json: serde_json::Value = match serde_json::from_str(&content) {
            Ok(j) => j,
            Err(_) => return false,
        };
        json.get("http.proxy").is_some()
    }

    fn backup(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        let path = vscode_settings_path()
            .ok_or("Failed to determine VS Code settings path")?;
        crate::backup::backup_config(&path, backup_dir)
    }

    fn configure(&self, proxy_url: &str) -> Result<(), String> {
        let path = vscode_settings_path()
            .ok_or("Failed to determine VS Code settings path")?;

        // Read existing config or create empty
        let mut json: serde_json::Value = if path.exists() {
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read settings: {}", e))?;
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        // Set proxy configuration
        if let Some(obj) = json.as_object_mut() {
            obj.insert("http.proxy".to_string(), serde_json::Value::String(proxy_url.to_string()));
            obj.insert("http.proxyStrictSSL".to_string(), serde_json::Value::Bool(false));
        }

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let new_content = serde_json::to_string_pretty(&json)
            .map_err(|e| format!("Serialization failed: {}", e))?;
        fs::write(&path, new_content)
            .map_err(|e| format!("Failed to write config: {}", e))?;
        Ok(())
    }

    fn restore(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        let path = vscode_settings_path()
            .ok_or("Failed to determine VS Code settings path")?;
        crate::backup::restore_config(&path, backup_dir)
    }
}

impl Plugin for Copilot {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            id: "copilot".into(),
            name: "GitHub Copilot".into(),
            version: "1.0.0".into(),
            description: "AI pair programmer by GitHub".into(),
            author: "GitHub".into(),
            config_path_template: "~/Library/Application Support/Code/User/settings.json".into(),
            categories: vec!["vscode-extension".into(), "ide".into()],
        }
    }
}
