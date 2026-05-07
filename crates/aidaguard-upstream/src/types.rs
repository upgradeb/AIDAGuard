use serde::{Deserialize, Serialize};

/// LLM API protocol type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolType {
    OpenAiCompatible,
    AnthropicCompatible,
}

/// Authentication method for LLM providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthType {
    BearerToken,
    ApiKeyHeader { header: String },
}

/// Model information for a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub context: usize,
    #[serde(default)]
    pub max_output: usize,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// Provider configuration from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub protocol: ProtocolType,
    pub auth: AuthType,
    pub endpoint: String,
    #[serde(default)]
    pub models: Vec<ModelInfo>,
}

fn default_timeout() -> u64 {
    60
}

/// User-facing upstream configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UpstreamConfig {
    #[serde(flatten)]
    pub provider: ProviderConfig,
    pub api_key: Option<String>,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    #[serde(default)]
    pub rate_limit_qps: u32,
}
