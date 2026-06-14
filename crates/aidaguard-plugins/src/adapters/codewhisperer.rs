//! Amazon CodeWhisperer adapter
//!
//! CodeWhisperer is Amazon's AI coding companion.
//! Configuration uses AWS credentials and VS Code settings.

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

fn aws_config_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".aws").join("config"))
}

fn aws_credentials_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".aws").join("credentials"))
}

pub struct CodeWhisperer;

impl CodeWhisperer {
    pub fn new() -> Self { Self }
}

impl ToolAdapter for CodeWhisperer {
    fn id(&self) -> &str { "codewhisperer" }
    fn name(&self) -> &str { "Amazon CodeWhisperer" }

    fn config_path(&self) -> &str {
        "~/.aws/credentials"
    }

    fn detect(&self) -> bool {
        // Check for AWS toolkit extension in VS Code
        if let Some(home) = home_dir() {
            let ext_dir = home.join(".vscode").join("extensions");
            if ext_dir.exists() {
                if let Ok(entries) = fs::read_dir(&ext_dir) {
                    for entry in entries.flatten() {
                        let name = entry.file_name();
                        let name = name.to_string_lossy();
                        if name.contains("amazonwebservices.aws-toolkit-vscode") {
                            return true;
                        }
                    }
                }
            }
        }
        // Check for AWS config
        if let Some(path) = aws_config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if content.contains("codewhisperer") || content.contains("code-whisperer") {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn current_endpoint(&self) -> Option<String> {
        // CodeWhisperer uses AWS endpoints
        Some("https://codewhisperer.us-east-1.amazonaws.com".to_string())
    }

    fn current_model(&self) -> Option<String> {
        Some("code-whisperer".to_string())
    }

    fn is_configured(&self) -> bool {
        // Check for CodeWhisperer-specific proxy key (aws.proxy), not generic http.proxy
        // which could have been set by the VS Code adapter or other tools.
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
        let ep = json.get("aws.proxy").and_then(|v| v.as_str()).unwrap_or("");
        ep.contains("127.0.0.1") || ep.contains("localhost")
    }

    fn backup(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        let mut backed_up = false;

        if let Some(path) = vscode_settings_path() {
            if path.exists() {
                crate::backup::backup_config(&path, backup_dir)?;
                backed_up = true;
            }
        }

        if let Some(path) = aws_config_path() {
            if path.exists() {
                crate::backup::backup_config(&path, backup_dir)?;
                backed_up = true;
            }
        }

        if let Some(path) = aws_credentials_path() {
            if path.exists() {
                crate::backup::backup_config(&path, backup_dir)?;
                backed_up = true;
            }
        }

        if backed_up {
            Ok(())
        } else {
            Err("No CodeWhisperer configuration found to backup".to_string())
        }
    }

    fn configure(&self, proxy_url: &str) -> Result<(), String> {
        // Configure VS Code settings for proxy
        if let Some(path) = vscode_settings_path() {
            let mut json: serde_json::Value = if path.exists() {
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read settings: {}", e))?;
                serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
            } else {
                serde_json::json!({})
            };

            if let Some(obj) = json.as_object_mut() {
                obj.insert("http.proxy".to_string(), serde_json::Value::String(proxy_url.to_string()));
                obj.insert("http.proxyStrictSSL".to_string(), serde_json::Value::Bool(false));
                // AWS Toolkit specific settings
                obj.insert("aws.proxy".to_string(), serde_json::Value::String(proxy_url.to_string()));
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

        if let Some(path) = vscode_settings_path() {
            if crate::backup::restore_config(&path, backup_dir).is_ok() {
                restored = true;
            }
        }

        if let Some(path) = aws_config_path() {
            if crate::backup::restore_config(&path, backup_dir).is_ok() {
                restored = true;
            }
        }

        if let Some(path) = aws_credentials_path() {
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

impl Plugin for CodeWhisperer {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            id: "codewhisperer".into(),
            name: "Amazon CodeWhisperer".into(),
            version: "1.0.0".into(),
            description: "AI coding companion by Amazon".into(),
            author: "Amazon Web Services".into(),
            config_path_template: "~/.aws/credentials".into(),
            categories: vec!["vscode-extension".into(), "aws".into()],
        }
    }
}
