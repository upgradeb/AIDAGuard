//! JetBrains AI adapter
//!
//! JetBrains AI is the AI assistant integrated into JetBrains IDEs.
//! Configuration is stored in IDE-specific config directories.

use std::fs;
use std::path::PathBuf;
use crate::home_dir;
use crate::{Plugin, PluginManifest, ToolAdapter};

fn jetbrains_config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        home_dir().map(|h| {
            h.join("Library").join("Application Support").join("JetBrains")
        })
    }
    #[cfg(target_os = "linux")]
    {
        home_dir().map(|h| {
            h.join(".config").join("JetBrains")
        })
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA").ok().map(|p| {
            PathBuf::from(p).join("JetBrains")
        })
    }
}

fn find_jetbrains_ide_options() -> Option<PathBuf> {
    let base = jetbrains_config_dir()?;
    if !base.exists() {
        return None;
    }

    // Look for any JetBrains IDE config
    let ide_patterns = [
        "IntelliJIdea", "PyCharm", "WebStorm", "GoLand",
        "CLion", "RubyMine", "PhpStorm", "Rider", "DataGrip"
    ];

    if let Ok(entries) = fs::read_dir(&base) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            for pattern in &ide_patterns {
                if name.starts_with(pattern) {
                    // Find ide.options file
                    let options_dir = entry.path().join("options");
                    if options_dir.exists() {
                        return Some(options_dir.join("ide.options.xml"));
                    }
                }
            }
        }
    }
    None
}

pub struct JetBrainsAI;

impl JetBrainsAI {
    pub fn new() -> Self { Self }
}

impl ToolAdapter for JetBrainsAI {
    fn id(&self) -> &str { "jetbrains-ai" }
    fn name(&self) -> &str { "JetBrains AI" }

    fn config_path(&self) -> &str {
        "~/Library/Application Support/JetBrains/*/options/ide.options.xml"
    }

    fn detect(&self) -> bool {
        // Check for JetBrains config directory with AI plugin
        if let Some(base) = jetbrains_config_dir() {
            if base.exists() {
                if let Ok(entries) = fs::read_dir(&base) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        // Look for AI-related files
                        let plugins_dir = path.join("plugins");
                        if plugins_dir.exists() {
                            if let Ok(plugin_entries) = fs::read_dir(&plugins_dir) {
                                for plugin in plugin_entries.flatten() {
                                    let name = plugin.file_name();
                                    let name = name.to_string_lossy();
                                    if name.contains("AIAssistant") || name.contains("JetBrainsAI") {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    fn current_endpoint(&self) -> Option<String> {
        // JetBrains AI uses fixed endpoints
        Some("https://api.jetbrains.ai".to_string())
    }

    fn current_model(&self) -> Option<String> {
        Some("jetbrains-ai-default".to_string())
    }

    fn is_configured(&self) -> bool {
        // Check if proxy is configured in ide.options.xml
        let path = match find_jetbrains_ide_options() {
            Some(p) => p,
            None => return false,
        };
        if !path.exists() {
            return false;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            // Simple check for proxy configuration
            return content.contains("proxy") && (content.contains("localhost") || content.contains("127.0.0.1"));
        }
        false
    }

    fn backup(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        let mut backed_up = false;

        // Backup all JetBrains IDE configs
        if let Some(base) = jetbrains_config_dir() {
            if base.exists() {
                if let Ok(entries) = fs::read_dir(&base) {
                    for entry in entries.flatten() {
                        let options_dir = entry.path().join("options");
                        if options_dir.exists() {
                            if let Ok(opt_entries) = fs::read_dir(&options_dir) {
                                for option_file in opt_entries.flatten() {
                                    let file_path = option_file.path();
                                    if let Some(ext) = file_path.extension() {
                                        if ext == "xml" {
                                            if crate::backup::backup_config(&file_path, backup_dir).is_ok() {
                                                backed_up = true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if backed_up {
            Ok(())
        } else {
            Err("No JetBrains AI configuration found to backup".to_string())
        }
    }

    fn configure(&self, proxy_url: &str) -> Result<(), String> {
        // Configure HTTP proxy for JetBrains IDEs
        if let Some(options_path) = find_jetbrains_ide_options() {
            let mut content = if options_path.exists() {
                fs::read_to_string(&options_path)
                    .map_err(|e| format!("Failed to read options: {}", e))?
            } else {
                r#"<?xml version="1.0" encoding="UTF-8"?>
<application>
  <component name="HttpConfigurable">
    <option name="USE_PROXY_PAC" value="false" />
  </component>
</application>"#.to_string()
            };

            // Simple XML modification (not using full XML parser for simplicity)
            // Add proxy configuration
            if !content.contains("PROXY_HOST") {
                content = content.replace("</component>", &format!(
                    r#"    <option name="PROXY_HOST" value="{}" />
    <option name="PROXY_PORT" value="{}" />
    <option name="USE_HTTP_PROXY" value="true" />
  </component>"#,
                    proxy_url.split(':').next().unwrap_or("localhost"),
                    proxy_url.split(':').nth(1).unwrap_or("8080")
                ));
            }

            if let Some(parent) = options_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }

            fs::write(&options_path, content)
                .map_err(|e| format!("Failed to write options: {}", e))?;
        }

        Ok(())
    }

    fn restore(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        // Restore from backup
        if let Some(base) = jetbrains_config_dir() {
            if base.exists() {
                if let Ok(entries) = fs::read_dir(&base) {
                    for entry in entries.flatten() {
                        let options_dir = entry.path().join("options");
                        if options_dir.exists() {
                            if let Ok(opt_entries) = fs::read_dir(&options_dir) {
                                for option_file in opt_entries.flatten() {
                                    let file_path = option_file.path();
                                    if let Some(ext) = file_path.extension() {
                                        if ext == "xml" {
                                            let _ = crate::backup::restore_config(&file_path, backup_dir);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl Plugin for JetBrainsAI {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            id: "jetbrains-ai".into(),
            name: "JetBrains AI".into(),
            version: "1.0.0".into(),
            description: "AI Assistant for JetBrains IDEs".into(),
            author: "JetBrains".into(),
            config_path_template: "~/Library/Application Support/JetBrains/*/options/ide.options.xml".into(),
            categories: vec!["ide-plugin".into(), "jetbrains".into()],
        }
    }
}
