//! Sourcegraph Cody adapter
//!
//! Cody is an AI coding assistant by Sourcegraph.
//! Configuration is stored in ~/.cody/config.json or editor settings.

use std::fs;
use std::path::PathBuf;
use crate::home_dir;
use crate::{Plugin, PluginManifest, ToolAdapter};

fn cody_config_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        home_dir().map(|h| h.join(".cody").join("config.json"))
    }
    #[cfg(target_os = "linux")]
    {
        home_dir().map(|h| h.join(".cody").join("config.json"))
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE").ok().map(|p| {
            PathBuf::from(p).join(".cody").join("config.json")
        })
    }
}

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

pub struct Cody;

impl Cody {
    pub fn new() -> Self { Self }
}

impl ToolAdapter for Cody {
    fn id(&self) -> &str { "cody" }
    fn name(&self) -> &str { "Sourcegraph Cody" }

    fn config_path(&self) -> &str {
        "~/.cody/config.json"
    }

    fn detect(&self) -> bool {
        // Check for Cody config
        if let Some(path) = cody_config_path() {
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
                        if name.contains("sourcegraph.cody") {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn current_endpoint(&self) -> Option<String> {
        let path = cody_config_path()?;
        if !path.exists() {
            return Some("https://sourcegraph.com".to_string());
        }
        let content = fs::read_to_string(&path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        json.get("endpoint")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    fn current_model(&self) -> Option<String> {
        let path = cody_config_path()?;
        if !path.exists() {
            return Some("claude-3-sonnet".to_string());
        }
        let content = fs::read_to_string(&path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        json.get("model")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    fn is_configured(&self) -> bool {
        if let Some(path) = cody_config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(url) = json.get("endpoint").and_then(|v| v.as_str()) {
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
        if let Some(path) = cody_config_path() {
            if path.exists() {
                crate::backup::backup_config(&path, backup_dir)?;
                backed_up = true;
            }
        }
        if let Some(path) = vscode_settings_path() {
            if path.exists() {
                crate::backup::backup_config(&path, backup_dir)?;
                backed_up = true;
            }
        }
        if backed_up {
            Ok(())
        } else {
            Err("No Cody configuration found to backup".to_string())
        }
    }

    fn configure(&self, proxy_url: &str) -> Result<(), String> {
        // Configure Cody-specific config
        if let Some(path) = cody_config_path() {
            let mut json: serde_json::Value = if path.exists() {
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read config: {}", e))?;
                serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
            } else {
                serde_json::json!({})
            };

            if let Some(obj) = json.as_object_mut() {
                obj.insert("endpoint".to_string(), serde_json::Value::String(proxy_url.to_string()));
            }

            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }

            let new_content = serde_json::to_string_pretty(&json)
                .map_err(|e| format!("Serialization failed: {}", e))?;
            fs::write(&path, new_content)
                .map_err(|e| format!("Failed to write config: {}", e))?;
        }

        // Also configure VS Code settings
        if let Some(path) = vscode_settings_path() {
            let mut json: serde_json::Value = if path.exists() {
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read settings: {}", e))?;
                serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
            } else {
                serde_json::json!({})
            };

            if let Some(obj) = json.as_object_mut() {
                obj.insert("cody.endpoint".to_string(), serde_json::Value::String(proxy_url.to_string()));
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
        if let Some(path) = cody_config_path() {
            if crate::backup::restore_config(&path, backup_dir).is_ok() {
                restored = true;
            }
        }
        if let Some(path) = vscode_settings_path() {
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

impl Plugin for Cody {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            id: "cody".into(),
            name: "Sourcegraph Cody".into(),
            version: "1.0.0".into(),
            description: "AI coding assistant by Sourcegraph".into(),
            author: "Sourcegraph".into(),
            config_path_template: "~/.cody/config.json".into(),
            categories: vec!["vscode-extension".into(), "code-assistant".into()],
        }
    }
}
