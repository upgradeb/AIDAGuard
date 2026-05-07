use std::fs;
use std::path::PathBuf;
use crate::home_dir;
use crate::{Plugin, PluginManifest, ToolAdapter};

fn config_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        home_dir().map(|h| {
            h.join("Library").join("Application Support")
                .join("Cursor").join("User").join("settings.json")
        })
    }
    #[cfg(target_os = "linux")]
    {
        home_dir().map(|h| h.join(".config").join("Cursor").join("User").join("settings.json"))
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA").ok().map(|p| {
            PathBuf::from(p).join("Cursor").join("User").join("settings.json")
        })
    }
}

pub struct Cursor;

impl Cursor {
    pub fn new() -> Self { Self }
}

impl ToolAdapter for Cursor {
    fn id(&self) -> &str { "cursor" }
    fn name(&self) -> &str { "Cursor" }

    fn config_path(&self) -> &str {
        #[cfg(target_os = "macos")]
        { "~/Library/Application Support/Cursor/User/settings.json" }
        #[cfg(target_os = "linux")]
        { "~/.config/Cursor/User/settings.json" }
        #[cfg(target_os = "windows")]
        { "%APPDATA%/Cursor/User/settings.json" }
    }

    fn detect(&self) -> bool {
        config_path().map(|p| p.exists()).unwrap_or(false)
    }

    fn current_endpoint(&self) -> Option<String> {
        let path = config_path()?;
        let content = fs::read_to_string(&path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        // Cursor may use cursor.apiBase or other keys
        json.get("cursor.apiBase")
            .or_else(|| json.get("openai.baseUrl"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    fn current_model(&self) -> Option<String> {
        let path = config_path()?;
        let content = fs::read_to_string(&path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        json.get("cursor.model")
            .or_else(|| json.get("openai.model"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    fn is_configured(&self) -> bool {
        self.current_endpoint()
            .map(|ep| ep.contains("127.0.0.1") || ep.contains("localhost"))
            .unwrap_or(false)
    }

    fn backup(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        let path = config_path().ok_or("Failed to determine Cursor config path".to_string())?;
        crate::backup::backup_config(&path, backup_dir)
    }

    fn configure(&self, proxy_url: &str) -> Result<(), String> {
        let path = config_path().ok_or("Failed to determine Cursor config path".to_string())?;
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read Cursor config: {}", e))?;
        let mut json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse Cursor config: {}", e))?;

        if let Some(obj) = json.as_object_mut() {
            obj.insert("cursor.apiBase".to_string(), serde_json::Value::String(proxy_url.to_string()));
            obj.insert("openai.baseUrl".to_string(), serde_json::Value::String(proxy_url.to_string()));
        }

        let new_content = serde_json::to_string_pretty(&json)
            .map_err(|e| format!("Serialization failed: {}", e))?;
        fs::write(&path, new_content)
            .map_err(|e| format!("Failed to write config: {}", e))?;
        Ok(())
    }

    fn restore(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        let path = config_path().ok_or("Failed to determine Cursor config path".to_string())?;
        crate::backup::restore_config(&path, backup_dir)
    }
}

impl Plugin for Cursor {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            id: "cursor".into(),
            name: "Cursor".into(),
            version: "1.0.0".into(),
            description: "AI-first code editor".into(),
            author: "Cursor / Anysphere".into(),
            config_path_template: "~/Library/Application Support/Cursor/User/settings.json".into(),
            categories: vec!["editor".into(), "openai-compatible".into()],
        }
    }
}
