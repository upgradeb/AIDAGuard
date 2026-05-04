use std::fs;
use std::path::PathBuf;
use crate::tools::home_dir;
use super::super::ToolAdapter;

fn config_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".claude").join("settings.json"))
}

pub struct ClaudeCode;

impl ClaudeCode {
    pub fn new() -> Self { Self }
}

impl ToolAdapter for ClaudeCode {
    fn id(&self) -> &str { "claude_code" }
    fn name(&self) -> &str { "Claude Code" }
    fn config_path(&self) -> &str { "~/.claude/settings.json" }

    fn detect(&self) -> bool {
        home_dir().map(|h| h.join(".claude").exists()).unwrap_or(false)
    }

    fn current_endpoint(&self) -> Option<String> {
        // Claude Code 可能通过环境变量 ANTHROPIC_BASE_URL 或 settings.json 配置
        if let Ok(val) = std::env::var("ANTHROPIC_BASE_URL") {
            if !val.is_empty() { return Some(val); }
        }
        let path = config_path()?;
        let content = fs::read_to_string(&path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        json.get("ANTHROPIC_BASE_URL")
            .or_else(|| json.get("baseUrl"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    fn current_model(&self) -> Option<String> {
        let path = config_path()?;
        let content = fs::read_to_string(&path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        json.get("model").and_then(|v| v.as_str()).map(|s| s.to_string())
    }

    fn is_configured(&self) -> bool {
        self.current_endpoint()
            .map(|ep| ep.contains("127.0.0.1") || ep.contains("localhost"))
            .unwrap_or(false)
    }

    fn backup(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        if let Some(path) = config_path() {
            if path.exists() {
                super::super::backup::backup_config(&path, backup_dir)?;
            }
        }
        Ok(())
    }

    fn configure(&self, proxy_url: &str) -> Result<(), String> {
        let path = config_path().ok_or("无法确定 Claude Code 配置路径".to_string())?;

        // 确保目录存在
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let mut json: serde_json::Value = if path.exists() {
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("读取 Claude Code 配置失败: {}", e))?;
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        if let Some(obj) = json.as_object_mut() {
            obj.insert("ANTHROPIC_BASE_URL".to_string(), serde_json::Value::String(proxy_url.to_string()));
        }

        let new_content = serde_json::to_string_pretty(&json)
            .map_err(|e| format!("序列化配置失败: {}", e))?;
        fs::write(&path, new_content)
            .map_err(|e| format!("写入配置失败: {}", e))?;
        Ok(())
    }

    fn restore(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        if let Some(path) = config_path() {
            if path.exists() {
                super::super::backup::restore_config(&path, backup_dir)?;
            }
        }
        Ok(())
    }
}
