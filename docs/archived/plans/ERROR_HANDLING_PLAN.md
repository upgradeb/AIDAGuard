# 错误处理增强规划

**工作项：** 3.5  
**优先级：** P2  
**工作量：** 2-3 天  
**依赖：** 无

---

## 一、目标

提供更友好、更结构化的错误处理：
- 统一错误类型体系
- 用户友好的错误消息
- 错误代码和恢复提示
- Tauri 前端错误展示

---

## 二、当前状态

### 2.1 现有错误类型

```rust
// aidaguard-core/src/error.rs

pub enum DetectionError {
    RuleCompilation(String),
    InvalidRegex { pattern: String, reason: String },
    RuleNotFound(String),
    Io(std::io::Error),
    ModelNotLoaded(String),
    EngineNotInitialized,
    Config(String),
}

pub enum StorageError {
    ConnectionFailed(String),
    EncryptionError(String),
    DecryptionError(String),
    InvalidKey,
    NotFound(String),
    MigrationFailed(String),
}

pub enum ProxyError {
    UpstreamConnection(String),
    Timeout(u64),
    InvalidBody(String),
    RateLimitExceeded,
    UpstreamError { status: u16, message: String },
}
```

### 2.2 问题分析

| 问题 | 说明 |
|------|------|
| 无统一错误类型 | 各模块错误独立，无法统一处理 |
| 错误链不完整 | 部分错误丢失上下文 |
| 缺少错误代码 | 前端无法根据代码做特定处理 |
| 存储实现混用 anyhow | `Storage` 使用 `anyhow::Error`，未用 `StorageError` |

---

## 三、设计方案

### 3.1 统一错误类型

```rust
// aidaguard-core/src/error.rs

use thiserror::Error;

/// AIDAGuard 统一错误类型
#[derive(Debug, Error)]
pub enum AidaGuardError {
    #[error("检测错误: {0}")]
    Detection(#[from] DetectionError),
    
    #[error("存储错误: {0}")]
    Storage(#[from] StorageError),
    
    #[error("代理错误: {0}")]
    Proxy(#[from] ProxyError),
    
    #[error("配置错误: {0}")]
    Config(#[from] ConfigError),
    
    #[error("插件错误: {0}")]
    Plugin(#[from] PluginError),
}
```

### 3.2 增强各错误类型

#### DetectionError 增强

```rust
#[derive(Debug, Error)]
pub enum DetectionError {
    // ── 规则相关 ──
    
    #[error("规则编译失败: {rule_id} - {reason}")]
    RuleCompilation {
        rule_id: String,
        reason: String,
    },
    
    #[error("无效正则 '{pattern}': {reason}")]
    InvalidRegex {
        pattern: String,
        reason: String,
    },
    
    #[error("规则未找到: {0}")]
    RuleNotFound(String),
    
    #[error("规则文件解析失败: {path} - {reason}")]
    RuleFileParse {
        path: String,
        reason: String,
    },
    
    // ── 引擎相关 ──
    
    #[error("检测引擎未初始化")]
    EngineNotInitialized,
    
    #[error("NLP 模型未加载: {language}")]
    ModelNotLoaded {
        language: String,
        hint: String,
    },
    
    #[error("检测超时: {duration_ms}ms")]
    DetectionTimeout {
        duration_ms: u64,
    },
    
    // ── IO 相关 ──
    
    #[error("文件 IO 错误: {0}")]
    Io(#[from] std::io::Error),
}
```

#### StorageError 增强

```rust
#[derive(Debug, Error)]
pub enum StorageError {
    // ── 连接相关 ──
    
    #[error("数据库连接失败: {path} - {reason}")]
    ConnectionFailed {
        path: String,
        reason: String,
    },
    
    #[error("数据库锁定超时")]
    DatabaseLocked,
    
    // ── 加密相关 ──
    
    #[error("加密失败: {reason}")]
    EncryptionFailed { reason: String },
    
    #[error("解密失败: 可能是密钥错误")]
    DecryptionFailed,
    
    #[error("无效的加密密钥")]
    InvalidKey,
    
    // ── 数据相关 ──
    
    #[error("记录未找到: {id}")]
    NotFound { id: String },
    
    #[error("数据库迁移失败: {reason}")]
    MigrationFailed { reason: String },
    
    // ── 配置相关 ──
    
    #[error("未知的存储类型: {type_name}")]
    UnknownType { type_name: String },
    
    #[error("缺少配置: {field}")]
    MissingConfig { field: String },
    
    // ── 容量相关 ──
    
    #[error("文档过大: {size} bytes (最大 {max_size} bytes)")]
    TooLarge { size: usize, max_size: usize },
}
```

#### ProxyError 增强

```rust
#[derive(Debug, Error)]
pub enum ProxyError {
    // ── 上游相关 ──
    
    #[error("上游连接失败: {url} - {reason}")]
    UpstreamConnection {
        url: String,
        reason: String,
    },
    
    #[error("上游超时: {duration_secs}s")]
    UpstreamTimeout { duration_secs: u64 },
    
    #[error("上游返回错误 {status}: {message}")]
    UpstreamError {
        status: u16,
        message: String,
    },
    
    // ── 请求相关 ──
    
    #[error("无效的请求体: {reason}")]
    InvalidBody { reason: String },
    
    #[error("请求速率限制")]
    RateLimitExceeded,
    
    // ── 检测相关 ──
    
    #[error("检测到敏感数据: {count} 处 [{types}]")]
    SensitiveDataDetected {
        count: usize,
        types: String,
    },
    
    // ── 流处理相关 ──
    
    #[error("流处理错误: {reason}")]
    StreamError { reason: String },
}
```

#### ConfigError 新增

```rust
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("配置文件不存在: {path}")]
    FileNotFound { path: String },
    
    #[error("配置文件解析失败: {path} - {reason}")]
    ParseFailed {
        path: String,
        reason: String,
    },
    
    #[error("缺少必填配置项: {field}")]
    MissingField { field: String },
    
    #[error("无效的配置值: {field} = {value} ({reason})")]
    InvalidValue {
        field: String,
        value: String,
        reason: String,
    },
}
```

#### PluginError 新增

```rust
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("插件加载失败: {id} - {reason}")]
    LoadFailed { id: String, reason: String },
    
    #[error("插件未找到: {id}")]
    NotFound { id: String },
    
    #[error("插件配置失败: {id} - {reason}")]
    ConfigureFailed { id: String, reason: String },
    
    #[error("插件恢复失败: {id}")]
    RestoreFailed { id: String },
    
    #[error("ABI 版本不匹配: 期望 {expected}, 实际 {actual}")]
    AbiMismatch { expected: u32, actual: u32 },
    
    #[error("插件库文件未找到: {dir}")]
    LibraryNotFound { dir: String },
    
    #[error("插件签名验证失败: {id}")]
    InvalidSignature { id: String },
}
```

### 3.3 错误代码体系

```rust
/// 错误代码
///
/// 前端可根据代码执行特定处理逻辑
impl AidaGuardError {
    pub fn code(&self) -> &'static str {
        match self {
            // 检测错误
            Self::Detection(DetectionError::RuleCompilation { .. }) => "DET_001",
            Self::Detection(DetectionError::InvalidRegex { .. }) => "DET_002",
            Self::Detection(DetectionError::RuleNotFound(_)) => "DET_003",
            Self::Detection(DetectionError::EngineNotInitialized) => "DET_004",
            Self::Detection(DetectionError::ModelNotLoaded { .. }) => "DET_005",
            Self::Detection(DetectionError::DetectionTimeout { .. }) => "DET_006",
            
            // 存储错误
            Self::Storage(StorageError::ConnectionFailed { .. }) => "STO_001",
            Self::Storage(StorageError::EncryptionFailed { .. }) => "STO_002",
            Self::Storage(StorageError::DecryptionFailed) => "STO_003",
            Self::Storage(StorageError::InvalidKey) => "STO_004",
            Self::Storage(StorageError::NotFound { .. }) => "STO_005",
            
            // 代理错误
            Self::Proxy(ProxyError::UpstreamConnection { .. }) => "PRX_001",
            Self::Proxy(ProxyError::UpstreamTimeout { .. }) => "PRX_002",
            Self::Proxy(ProxyError::RateLimitExceeded) => "PRX_003",
            Self::Proxy(ProxyError::SensitiveDataDetected { .. }) => "PRX_004",
            
            // 配置错误
            Self::Config(ConfigError::FileNotFound { .. }) => "CFG_001",
            Self::Config(ConfigError::ParseFailed { .. }) => "CFG_002",
            Self::Config(ConfigError::MissingField { .. }) => "CFG_003",
            
            // 插件错误
            Self::Plugin(PluginError::LoadFailed { .. }) => "PLG_001",
            Self::Plugin(PluginError::NotFound { .. }) => "PLG_002",
        }
    }
}
```

### 3.4 用户友好消息

```rust
impl AidaGuardError {
    /// 获取用户友好的错误消息（中文）
    pub fn user_message(&self) -> String {
        match self {
            // 检测错误
            Self::Detection(DetectionError::RuleCompilation { rule_id, reason }) => {
                format!("⚠️ 规则 [{}] 编译失败: {}\n请检查规则文件格式。", rule_id, reason)
            }
            Self::Detection(DetectionError::ModelNotLoaded { language, hint }) => {
                format!("⚠️ NLP 模型未加载（语言: {}）\n{}", language, hint)
            }
            Self::Detection(DetectionError::DetectionTimeout { duration_ms }) => {
                format!("⚠️ 检测超时（{}ms），文本可能过长。", duration_ms)
            }
            
            // 存储错误
            Self::Storage(StorageError::ConnectionFailed { path, reason }) => {
                format!("⚠️ 数据库连接失败: {}\n原因: {}\n请检查数据库路径权限。", path, reason)
            }
            Self::Storage(StorageError::DecryptionFailed) => {
                "⚠️ 数据解密失败，可能是加密密钥错误。\n请检查配置中的加密密钥。".into()
            }
            
            // 代理错误
            Self::Proxy(ProxyError::UpstreamConnection { url, reason }) => {
                format!("⚠️ 无法连接到上游服务: {}\n原因: {}\n请检查网络连接和上游地址。", url, reason)
            }
            Self::Proxy(ProxyError::SensitiveDataDetected { count, types }) => {
                format!("⚠️ 检测到 {} 处敏感数据 [{}]\n建议检查后再发送。", count, types)
            }
            
            // 配置错误
            Self::Config(ConfigError::ParseFailed { path, reason }) => {
                format!("⚠️ 配置文件解析失败: {}\n原因: {}\n请检查 TOML 格式。", path, reason)
            }
            
            // 插件错误
            Self::Plugin(PluginError::LoadFailed { id, reason }) => {
                format!("⚠️ 插件 [{}] 加载失败: {}", id, reason)
            }
            
            // 默认
            _ => format!("⚠️ 错误: {}", self),
        }
    }
    
    /// 获取恢复提示
    pub fn recovery_hint(&self) -> &'static str {
        match self {
            Self::Detection(DetectionError::RuleCompilation { .. }) => 
                "检查 YAML 语法和必填字段",
            Self::Detection(DetectionError::ModelNotLoaded { .. }) => 
                "启用 nlp feature 并确保网络连接",
            Self::Storage(StorageError::ConnectionFailed { .. }) => 
                "检查数据库路径权限和磁盘空间",
            Self::Storage(StorageError::DecryptionFailed) => 
                "验证加密密钥是否正确",
            Self::Proxy(ProxyError::UpstreamConnection { .. }) => 
                "检查网络连接和上游 URL",
            Self::Config(ConfigError::ParseFailed { .. }) => 
                "检查配置文件格式",
            _ => "查看日志获取详细信息",
        }
    }
}
```

### 3.5 Tauri 错误序列化

```rust
/// 前端错误响应
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    /// 错误代码
    pub code: String,
    /// 错误消息
    pub message: String,
    /// 恢复提示
    pub recovery_hint: String,
    /// 错误详情（可选）
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
```

---

## 四、Storage 模块错误迁移

### 4.1 当前问题

```rust
// aidaguard-storage/src/lib.rs

// 当前使用 anyhow::Error
pub fn record(&self, ...) -> Result<(), anyhow::Error> { ... }
pub fn list(&self, ...) -> Result<Vec<DetectionRecord>, anyhow::Error> { ... }
```

### 4.2 迁移后

```rust
// aidaguard-storage/src/sqlite.rs

use aidaguard_core::error::StorageError;

impl AuditStorage for SqliteStorage {
    fn record(&self, ...) -> Result<(), StorageError> {
        // ...
        Ok(())
    }
    
    fn list(&self, limit: usize, offset: usize) -> Result<Vec<DetectionRecord>, StorageError> {
        // ...
    }
}
```

---

## 五、文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `aidaguard-core/src/error.rs` | 重写 | 统一错误类型 + 代码 + 消息 |
| `aidaguard-storage/src/lib.rs` | 修改 | anyhow → StorageError |
| `aidaguard-proxy/src/server.rs` | 修改 | 使用统一错误类型 |
| `aidaguard-tauri/src-tauri/src/main.rs` | 修改 | ErrorResponse 序列化 |

---

## 六、验收标准

- [ ] 统一错误类型定义完整
- [ ] 错误代码覆盖所有变体
- [ ] 用户友好消息完整
- [ ] 恢复提示完整
- [ ] Storage 模块迁移完成
- [ ] Tauri 前端可展示错误
