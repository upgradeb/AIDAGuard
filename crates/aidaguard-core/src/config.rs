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
fn default_storage_type() -> String { "sqlite".to_string() }
fn default_notification_enabled() -> bool { true }
fn default_notification_rate_limit_secs() -> u64 { 60 }
fn default_region() -> String { "global".to_string() }
fn default_rule_industries() -> Vec<String> { Vec::new() }

/// 存储子配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    /// 是否启用审计记录
    #[serde(default = "default_storage_enabled")]
    pub enabled: bool,

    /// 存储类型: "sqlite" | "memory"
    #[serde(default = "default_storage_type")]
    pub storage_type: String,

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
            storage_type: default_storage_type(),
            db_path: default_storage_db_path(),
            encryption_key: None,
        }
    }
}

/// 上游协议类型
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UpstreamProtocol {
    /// OpenAI 兼容协议 (默认)
    #[serde(alias = "openai")]
    OpenAi,
    /// Anthropic 兼容协议
    #[serde(alias = "anthropic")]
    Anthropic,
}

impl std::fmt::Display for UpstreamProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenAi => write!(f, "openai"),
            Self::Anthropic => write!(f, "anthropic"),
        }
    }
}

fn default_protocol() -> UpstreamProtocol { UpstreamProtocol::OpenAi }

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
    /// 协议类型: openai 或 anthropic，默认 openai
    #[serde(default = "default_protocol")]
    pub protocol: UpstreamProtocol,
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
            protocol: default_protocol(),
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

fn default_nlp_enabled() -> bool { false }  // 默认关闭，降低客户端 CPU 占用
fn default_nlp_language() -> String { "en".to_string() }
fn default_detection_primary_region() -> String { "cn".to_string() }
fn default_detection_additional_regions() -> Vec<String> { Vec::new() }

/// 检测区域配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DetectionRegion {
    /// 主区域代码 (e.g. "cn", "us", "eu", "gb", "sg", "jp", "kr")
    #[serde(default = "default_detection_primary_region")]
    pub primary_region: String,

    /// 额外启用的区域列表
    /// 例如主区域为 "cn"，额外启用 ["us", "eu"] 以检测 SSN、IBAN 等
    #[serde(default = "default_detection_additional_regions")]
    pub additional_regions: Vec<String>,
}

impl Default for DetectionRegion {
    fn default() -> Self {
        Self {
            primary_region: default_detection_primary_region(),
            additional_regions: default_detection_additional_regions(),
        }
    }
}

impl DetectionRegion {
    /// 获取所有启用的区域列表（主区域 + 额外区域），去重
    pub fn all_regions(&self) -> Vec<String> {
        let mut regions = vec![self.primary_region.clone()];
        for r in &self.additional_regions {
            if !regions.contains(r) {
                regions.push(r.clone());
            }
        }
        regions
    }

    /// 所有支持的区域列表
    pub fn available_regions() -> Vec<(&'static str, &'static str)> {
        vec![
            ("cn", "中国 (PIPL)"),
            ("us", "美国 (CCPA/HIPAA)"),
            ("eu", "欧盟 (GDPR)"),
            ("gb", "英国 (UK GDPR)"),
            ("sg", "新加坡 (PDPA)"),
            ("jp", "日本 (APPI)"),
            ("kr", "韩国 (PIPA)"),
        ]
    }
}

/// NLP/NER 子配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NlpConfig {
    /// 是否启用 NLP NER 检测（非结构化实体）
    /// 默认关闭以降低客户端 CPU 占用，用户可按需开启
    #[serde(default = "default_nlp_enabled")]
    pub enabled: bool,

    /// 默认语言 ("en" | "zh")
    #[serde(default = "default_nlp_language")]
    pub default_language: String,

    /// 模型缓存目录，留空使用系统默认
    #[serde(default)]
    pub cache_dir: Option<String>,
}

impl Default for NlpConfig {
    fn default() -> Self {
        Self {
            enabled: default_nlp_enabled(),
            default_language: default_nlp_language(),
            cache_dir: None,
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

    /// Active region for rule presets (e.g. "cn", "us", "eu", "gb", "global")
    /// Deprecated: use detection_region.primary_region instead
    #[serde(default = "default_region")]
    pub region: String,

    /// Enabled industry sub-presets within the region (e.g. ["medical", "finance"])
    /// Deprecated: industry sub-presets are no longer used in flat rule structure
    #[serde(default = "default_rule_industries")]
    pub rule_industries: Vec<String>,

    /// 检测区域配置（多区域支持）
    #[serde(default)]
    pub detection_region: DetectionRegion,

    /// NLP / NER configuration
    #[serde(default)]
    pub nlp: NlpConfig,
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
            region: default_region(),
            rule_industries: default_rule_industries(),
            detection_region: DetectionRegion::default(),
            nlp: NlpConfig::default(),
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

    /// Compute the list of rule preset names from region settings.
    ///
    /// Uses `detection_region` if configured, falls back to legacy `region` field.
    ///
    /// New flat structure: `rules/core.yaml` is always loaded as baseline,
    /// then `rules/{region}.yaml` for each enabled region.
    /// Falls back to legacy directory structure if flat files are not present.
    pub fn rule_presets(&self) -> Vec<String> {
        // Migrate legacy region field to detection_region if detection_region is at default
        let detection_region = if self.detection_region.primary_region == default_detection_primary_region()
            && self.detection_region.additional_regions.is_empty()
            && self.region != "global" && !self.region.is_empty()
        {
            tracing::warn!(
                "Config field `region` is deprecated, use `detection_region.primary_region` instead. Auto-migrating region='{}'",
                self.region
            );
            DetectionRegion {
                primary_region: self.region.clone(),
                additional_regions: Vec::new(),
            }
        } else {
            self.detection_region.clone()
        };

        let regions = detection_region.all_regions();

        // Check if the new flat structure exists
        let base = Path::new(&self.rules_dir);
        let flat_core = base.join("core.yaml");

        if flat_core.exists() {
            // New flat structure: core.yaml + region.yaml files
            let mut presets = vec!["core".to_string()];
            for region in &regions {
                if !region.is_empty() && region != "global" {
                    presets.push(region.clone());
                }
            }
            presets
        } else {
            // Legacy directory structure: global/ + region/ + region/industry/
            let mut presets = vec!["global".to_string()];
            for region in &regions {
                if !region.is_empty() && region != "global" {
                    presets.push(region.clone());
                    for industry in &self.rule_industries {
                        if !industry.is_empty() {
                            presets.push(format!("{}/{}", region, industry));
                        }
                    }
                }
            }
            presets
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
