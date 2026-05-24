//! Codeium adapter
//!
//! Codeium is a free AI coding assistant that works with multiple editors.
//! Configuration is stored in ~/.codeium/config.json or editor-specific settings.

use std::fs;
use std::path::PathBuf;
use crate::home_dir;
use crate::{Plugin, PluginManifest, ToolAdapter};

fn codeium_config_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        home_dir().map(|h| h.join(".codeium").join("config.json"))
    }
    #[cfg(target_os = "linux")]
    {
        home_dir().map(|h| h.join(".codeium").join("config.json"))
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE").ok().map(|p| {
            PathBuf::from(p).join(".codeium").join("config.json")
        })
    }
}

fn codeium_vscode_settings_path() -> Option<PathBuf> {
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

pub struct Codeium;

impl Codeium {
    pub fn new() -> Self { Self }
}

impl ToolAdapter for Codeium {
    fn id(&self) -> &str { "codeium" }
    fn name(&self) -> &str { "Codeium" }

    fn config_path(&self) -> &str {
        "~/.codeium/config.json"
    }

    fn detect(&self) -> bool {
        // Check for Codeium config or VS Code extension
        if let Some(path) = codeium_config_path() {
            if path.exists() {
                return true;
            }
        }
        // Check VS Code extension
        if let Some(home) = home_dir() {
            let ext_dir = home.join(".vscode").join("extensions");
            if ext_dir.exists() {
                if let Ok(entries) = fs::read_dir(&ext_dir) {
                    for entry in entries.flatten() {
                        let name = entry.file_name();
                        let name = name.to_string_lossy();
                        if name.starts_with("codeium") {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn current_endpoint(&self) -> Option<String> {
        let path = codeium_config_path()?;
        if !path.exists() {
            // Codeium uses default endpoint
            return Some("https://server.codeium.com".to_string());
        }
        let content = fs::read_to_string(&path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        json.get("api_base_url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    fn current_model(&self) -> Option<String> {
        // Codeium doesn't expose model selection
        Some("codeium-default".to_string())
    }

    fn is_configured(&self) -> bool {
        if let Some(path) = codeium_config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(url) = json.get("api_base_url").and_then(|v| v.as_str()) {
                            return url.contains("127.0.0.1") || url.contains("localhost");
                        }
                    }
                }
            }
        }
        false
    }

    fn backup(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        let mut backed_up = false;
        if let Some(path) = codeium_config_path() {
            if path.exists() {
                crate::backup::backup_config(&path, backup_dir)?;
                backed_up = true;
            }
        }
        if let Some(path) = codeium_vscode_settings_path() {
            if path.exists() {
                crate::backup::backup_config(&path, backup_dir)?;
                backed_up = true;
            }
        }
        if backed_up {
            Ok(())
        } else {
            Err("No Codeium configuration found to backup".to_string())
        }
    }

    fn configure(&self, proxy_url: &str) -> Result<(), String> {
        // Configure Codeium-specific config
        if let Some(path) = codeium_config_path() {
            let mut json: serde_json::Value = if path.exists() {
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read config: {}", e))?;
                serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
            } else {
                serde_json::json!({})
            };

            if let Some(obj) = json.as_object_mut() {
                obj.insert("api_base_url".to_string(), serde_json::Value::String(proxy_url.to_string()));
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
        }

        // Also configure VS Code settings for Codeium extension
        if let Some(path) = codeium_vscode_settings_path() {
            let mut json: serde_json::Value = if path.exists() {
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read settings: {}", e))?;
                serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
            } else {
                serde_json::json!({})
            };

            if let Some(obj) = json.as_object_mut() {
                obj.insert("codeium.apiBaseUrl".to_string(), serde_json::Value::String(proxy_url.to_string()));
            }

            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }

            let new_content = serde_json::to_string_pretty(&json)
                .map_err(|e| format!("Serialization failed: {}", e))?;
            fs::write(&path, new_content)
                .map_err(|e| format!("Failed to write settings: {}", e))?;
        }

        Ok(())
    }

    fn restore(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        let mut restored = false;
        if let Some(path) = codeium_config_path() {
            if crate::backup::restore_config(&path, backup_dir).is_ok() {
                restored = true;
            }
        }
        if let Some(path) = codeium_vscode_settings_path() {
            if crate::backup::restore_config(&path, backup_dir).is_ok() {
                restored = true;
            }
        }
        if restored {
            Ok(())
        } else {
            Err("No backup found to restore".to_string())
        }
    }
}

impl Plugin for Codeium {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            id: "codeium".into(),
            name: "Codeium".into(),
            version: "1.0.0".into(),
            description: "Free AI coding assistant".into(),
            author: "Codeium / Exafunction".into(),
            config_path_template: "~/.codeium/config.json".into(),
            categories: vec!["vscode-extension".into(), "free-tier".into()],
        }
    }
}
