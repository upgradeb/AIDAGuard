use std::fs;
use std::path::PathBuf;
use crate::home_dir;
use crate::{Plugin, PluginManifest, ToolAdapter};

/// VS Code settings.json shared by all extensions (Cline, Roo Code, etc.)
fn vscode_settings_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        home_dir().map(|h| {
            h.join("Library").join("Application Support").join("Code")
                .join("User").join("settings.json")
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

/// Cline globalStorage directory
fn storage_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        home_dir().map(|h| {
            h.join("Library").join("Application Support").join("Code")
                .join("User").join("globalStorage")
                .join("saoudrizwan.claude-dev")
        })
    }
    #[cfg(target_os = "linux")]
    {
        home_dir().map(|h| {
            h.join(".config").join("Code").join("User")
                .join("globalStorage")
                .join("saoudrizwan.claude-dev")
        })
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA").ok().map(|p| {
            PathBuf::from(p).join("Code").join("User")
                .join("globalStorage")
                .join("saoudrizwan.claude-dev")
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
        { "~/Library/Application Support/Code/User/settings.json" }
        #[cfg(target_os = "linux")]
        { "~/.config/Code/User/settings.json" }
        #[cfg(target_os = "windows")]
        { "%APPDATA%/Code/User/settings.json" }
    }

    fn detect(&self) -> bool {
        storage_dir().map(|p| p.exists()).unwrap_or(false)
    }

    fn current_endpoint(&self) -> Option<String> {
        let path = vscode_settings_path()?;
        let content = fs::read_to_string(&path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        json.get("http.proxy")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    fn current_model(&self) -> Option<String> {
        let path = vscode_settings_path()?;
        let content = fs::read_to_string(&path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        json.get("roo-cline.model")
            .or_else(|| json.get("cline.model"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    fn is_configured(&self) -> bool {
        self.current_endpoint()
            .map(|ep| ep.contains("127.0.0.1") || ep.contains("localhost"))
            .unwrap_or(false)
    }

    fn backup(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        let path = vscode_settings_path()
            .ok_or("Failed to determine VS Code settings path".to_string())?;
        if !path.exists() {
            return Err("VS Code settings.json not found".into());
        }
        crate::backup::backup_config(&path, backup_dir)
    }

    fn configure(&self, proxy_url: &str) -> Result<(), String> {
        let path = vscode_settings_path()
            .ok_or("Failed to determine VS Code settings path".to_string())?;
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read VS Code settings: {}", e))?;
        let mut json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse VS Code settings: {}", e))?;

        if let Some(obj) = json.as_object_mut() {
            obj.insert("http.proxy".to_string(), serde_json::Value::String(proxy_url.to_string()));
            obj.insert("http.proxyStrictSSL".to_string(), serde_json::Value::Bool(false));
        }

        let new_content = serde_json::to_string_pretty(&json)
            .map_err(|e| format!("Serialization failed: {}", e))?;
        fs::write(&path, &new_content)
            .map_err(|e| format!("Failed to write VS Code settings: {}", e))?;
        Ok(())
    }

    fn restore(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        let path = vscode_settings_path()
            .ok_or("Failed to determine VS Code settings path".to_string())?;
        crate::backup::restore_config(&path, backup_dir)
    }
}

impl Plugin for Cline {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            id: "cline".into(),
            name: "Cline".into(),
            version: "1.0.0".into(),
            description: "VS Code extension for autonomous coding (saoudrizwan.claude-dev)".into(),
            author: "Cline".into(),
            config_path_template: "~/Library/Application Support/Code/User/settings.json".into(),
            categories: vec!["vscode-extension".into(), "openai-compatible".into()],
        }
    }
}
