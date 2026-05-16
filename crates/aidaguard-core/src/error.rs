//! Structured error types for AIDAGuard.
//!
//! Provides detailed error information with error codes, user-friendly messages,
//! and recovery hints for the Tauri frontend.

use serde::Serialize;

// ── Detection Errors ──

/// Detection-related errors.
#[derive(Debug, thiserror::Error)]
pub enum DetectionError {
    #[error("Rule compilation failed: {0}")]
    RuleCompilation(String),

    #[error("Invalid regex pattern '{pattern}': {reason}")]
    InvalidRegex { pattern: String, reason: String },

    #[error("Rule not found: {0}")]
    RuleNotFound(String),

    #[error("Rule file parse failed: {path} - {reason}")]
    RuleFileParse { path: String, reason: String },

    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("NLP model not loaded for language: {0}")]
    ModelNotLoaded(String),

    #[error("Detection engine not initialized")]
    EngineNotInitialized,

    #[error("Detection timeout: {duration_ms}ms")]
    DetectionTimeout { duration_ms: u64 },

    #[error("Configuration error: {0}")]
    Config(String),
}

impl DetectionError {
    /// Get error code for frontend.
    pub fn code(&self) -> &'static str {
        match self {
            Self::RuleCompilation(_) => "DET_001",
            Self::InvalidRegex { .. } => "DET_002",
            Self::RuleNotFound(_) => "DET_003",
            Self::RuleFileParse { .. } => "DET_004",
            Self::Io(_) => "DET_005",
            Self::ModelNotLoaded(_) => "DET_006",
            Self::EngineNotInitialized => "DET_007",
            Self::DetectionTimeout { .. } => "DET_008",
            Self::Config(_) => "DET_009",
        }
    }

    /// Get a recovery hint for this error.
    pub fn recovery_hint(&self) -> &'static str {
        match self {
            Self::RuleCompilation(_) => "检查 YAML 语法和必填字段",
            Self::InvalidRegex { .. } => "使用正则测试工具验证模式语法",
            Self::RuleNotFound(_) => "验证规则 ID 存在于已加载的规则中",
            Self::RuleFileParse { .. } => "检查规则文件格式和路径",
            Self::Io(_) => "检查文件权限和磁盘空间",
            Self::ModelNotLoaded(_) => "启用 'nlp' feature 并确保网络连接以下载模型",
            Self::EngineNotInitialized => "在使用前调用 AnalyzerEngine::builder().build()",
            Self::DetectionTimeout { .. } => "文本可能过长，考虑分段检测",
            Self::Config(_) => "检查 config.toml 语法和必填字段",
        }
    }
}

// ── Storage Errors ──

/// Storage-related errors.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Database connection failed: {path} - {reason}")]
    ConnectionFailed { path: String, reason: String },

    #[error("Database locked timeout")]
    DatabaseLocked,

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("Invalid encryption key")]
    InvalidKey,

    #[error("Record not found: {0}")]
    NotFound(String),

    #[error("Database migration failed: {0}")]
    MigrationFailed(String),

    #[error("Unknown storage type: {type_name}")]
    UnknownType { type_name: String },

    #[error("Missing config: {field}")]
    MissingConfig { field: String },

    #[error("Document too large: {size} bytes (max: {max_size} bytes)")]
    TooLarge { size: usize, max_size: usize },
}

impl StorageError {
    /// Get error code for frontend.
    pub fn code(&self) -> &'static str {
        match self {
            Self::ConnectionFailed { .. } => "STO_001",
            Self::DatabaseLocked => "STO_002",
            Self::EncryptionError(_) => "STO_003",
            Self::DecryptionError(_) => "STO_004",
            Self::InvalidKey => "STO_005",
            Self::NotFound(_) => "STO_006",
            Self::MigrationFailed(_) => "STO_007",
            Self::UnknownType { .. } => "STO_008",
            Self::MissingConfig { .. } => "STO_009",
            Self::TooLarge { .. } => "STO_010",
        }
    }

    /// Get a recovery hint for this error.
    pub fn recovery_hint(&self) -> &'static str {
        match self {
            Self::ConnectionFailed { .. } => "检查数据库路径权限和磁盘空间",
            Self::DatabaseLocked => "等待其他操作完成或重启应用",
            Self::EncryptionError(_) | Self::DecryptionError(_) => "验证加密密钥是否正确",
            Self::InvalidKey => "提供非空的加密密钥",
            Self::NotFound(_) => "请求的记录可能已被删除",
            Self::MigrationFailed(_) => "备份数据库并检查模式兼容性",
            Self::UnknownType { .. } => "使用支持的存储类型: sqlite, memory",
            Self::MissingConfig { .. } => "检查配置文件中的必填字段",
            Self::TooLarge { .. } => "减小文档大小或增加配置的最大限制",
        }
    }
}

// ── Proxy Errors ──

/// Proxy-related errors.
#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("Upstream connection failed: {url} - {reason}")]
    UpstreamConnection { url: String, reason: String },

    #[error("Request timeout after {duration_secs} seconds")]
    Timeout { duration_secs: u64 },

    #[error("Invalid request body: {reason}")]
    InvalidBody { reason: String },

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Upstream returned error {status}: {message}")]
    UpstreamError { status: u16, message: String },

    #[error("Sensitive data detected: {count} occurrences [{types}]")]
    SensitiveDataDetected { count: usize, types: String },

    #[error("Stream processing error: {reason}")]
    StreamError { reason: String },
}

impl ProxyError {
    /// Get error code for frontend.
    pub fn code(&self) -> &'static str {
        match self {
            Self::UpstreamConnection { .. } => "PRX_001",
            Self::Timeout { .. } => "PRX_002",
            Self::InvalidBody { .. } => "PRX_003",
            Self::RateLimitExceeded => "PRX_004",
            Self::UpstreamError { .. } => "PRX_005",
            Self::SensitiveDataDetected { .. } => "PRX_006",
            Self::StreamError { .. } => "PRX_007",
        }
    }

    /// Get a recovery hint for this error.
    pub fn recovery_hint(&self) -> &'static str {
        match self {
            Self::UpstreamConnection { .. } => "检查网络连接和上游 URL",
            Self::Timeout { .. } => "增加超时时间或检查上游服务状态",
            Self::InvalidBody { .. } => "验证请求体格式是否符合 API 规范",
            Self::RateLimitExceeded => "等待后重试或升级 API 计划",
            Self::UpstreamError { .. } => "检查上游服务日志获取详情",
            Self::SensitiveDataDetected { .. } => "检查敏感数据检测结果，确认是否需要过滤",
            Self::StreamError { .. } => "检查流处理配置和网络连接",
        }
    }
}

// ── Config Errors ──

/// Configuration-related errors.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Config file not found: {path}")]
    FileNotFound { path: String },

    #[error("Config parse failed: {path} - {reason}")]
    ParseFailed { path: String, reason: String },

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Invalid config value: {field} = {value} ({reason})")]
    InvalidValue { field: String, value: String, reason: String },
}

impl ConfigError {
    /// Get error code for frontend.
    pub fn code(&self) -> &'static str {
        match self {
            Self::FileNotFound { .. } => "CFG_001",
            Self::ParseFailed { .. } => "CFG_002",
            Self::MissingField { .. } => "CFG_003",
            Self::InvalidValue { .. } => "CFG_004",
        }
    }

    /// Get a recovery hint for this error.
    pub fn recovery_hint(&self) -> &'static str {
        match self {
            Self::FileNotFound { .. } => "创建配置文件或检查路径",
            Self::ParseFailed { .. } => "检查 TOML 格式语法",
            Self::MissingField { .. } => "在配置文件中添加必填字段",
            Self::InvalidValue { .. } => "修正配置值到有效范围",
        }
    }
}

// ── Plugin Errors ──

/// Plugin-related errors.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin load failed: {id} - {reason}")]
    LoadFailed { id: String, reason: String },

    #[error("Plugin not found: {id}")]
    NotFound { id: String },

    #[error("Plugin configure failed: {id} - {reason}")]
    ConfigureFailed { id: String, reason: String },

    #[error("Plugin restore failed: {id}")]
    RestoreFailed { id: String },

    #[error("ABI version mismatch: expected {expected}, got {actual}")]
    AbiMismatch { expected: u32, actual: u32 },

    #[error("Plugin library not found: {dir}")]
    LibraryNotFound { dir: String },

    #[error("Invalid plugin signature: {id}")]
    InvalidSignature { id: String },

    #[error("Invalid manifest: {reason}")]
    InvalidManifest { reason: String },

    #[error("Plugin initialization failed: {id}")]
    InitFailed { id: String },
}

impl PluginError {
    /// Get error code for frontend.
    pub fn code(&self) -> &'static str {
        match self {
            Self::LoadFailed { .. } => "PLG_001",
            Self::NotFound { .. } => "PLG_002",
            Self::ConfigureFailed { .. } => "PLG_003",
            Self::RestoreFailed { .. } => "PLG_004",
            Self::AbiMismatch { .. } => "PLG_005",
            Self::LibraryNotFound { .. } => "PLG_006",
            Self::InvalidSignature { .. } => "PLG_007",
            Self::InvalidManifest { .. } => "PLG_008",
            Self::InitFailed { .. } => "PLG_009",
        }
    }

    /// Get a recovery hint for this error.
    pub fn recovery_hint(&self) -> &'static str {
        match self {
            Self::LoadFailed { .. } => "检查插件文件是否损坏",
            Self::NotFound { .. } => "验证插件 ID 是否正确",
            Self::ConfigureFailed { .. } => "检查插件配置权限",
            Self::RestoreFailed { .. } => "检查备份文件是否存在",
            Self::AbiMismatch { .. } => "使用与当前版本兼容的插件",
            Self::LibraryNotFound { .. } => "检查插件目录中是否存在库文件",
            Self::InvalidSignature { .. } => "使用可信来源的插件",
            Self::InvalidManifest { .. } => "检查 manifest.json 格式",
            Self::InitFailed { .. } => "检查插件初始化依赖",
        }
    }
}

// ── Unified Error Type ──

/// AIDAGuard unified error type.
#[derive(Debug, thiserror::Error)]
pub enum AidaGuardError {
    #[error("Detection error: {0}")]
    Detection(#[from] DetectionError),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Proxy error: {0}")]
    Proxy(#[from] ProxyError),

    #[error("Config error: {0}")]
    Config(#[from] ConfigError),

    #[error("Plugin error: {0}")]
    Plugin(#[from] PluginError),
}

impl AidaGuardError {
    /// Get error code for frontend.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Detection(e) => e.code(),
            Self::Storage(e) => e.code(),
            Self::Proxy(e) => e.code(),
            Self::Config(e) => e.code(),
            Self::Plugin(e) => e.code(),
        }
    }

    /// Get user-friendly message (Chinese).
    pub fn user_message(&self) -> String {
        match self {
            Self::Detection(e) => format!("⚠️ 检测错误: {}", e),
            Self::Storage(e) => format!("⚠️ 存储错误: {}", e),
            Self::Proxy(e) => format!("⚠️ 代理错误: {}", e),
            Self::Config(e) => format!("⚠️ 配置错误: {}", e),
            Self::Plugin(e) => format!("⚠️ 插件错误: {}", e),
        }
    }

    /// Get recovery hint.
    pub fn recovery_hint(&self) -> &'static str {
        match self {
            Self::Detection(e) => e.recovery_hint(),
            Self::Storage(e) => e.recovery_hint(),
            Self::Proxy(e) => e.recovery_hint(),
            Self::Config(e) => e.recovery_hint(),
            Self::Plugin(e) => e.recovery_hint(),
        }
    }
}

// ── Frontend Error Response ──

/// Error response for Tauri frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    /// Error code
    pub code: String,
    /// User-friendly message
    pub message: String,
    /// Recovery hint
    pub recovery_hint: String,
    /// Detailed error (optional)
    pub details: Option<String>,
}

impl From<&AidaGuardError> for ErrorResponse {
    fn from(err: &AidaGuardError) -> Self {
        Self {
            code: err.code().to_string(),
            message: err.user_message(),
            recovery_hint: err.recovery_hint().to_string(),
            details: Some(err.to_string()),
        }
    }
}

impl From<&DetectionError> for ErrorResponse {
    fn from(err: &DetectionError) -> Self {
        Self {
            code: err.code().to_string(),
            message: format!("⚠️ 检测错误: {}", err),
            recovery_hint: err.recovery_hint().to_string(),
            details: Some(err.to_string()),
        }
    }
}

impl From<&StorageError> for ErrorResponse {
    fn from(err: &StorageError) -> Self {
        Self {
            code: err.code().to_string(),
            message: format!("⚠️ 存储错误: {}", err),
            recovery_hint: err.recovery_hint().to_string(),
            details: Some(err.to_string()),
        }
    }
}

impl From<&ProxyError> for ErrorResponse {
    fn from(err: &ProxyError) -> Self {
        Self {
            code: err.code().to_string(),
            message: format!("⚠️ 代理错误: {}", err),
            recovery_hint: err.recovery_hint().to_string(),
            details: Some(err.to_string()),
        }
    }
}
