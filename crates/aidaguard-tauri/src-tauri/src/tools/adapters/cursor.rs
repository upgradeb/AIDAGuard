use std::fs;
use std::path::PathBuf;
use crate::tools::home_dir;
use super::super::ToolAdapter;

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
        // Cursor 可能使用 cursor.apiBase 或其他键
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
        let path = config_path().ok_or("无法确定 Cursor 配置路径".to_string())?;
        super::super::backup::backup_config(&path, backup_dir)
    }

    fn configure(&self, proxy_url: &str) -> Result<(), String> {
        let path = config_path().ok_or("无法确定 Cursor 配置路径".to_string())?;
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("读取 Cursor 配置失败: {}", e))?;
        let mut json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("解析 Cursor 配置失败: {}", e))?;

        if let Some(obj) = json.as_object_mut() {
            obj.insert("cursor.apiBase".to_string(), serde_json::Value::String(proxy_url.to_string()));
            obj.insert("openai.baseUrl".to_string(), serde_json::Value::String(proxy_url.to_string()));
        }

        let new_content = serde_json::to_string_pretty(&json)
            .map_err(|e| format!("序列化配置失败: {}", e))?;
        fs::write(&path, new_content)
            .map_err(|e| format!("写入配置失败: {}", e))?;
        Ok(())
    }

    fn restore(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        let path = config_path().ok_or("无法确定 Cursor 配置路径".to_string())?;
        super::super::backup::restore_config(&path, backup_dir)
    }
}
