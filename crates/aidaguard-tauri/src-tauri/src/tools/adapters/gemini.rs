use std::fs;
use std::path::PathBuf;
use crate::tools::home_dir;
use super::super::ToolAdapter;

fn config_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".gemini").join("settings.json"))
}

pub struct GeminiCli;

impl GeminiCli {
    pub fn new() -> Self { Self }
}

impl ToolAdapter for GeminiCli {
    fn id(&self) -> &str { "gemini" }
    fn name(&self) -> &str { "Gemini CLI" }
    fn config_path(&self) -> &str { "~/.gemini/settings.json" }

    fn detect(&self) -> bool {
        home_dir().map(|h| h.join(".gemini").exists()).unwrap_or(false)
    }

    fn current_endpoint(&self) -> Option<String> {
        // Gemini CLI uses GOOGLE_GEMINI_BASE_URL env var (unofficial)
        if let Ok(val) = std::env::var("GOOGLE_GEMINI_BASE_URL") {
            if !val.is_empty() { return Some(val); }
        }
        // Check settings.json for custom endpoint (future-proofing)
        let path = config_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path).ok()?;
            let json: serde_json::Value = serde_json::from_str(&content).ok()?;
            json.get("advanced")
                .and_then(|a| a.get("apiEndpoint"))
                .or_else(|| json.get("apiEndpoint"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        } else {
            None
        }
    }

    fn current_model(&self) -> Option<String> {
        // Check env var first
        if let Ok(val) = std::env::var("GEMINI_MODEL") {
            if !val.is_empty() { return Some(val); }
        }
        // Check settings.json
        let path = config_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path).ok()?;
            let json: serde_json::Value = serde_json::from_str(&content).ok()?;
            json.get("model")
                .and_then(|m| m.get("name"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        } else {
            None
        }
    }

    fn is_configured(&self) -> bool {
        self.current_endpoint()
            .map(|ep| ep.contains("127.0.0.1") || ep.contains("localhost"))
            .unwrap_or(false)
    }

    fn backup(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        // Backup .env file if it exists (Gemini stores keys there)
        if let Some(home) = home_dir() {
            let dotenv = home.join(".gemini").join(".env");
            if dotenv.exists() {
                super::super::backup::backup_config(&dotenv, backup_dir)?;
            }
        }
        // Also backup settings.json
        if let Some(path) = config_path() {
            if path.exists() {
                super::super::backup::backup_config(&path, backup_dir)?;
            }
        }
        Ok(())
    }

    fn configure(&self, proxy_url: &str) -> Result<(), String> {
        // Gemini CLI doesn't have native base URL support in settings.json (as of 2025).
        // The unofficial way is through GOOGLE_GEMINI_BASE_URL env var.
        // We write it into ~/.gemini/.env file.
        if let Some(home) = home_dir() {
            let gemini_dir = home.join(".gemini");
            let _ = fs::create_dir_all(&gemini_dir);
            let env_path = gemini_dir.join(".env");

            let mut content = if env_path.exists() {
                fs::read_to_string(&env_path).unwrap_or_default()
            } else {
                String::new()
            };

            // Update or add GOOGLE_GEMINI_BASE_URL
            let key = "GOOGLE_GEMINI_BASE_URL";
            if content.contains(&format!("{}=", key)) {
                // Replace existing line
                let mut new_lines: Vec<String> = Vec::new();
                for line in content.lines() {
                    if line.trim().starts_with(&format!("{}=", key)) {
                        new_lines.push(format!("{}={}", key, proxy_url));
                    } else {
                        new_lines.push(line.to_string());
                    }
                }
                content = new_lines.join("\n") + "\n";
            } else {
                if !content.is_empty() && !content.ends_with('\n') {
                    content.push('\n');
                }
                content.push_str(&format!("{}={}\n", key, proxy_url));
            }

            fs::write(&env_path, content)
                .map_err(|e| format!("写入 Gemini CLI .env 失败: {}", e))?;
        }
        Ok(())
    }

    fn restore(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        if let Some(home) = home_dir() {
            let dotenv = home.join(".gemini").join(".env");
            if dotenv.exists() {
                super::super::backup::restore_config(&dotenv, backup_dir)?;
            }
        }
        if let Some(path) = config_path() {
            if path.exists() {
                super::super::backup::restore_config(&path, backup_dir)?;
            }
        }
        Ok(())
    }
}
