use serde::{Deserialize, Serialize};
use std::path::Path;

fn default_port() -> u16 { 19000 }
fn default_target_url() -> String { String::new() }
fn default_rules_dir() -> String { "./rules".to_string() }
fn default_log_level() -> String { "info".to_string() }
fn default_api_key() -> String { String::new() }
fn default_max_body_size_mb() -> usize { 10 }
fn default_storage_enabled() -> bool { false }
fn default_storage_db_path() -> String { "./data/aidaguard.db".to_string() }
fn default_notification_enabled() -> bool { true }
fn default_notification_rate_limit_secs() -> u64 { 60 }

/// 存储子配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    /// 是否启用审计记录
    #[serde(default = "default_storage_enabled")]
    pub enabled: bool,

    /// 数据库文件路径
    #[serde(default = "default_storage_db_path")]
    pub db_path: String,

    /// 加密密钥，未设置时使用默认密钥
    #[serde(default)]
    pub encryption_key: Option<String>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            enabled: default_storage_enabled(),
            db_path: default_storage_db_path(),
            encryption_key: None,
        }
    }
}

/// 上游 LLM 接入配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpstreamConfig {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub default: bool,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    #[serde(default)]
    pub rate_limit_qps: u32,
    #[serde(default)]
    pub models: Vec<String>,
}

fn default_timeout() -> u64 { 300 }

impl Default for UpstreamConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            url: String::new(),
            api_key: None,
            default: false,
            timeout_secs: default_timeout(),
            rate_limit_qps: 0,
            models: Vec::new(),
        }
    }
}

/// 通知子配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotificationConfig {
    #[serde(default = "default_notification_enabled")]
    pub enabled: bool,

    /// 同一规则最短通知间隔（秒），默认 60
    #[serde(default = "default_notification_rate_limit_secs")]
    pub rate_limit_secs: u64,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: default_notification_enabled(),
            rate_limit_secs: default_notification_rate_limit_secs(),
        }
    }
}

/// Aidaguard 配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "default_api_key")]
    pub api_key: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_target_url")]
    pub target_url: String,

    #[serde(default = "default_rules_dir")]
    pub rules_dir: String,

    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// 请求体最大大小（MB），超出返回 413
    #[serde(default = "default_max_body_size_mb")]
    pub max_body_size_mb: usize,

    #[serde(default)]
    pub storage: StorageConfig,

    /// 上游 LLM 列表
    #[serde(default)]
    pub upstreams: Vec<UpstreamConfig>,

    /// 桌面通知配置
    #[serde(default)]
    pub notification: NotificationConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: default_api_key(),
            port: default_port(),
            target_url: default_target_url(),
            rules_dir: default_rules_dir(),
            log_level: default_log_level(),
            max_body_size_mb: default_max_body_size_mb(),
            storage: StorageConfig::default(),
            upstreams: Vec::new(),
            notification: NotificationConfig::default(),
        }
    }
}

impl Config {
    /// 从 config.toml 文件加载配置，文件不存在时使用默认值。
    pub fn load() -> Self {
        Self::load_from(Path::new("config.toml")).unwrap_or_default()
    }

    /// 从指定路径加载配置。
    pub fn load_from(path: &Path) -> Option<Self> {
        if !path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(path).ok()?;
        match toml::from_str(&content) {
            Ok(c) => {
                tracing::info!("已加载配置文件: {}", path.display());
                Some(c)
            }
            Err(e) => {
                tracing::warn!("配置文件解析失败 {}: {}", path.display(), e);
                None
            }
        }
    }

    /// 将配置写入 TOML 文件。
    pub fn save_to(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        tracing::info!("配置已保存: {}", path.display());
        Ok(())
    }
}
