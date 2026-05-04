use std::path::PathBuf;
use crate::tools::home_dir;
use super::super::ToolAdapter;

fn config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        home_dir().map(|h| {
            h.join("Library").join("Application Support")
                .join("Windsurf").join("User")
        })
    }
    #[cfg(target_os = "linux")]
    {
        home_dir().map(|h| h.join(".config").join("Windsurf").join("User"))
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA").ok().map(|p| {
            PathBuf::from(p).join("Windsurf").join("User")
        })
    }
}

pub struct Windsurf;

impl Windsurf {
    pub fn new() -> Self { Self }
}

impl ToolAdapter for Windsurf {
    fn id(&self) -> &str { "windsurf" }
    fn name(&self) -> &str { "Windsurf" }

    fn config_path(&self) -> &str {
        #[cfg(target_os = "macos")]
        { "~/Library/Application Support/Windsurf/User/settings.json" }
        #[cfg(target_os = "linux")]
        { "~/.config/Windsurf/User/settings.json" }
        #[cfg(target_os = "windows")]
        { "%APPDATA%/Windsurf/User/settings.json" }
    }

    fn detect(&self) -> bool {
        config_dir().map(|p| p.join("settings.json").exists()).unwrap_or(false)
    }

    fn current_endpoint(&self) -> Option<String> {
        None
    }

    fn current_model(&self) -> Option<String> {
        None
    }

    fn is_configured(&self) -> bool {
        false
    }

    fn backup(&self, _backup_dir: &std::path::Path) -> Result<(), String> {
        Err("Windsurf 一键配置将在后续版本支持".into())
    }

    fn configure(&self, _proxy_url: &str) -> Result<(), String> {
        Err("Windsurf 一键配置将在后续版本支持".into())
    }

    fn restore(&self, _backup_dir: &std::path::Path) -> Result<(), String> {
        Err("Windsurf 一键配置将在后续版本支持".into())
    }
}
