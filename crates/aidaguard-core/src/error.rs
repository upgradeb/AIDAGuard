//! Structured error types for Aidaguard.
//!
//! Provides detailed error information with recovery hints.

/// Detection-related errors.
#[derive(Debug, thiserror::Error)]
pub enum DetectionError {
    #[error("Rule compilation failed: {0}")]
    RuleCompilation(String),

    #[error("Invalid regex pattern '{pattern}': {reason}")]
    InvalidRegex { pattern: String, reason: String },

    #[error("Rule not found: {0}")]
    RuleNotFound(String),

    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("NLP model not loaded for language: {0}")]
    ModelNotLoaded(String),

    #[error("Detection engine not initialized")]
    EngineNotInitialized,

    #[error("Configuration error: {0}")]
    Config(String),
}

impl DetectionError {
    /// Get a recovery hint for this error.
    pub fn recovery_hint(&self) -> &'static str {
        match self {
            Self::RuleCompilation(_) => "Check YAML syntax and ensure all required fields are present",
            Self::InvalidRegex { .. } => "Use a regex tester to validate the pattern syntax",
            Self::RuleNotFound(_) => "Verify the rule ID exists in the loaded rules",
            Self::Io(_) => "Check file permissions and disk space",
            Self::ModelNotLoaded(_) => "Enable 'nlp' feature and ensure network access for model download",
            Self::EngineNotInitialized => "Call AnalyzerEngine::builder().build() before detection",
            Self::Config(_) => "Check config.toml syntax and required fields",
        }
    }
}

/// Storage-related errors.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Database connection failed: {path} - {reason}")]
    ConnectionFailed { path: String, reason: String },

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
}

impl StorageError {
    /// Get a recovery hint for this error.
    pub fn recovery_hint(&self) -> &'static str {
        match self {
            Self::ConnectionFailed { .. } => "Check database path permissions and disk space",
            Self::EncryptionError(_) | Self::DecryptionError(_) => "Verify encryption key is correct",
            Self::InvalidKey => "Provide a non-empty encryption key",
            Self::NotFound(_) => "The requested record may have been deleted",
            Self::MigrationFailed(_) => "Backup database and check schema compatibility",
            Self::UnknownType { .. } => "Use a supported storage type: sqlite, memory",
        }
    }
}

/// Proxy-related errors.
#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("Upstream connection failed: {0}")]
    UpstreamConnection(String),

    #[error("Request timeout after {0} seconds")]
    Timeout(u64),

    #[error("Invalid request body: {0}")]
    InvalidBody(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Upstream returned error {status}: {message}")]
    UpstreamError { status: u16, message: String },
}

impl ProxyError {
    /// Get a recovery hint for this error.
    pub fn recovery_hint(&self) -> &'static str {
        match self {
            Self::UpstreamConnection(_) => "Check network connectivity and upstream URL",
            Self::Timeout(_) => "Increase timeout or check upstream service health",
            Self::InvalidBody(_) => "Verify request body format matches API specification",
            Self::RateLimitExceeded => "Wait before retrying or upgrade API plan",
            Self::UpstreamError { .. } => "Check upstream service logs for details",
        }
    }
}
