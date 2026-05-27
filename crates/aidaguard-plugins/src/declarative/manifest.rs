use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub categories: Vec<String>,

    #[serde(default)]
    pub detect: DetectConfig,

    #[serde(default)]
    pub config: Option<FileConfig>,

    #[serde(default)]
    pub secondary_configs: Vec<FileConfig>,

    #[serde(default)]
    pub custom: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DetectConfig {
    #[serde(rename = "dir_exists")]
    DirExists { path: String },
    #[serde(rename = "file_exists")]
    FileExists { path: String },
    #[serde(rename = "any_file_exists")]
    AnyFileExists { paths: Vec<String> },
    #[serde(rename = "dir_has_prefix")]
    DirHasPrefix { dir: String, prefix: String },
    /// No detection — always considered installed (for built-in tools)
    #[serde(rename = "always")]
    Always,
}

impl Default for DetectConfig {
    fn default() -> Self {
        DetectConfig::Always
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileConfig {
    pub path: String,
    #[serde(default)]
    pub format: ConfigFormat,
    #[serde(default)]
    pub endpoint: Option<ReadWriteConfig>,
    #[serde(default)]
    pub model: Option<ReadConfig>,
    /// How to restore this file: "file" (replace from backup) or "remove_keys" (selectively remove keys)
    #[serde(default = "default_restore_mode")]
    pub restore_mode: RestoreMode,
}

fn default_restore_mode() -> RestoreMode {
    RestoreMode::File
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RestoreMode {
    File,
    RemoveKeys,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigFormat {
    #[default]
    Json,
    Toml,
    Yaml,
    Env,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadWriteConfig {
    /// Candidate JSON paths to read, tried in order
    #[serde(default)]
    pub read: Vec<String>,
    /// Write mapping: JSON path → value source
    #[serde(default)]
    pub write: HashMap<String, WriteValue>,
    /// Environment variable fallback for reading
    #[serde(default)]
    pub read_env_fallback: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadConfig {
    #[serde(default)]
    pub read: Vec<String>,
    #[serde(default)]
    pub read_env_fallback: Option<String>,
}

#[derive(Debug, Clone)]
pub enum WriteValue {
    ProxyUrl,
    Static(String),
    Delete,
}

impl WriteValue {
    pub fn resolve(&self, proxy_url: &str) -> Option<String> {
        match self {
            WriteValue::ProxyUrl => Some(proxy_url.to_string()),
            WriteValue::Static(s) => Some(s.clone()),
            WriteValue::Delete => None,
        }
    }

    pub fn is_proxy_url(&self) -> bool {
        matches!(self, WriteValue::ProxyUrl)
    }
}

impl<'de> Deserialize<'de> for WriteValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct WriteValueVisitor;
        impl<'de> serde::de::Visitor<'de> for WriteValueVisitor {
            type Value = WriteValue;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str(r#""proxyUrl", a fixed string, or "" for delete"#)
            }

            fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<WriteValue, E> {
                match s {
                    "proxyUrl" => Ok(WriteValue::ProxyUrl),
                    "" => Ok(WriteValue::Delete),
                    other => Ok(WriteValue::Static(other.to_string())),
                }
            }
        }
        deserializer.deserialize_str(WriteValueVisitor)
    }
}
