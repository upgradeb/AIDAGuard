# AIDAGuard Phase 3 - 细化工作计划

**版本：** v0.5.0 目标  
**日期：** 2026-05-16  
**状态：** 待实施

---

## 一、Phase 3 工作项总览

| 编号 | 工作项 | 优先级 | 工作量 | 依赖 | 开始时间 |
|------|--------|--------|--------|------|----------|
| 3.1 | 依赖关系重构 | P2 | 2-3 天 | 无 | Week 1 |
| 3.2 | AuditStorage trait 抽象 | P2 | 3-5 天 | 3.1 | Week 1-2 |
| 3.3 | Base64 文档拦截 | P1 | 3-5 天 | 无 | Week 1 |
| 3.4 | PDF 文本提取 | P1 | 2-3 天 | 3.3 | Week 2 |
| 3.5 | Word/Excel 文本提取 | P1 | 3-4 天 | 3.3 | Week 2-3 |
| 3.6 | 文档脱敏重建 | P2 | 5-7 天 | 3.4, 3.5 | Week 3-4 |
| 3.7 | 插件动态加载 | P3 | 5-7 天 | 3.1 | Week 4+ |

**总工作量：** 23-34 天（约 4-5 周）

---

## 二、详细工作分解

### 3.1 依赖关系重构

**优先级：** P2  
**工作量：** 2-3 天  
**负责人：** -  
**状态：** 待开始

#### 背景

当前 `aidaguard-core` 重新导出 `aidaguard-storage`，导致反向依赖：

```rust
// aidaguard-core/src/storage/mod.rs
pub use aidaguard_storage::*;  // ← 反向依赖
```

这违背了 core 作为基础层的设计原则。

#### 目标

```
重构前：
aidaguard-core → aidaguard-storage (反向依赖)

重构后：
aidaguard-core (纯基础层，定义 trait)
    ↑
aidaguard-storage (实现 trait)
```

#### 实现步骤

**Step 1: 定义 AuditStorage trait**（0.5 天）

```rust
// aidaguard-core/src/storage.rs (新文件)
use async_trait::async_trait;
use crate::entity::DetectionRecord;
use crate::error::StorageError;

/// 审计存储接口
#[async_trait]
pub trait AuditStorage: Send + Sync {
    /// 记录单条检测结果
    async fn record(&self, record: &DetectionRecord) -> Result<(), StorageError>;
    
    /// 批量记录
    async fn batch_record(&self, records: &[DetectionRecord]) -> Result<usize, StorageError>;
    
    /// 分页查询
    async fn list(&self, limit: usize, offset: usize) -> Result<Vec<DetectionRecord>, StorageError>;
    
    /// 条件查询
    async fn list_filtered(
        &self,
        limit: usize,
        offset: usize,
        filter: AuditFilter,
    ) -> Result<Vec<DetectionRecord>, StorageError>;
    
    /// 统计信息
    async fn stats(&self) -> Result<AuditStats, StorageError>;
    
    /// 删除记录
    async fn delete(&self, id: &str) -> Result<(), StorageError>;
    
    /// 清理过期记录
    async fn purge_before(&self, timestamp_ms: i64) -> Result<usize, StorageError>;
}

/// 查询过滤器
#[derive(Debug, Clone, Default)]
pub struct AuditFilter {
    pub rule_id: Option<String>,
    pub path: Option<String>,
    pub date_from_ms: Option<i64>,
    pub date_to_ms: Option<i64>,
    pub strategy: Option<String>,
}

/// 审计统计
#[derive(Debug, Clone)]
pub struct AuditStats {
    pub total_records: usize,
    pub records_by_rule: HashMap<String, usize>,
    pub records_by_strategy: HashMap<String, usize>,
    pub earliest_timestamp_ms: Option<i64>,
    pub latest_timestamp_ms: Option<i64>,
}
```

**Step 2: 移除反向依赖**（0.5 天）

```rust
// 删除 aidaguard-core/src/storage/mod.rs 的 re-export
// 改为：
// aidaguard-core/src/storage.rs
mod storage;
pub use storage::{AuditStorage, AuditFilter, AuditStats};
```

**Step 3: 实现 trait**（0.5 天）

```rust
// aidaguard-storage/src/lib.rs
use aidaguard_core::storage::{AuditStorage, AuditFilter, AuditStats};
use async_trait::async_trait;

pub struct SqliteStorage {
    conn: rusqlite::Connection,
}

#[async_trait]
impl AuditStorage for SqliteStorage {
    async fn record(&self, record: &DetectionRecord) -> Result<(), StorageError> {
        // 现有实现
    }
    
    // ... 其他方法
}
```

**Step 4: 更新依赖方**（0.5 天）

```rust
// aidaguard-proxy/src/server.rs
// 修改前：
use aidaguard_core::storage::SqliteStorage;

// 修改后：
use aidaguard_storage::SqliteStorage;
use aidaguard_core::storage::AuditStorage;
```

#### 验收标准

- [ ] `aidaguard-core` 不再依赖 `aidaguard-storage`
- [ ] `cargo tree` 显示依赖图清晰，无反向依赖
- [ ] 所有测试通过
- [ ] 编译时间不增加

#### 文件变更

| 文件 | 操作 |
|------|------|
| `aidaguard-core/src/storage.rs` | 新增 |
| `aidaguard-core/src/storage/mod.rs` | 删除 |
| `aidaguard-core/Cargo.toml` | 移除 aidaguard-storage 依赖 |
| `aidaguard-storage/src/lib.rs` | 实现 trait |
| `aidaguard-proxy/src/server.rs` | 更新 import |
| `aidaguard-tauri/src-tauri/src/main.rs` | 更新 import |

---

### 3.2 AuditStorage trait 抽象

**优先级：** P2  
**工作量：** 3-5 天  
**依赖：** 3.1  
**状态：** 待开始

#### 目标

支持可插拔的存储后端，为未来扩展做准备：
- SQLite（当前）
- PostgreSQL（企业部署）
- Memory（测试）
- S3/云存储（归档）

#### 实现步骤

**Step 1: 完善 trait 定义**（1 天）

```rust
// aidaguard-core/src/storage.rs

/// 存储配置
#[derive(Debug, Clone)]
pub enum StorageConfig {
    Sqlite { path: PathBuf },
    Postgres { url: String },
    Memory,
    S3 { bucket: String, prefix: String },
}

/// 存储工厂
pub struct StorageFactory;

impl StorageFactory {
    pub async fn create(config: StorageConfig) -> Result<Box<dyn AuditStorage>, StorageError> {
        match config {
            StorageConfig::Sqlite { path } => {
                Ok(Box::new(SqliteStorage::new(path).await?))
            }
            StorageConfig::Postgres { url } => {
                #[cfg(feature = "postgres")]
                {
                    Ok(Box::new(PostgresStorage::new(&url).await?))
                }
                #[cfg(not(feature = "postgres"))]
                Err(StorageError::FeatureNotEnabled("postgres"))
            }
            StorageConfig::Memory => {
                Ok(Box::new(MemoryStorage::new()))
            }
            StorageConfig::S3 { bucket, prefix } => {
                #[cfg(feature = "s3")]
                {
                    Ok(Box::new(S3Storage::new(bucket, prefix).await?))
                }
                #[cfg(not(feature = "s3"))]
                Err(StorageError::FeatureNotEnabled("s3"))
            }
        }
    }
}
```

**Step 2: 实现 MemoryStorage**（1 天）

```rust
// aidaguard-storage/src/memory.rs
use std::sync::Arc;
use tokio::sync::RwLock;
use aidaguard_core::storage::{AuditStorage, AuditFilter, AuditStats};

pub struct MemoryStorage {
    records: Arc<RwLock<Vec<DetectionRecord>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl AuditStorage for MemoryStorage {
    async fn record(&self, record: &DetectionRecord) -> Result<(), StorageError> {
        let mut records = self.records.write().await;
        records.push(record.clone());
        Ok(())
    }
    
    // ... 其他方法
}
```

**Step 3: 添加配置支持**（1 天）

```rust
// aidaguard-core/src/config.rs

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "default_storage_type")]
    pub storage_type: String,  // "sqlite" | "postgres" | "memory"
    
    #[serde(default = "default_sqlite_path")]
    pub sqlite_path: PathBuf,
    
    pub postgres_url: Option<String>,
}

fn default_storage_type() -> String { "sqlite".into() }
fn default_sqlite_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("aidaguard")
        .join("audit.db")
}
```

**Step 4: 单元测试**（1 天）

```rust
// tests/storage_test.rs

#[tokio::test]
async fn test_memory_storage() {
    let storage = MemoryStorage::new();
    let record = DetectionRecord::default();
    
    storage.record(&record).await.unwrap();
    let records = storage.list(10, 0).await.unwrap();
    
    assert_eq!(records.len(), 1);
}

#[tokio::test]
async fn test_sqlite_storage() {
    let storage = SqliteStorage::new_in_memory().await.unwrap();
    // ... 测试所有方法
}
```

#### 验收标准

- [ ] trait 定义完整，所有方法有文档
- [ ] MemoryStorage 实现完整
- [ ] SqliteStorage 实现完整
- [ ] 单元测试覆盖率 > 80%
- [ ] 配置文件支持存储类型切换

---

### 3.3 Base64 文档拦截

**优先级：** P1  
**工作量：** 3-5 天  
**依赖：** 无  
**状态：** 待开始

#### 背景

Web AI（ChatGPT、Claude Web）用户可能上传文档，文档以 Base64 编码发送：

```json
{
  "messages": [
    {
      "role": "user",
      "content": [
        {
          "type": "text",
          "text": "分析这份合同"
        },
        {
          "type": "image_url",
          "image_url": {
            "url": "data:application/pdf;base64,JVBERi0xLjQK..."
          }
        }
      ]
    }
  ]
}
```

#### 目标

拦截 Base64 编码的文档，提取文本进行敏感数据检测。

#### 实现步骤

**Step 1: 定义文档拦截器**（1 天）

```rust
// crates/aidaguard-proxy/src/document_interceptor.rs

use base64::{Engine as _, engine::general_purpose};
use serde_json::Value;

/// 文档类型
#[derive(Debug, Clone, PartialEq)]
pub enum DocumentType {
    Pdf,
    Word,
    Excel,
    PowerPoint,
    Image,
    Unknown,
}

/// Base64 文档拦截器
pub struct DocumentInterceptor {
    detector: Arc<AnalyzerEngine>,
    config: DocumentInterceptorConfig,
}

#[derive(Debug, Clone)]
pub struct DocumentInterceptorConfig {
    /// 是否启用文档检测
    pub enabled: bool,
    /// 最大文档大小（字节）
    pub max_size: usize,
    /// 是否脱敏 PDF
    pub sanitize_pdf: bool,
    /// 是否脱敏 Word
    pub sanitize_word: bool,
    /// 是否脱敏 Excel
    pub sanitize_excel: bool,
}

impl Default for DocumentInterceptorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_size: 10 * 1024 * 1024, // 10MB
            sanitize_pdf: false,  // PDF 脱敏复杂，默认关闭
            sanitize_word: true,
            sanitize_excel: true,
        }
    }
}
```

**Step 2: 实现 Base64 解析**（1 天）

```rust
impl DocumentInterceptor {
    /// 检测并处理请求中的 Base64 文档
    pub async fn intercept_request(
        &self,
        request_body: &mut Value,
    ) -> Result<InterceptResult, ProxyError> {
        if !self.config.enabled {
            return Ok(InterceptResult::Unchanged);
        }
        
        // 遍历 messages
        if let Some(messages) = request_body.get_mut("messages").and_then(|m| m.as_array_mut()) {
            for message in messages {
                if let Some(content) = message.get_mut("content") {
                    self.process_content(content).await?;
                }
            }
        }
        
        Ok(InterceptResult::Processed)
    }
    
    async fn process_content(&self, content: &mut Value) -> Result<(), ProxyError> {
        // 处理数组格式（多模态）
        if let Some(parts) = content.as_array_mut() {
            for part in parts {
                if let Some(url) = part.get("image_url").and_then(|i| i.get("url")) {
                    if let Some(url_str) = url.as_str() {
                        if url_str.starts_with("data:") {
                            self.process_data_url(part, url_str).await?;
                        }
                    }
                }
            }
        }
        
        // 处理字符串格式（纯文本）
        if let Some(text) = content.as_str() {
            // 已经是纯文本，由现有检测器处理
        }
        
        Ok(())
    }
    
    async fn process_data_url(&self, part: &mut Value, data_url: &str) -> Result<(), ProxyError> {
        // 解析 data URL
        let (mime_type, base64_data) = self.parse_data_url(data_url)?;
        
        // 解码 Base64
        let bytes = general_purpose::STANDARD.decode(base64_data)?;
        
        // 检查大小
        if bytes.len() > self.config.max_size {
            return Err(ProxyError::DocumentTooLarge(bytes.len()));
        }
        
        // 根据类型处理
        match self.detect_document_type(&mime_type, &bytes) {
            DocumentType::Pdf => self.process_pdf(part, &bytes).await?,
            DocumentType::Word => self.process_word(part, &bytes).await?,
            DocumentType::Excel => self.process_excel(part, &bytes).await?,
            DocumentType::Image => self.process_image(part, &bytes).await?,
            _ => {} // 不支持的类型，跳过
        }
        
        Ok(())
    }
    
    fn parse_data_url(&self, data_url: &str) -> Result<(String, &str), ProxyError> {
        // data:application/pdf;base64,JVBERi0xLjQK...
        let without_prefix = data_url.strip_prefix("data:")
            .ok_or(ProxyError::InvalidDataUrl)?;
        
        let parts: Vec<&str> = without_prefix.splitn(2, ',').collect();
        if parts.len() != 2 {
            return Err(ProxyError::InvalidDataUrl);
        }
        
        let mime_and_encoding = parts[0];
        let base64_data = parts[1];
        
        let mime_type = mime_and_encoding.split(';').next().unwrap();
        
        Ok((mime_type.to_string(), base64_data))
    }
    
    fn detect_document_type(&self, mime_type: &str, bytes: &[u8]) -> DocumentType {
        // 根据 MIME 类型判断
        match mime_type {
            "application/pdf" => DocumentType::Pdf,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => DocumentType::Word,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => DocumentType::Excel,
            "application/vnd.openxmlformats-officedocument.presentationml.presentation" => DocumentType::PowerPoint,
            m if m.starts_with("image/") => DocumentType::Image,
            _ => DocumentType::Unknown,
        }
    }
}
```

**Step 3: 实现拦截结果**（0.5 天）

```rust
/// 拦截结果
#[derive(Debug, Clone)]
pub enum InterceptResult {
    /// 未修改
    Unchanged,
    /// 已处理
    Processed {
        /// 检测到的敏感数据数量
        matches_count: usize,
        /// 处理的文档类型
        doc_type: DocumentType,
    },
    /// 已脱敏
    Sanitized {
        /// 原始文档大小
        original_size: usize,
        /// 脱敏后大小
        sanitized_size: usize,
        /// 替换数量
        replacements: usize,
    },
    /// 警告（检测到敏感数据但未脱敏）
    Warning {
        message: String,
        matches: Vec<Match>,
    },
}
```

**Step 4: 集成到代理**（1 天）

```rust
// crates/aidaguard-proxy/src/handler.rs

pub async fn handle_chat_request(
    mut req: Request<Body>,
    interceptor: &DocumentInterceptor,
) -> Result<Response<Body>, ProxyError> {
    // 读取请求体
    let body = hyper::body::to_bytes(req.body_mut()).await?;
    let mut json: Value = serde_json::from_slice(&body)?;
    
    // 拦截文档
    let result = interceptor.intercept_request(&mut json).await?;
    
    match result {
        InterceptResult::Warning { message, matches } => {
            // 记录警告
            tracing::warn!("文档检测警告: {}", message);
            // 可选：修改请求添加警告提示
        }
        InterceptResult::Sanitized { .. } => {
            // 重新编码请求体
            let new_body = serde_json::to_vec(&json)?;
            *req.body_mut() = Body::from(new_body);
        }
        _ => {}
    }
    
    // 转发到上游
    forward_request(req).await
}
```

**Step 5: 单元测试**（0.5 天）

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_data_url() {
        let interceptor = DocumentInterceptor::new_mock();
        let url = "data:application/pdf;base64,JVBERi0xLjQK";
        let (mime, data) = interceptor.parse_data_url(url).unwrap();
        assert_eq!(mime, "application/pdf");
        assert_eq!(data, "JVBERi0xLjQK");
    }
    
    #[tokio::test]
    async fn test_intercept_pdf() {
        let interceptor = DocumentInterceptor::new_mock();
        let pdf_bytes = include_bytes!("../tests/fixtures/sample.pdf");
        let base64 = general_purpose::STANDARD.encode(pdf_bytes);
        
        let mut request = json!({
            "messages": [{
                "role": "user",
                "content": [{
                    "type": "image_url",
                    "image_url": {
                        "url": format!("data:application/pdf;base64,{}", base64)
                    }
                }]
            }]
        });
        
        let result = interceptor.intercept_request(&mut request).await.unwrap();
        // 验证结果
    }
}
```

#### 验收标准

- [ ] 能正确解析 data URL 格式
- [ ] 能解码 Base64 内容
- [ ] 能识别 PDF/Word/Excel/Image 类型
- [ ] 能拦截并处理请求中的文档
- [ ] 单元测试通过
- [ ] 性能测试：10MB 文档处理时间 < 1s

#### 文件变更

| 文件 | 操作 |
|------|------|
| `aidaguard-proxy/src/document_interceptor.rs` | 新增 |
| `aidaguard-proxy/src/handler.rs` | 修改 |
| `aidaguard-proxy/src/lib.rs` | 修改 |
| `aidaguard-core/src/config.rs` | 新增配置 |

---

### 3.4 PDF 文本提取

**优先级：** P1  
**工作量：** 2-3 天  
**依赖：** 3.3  
**状态：** 待开始

#### 目标

从 PDF 文档中提取文本内容，用于敏感数据检测。

#### 实现步骤

**Step 1: 添加依赖**（0.5 天）

```toml
# aidaguard-detector/Cargo.toml

[dependencies]
# PDF 文本提取
pdf-extract = "0.7"
lopdf = "0.34"
```

**Step 2: 实现 PDF 处理器**（1 天）

```rust
// crates/aidaguard-detector/src/document/pdf.rs

use pdf_extract::extract_text_from_mem;
use lopdf::Document;

pub struct PdfProcessor;

impl PdfProcessor {
    /// 提取 PDF 文本内容
    pub fn extract_text(data: &[u8]) -> Result<String, DocError> {
        let text = extract_text_from_mem(data)
            .map_err(|e| DocError::PdfExtract(e.to_string()))?;
        Ok(text)
    }
    
    /// 提取 PDF 文本及位置（用于精确脱敏）
    pub fn extract_text_with_positions(data: &[u8]) -> Result<Vec<TextSpan>, DocError> {
        let doc = Document::load_mem(data)
            .map_err(|e| DocError::PdfLoad(e.to_string()))?;
        
        let mut spans = Vec::new();
        
        // 遍历页面
        for (page_num, page) in doc.get_pages() {
            // 提取文本对象
            // ... 复杂的 PDF 解析逻辑
        }
        
        Ok(spans)
    }
    
    /// 获取 PDF 元信息
    pub fn get_metadata(data: &[u8]) -> Result<PdfMetadata, DocError> {
        let doc = Document::load_mem(data)?;
        
        Ok(PdfMetadata {
            page_count: doc.get_pages().len(),
            title: doc.trailer.get("Title").and_then(|v| v.as_str()).map(|s| s.to_string()),
            author: doc.trailer.get("Author").and_then(|v| v.as_str()).map(|s| s.to_string()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct PdfMetadata {
    pub page_count: usize,
    pub title: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TextSpan {
    pub text: String,
    pub page: usize,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
```

**Step 3: 集成到拦截器**（0.5 天）

```rust
// 在 document_interceptor.rs 中

impl DocumentInterceptor {
    async fn process_pdf(&self, part: &mut Value, bytes: &[u8]) -> Result<(), ProxyError> {
        // 提取文本
        let text = PdfProcessor::extract_text(bytes)?;
        
        // 检测敏感数据
        let matches = self.detector.scan(&text);
        
        if matches.is_empty() {
            return Ok(());
        }
        
        // 记录检测结果
        tracing::warn!(
            "PDF 检测到 {} 处敏感数据",
            matches.len()
        );
        
        // 如果启用脱敏
        if self.config.sanitize_pdf {
            // PDF 脱敏复杂，建议使用标注层
            let sanitized = self.sanitize_pdf_with_annotation(bytes, &matches)?;
            let new_base64 = general_purpose::STANDARD.encode(&sanitized);
            
            // 更新请求
            if let Some(url) = part.get_mut("image_url").and_then(|i| i.get_mut("url")) {
                *url = Value::String(format!("data:application/pdf;base64,{}", new_base64));
            }
        } else {
            // 不脱敏，仅警告
            return Err(ProxyError::SensitiveDataDetected {
                doc_type: "PDF".into(),
                count: matches.len(),
                suggestion: "建议检查文档内容后再发送".into(),
            });
        }
        
        Ok(())
    }
}
```

**Step 4: 单元测试**（0.5 天）

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_text() {
        let pdf = include_bytes!("../tests/fixtures/sample.pdf");
        let text = PdfProcessor::extract_text(pdf).unwrap();
        assert!(!text.is_empty());
        assert!(text.contains("Expected text"));
    }
    
    #[test]
    fn test_extract_sensitive_data() {
        let pdf = include_bytes!("../tests/fixtures/contract.pdf");
        let text = PdfProcessor::extract_text(pdf).unwrap();
        
        // 应该能提取到身份证号
        assert!(text.contains("310101"));
    }
}
```

#### 验收标准

- [ ] 能提取 PDF 文本内容
- [ ] 能处理中文 PDF
- [ ] 能处理多页 PDF
- [ ] 文本提取准确率 > 90%（对比人工检查）
- [ ] 性能：10 页 PDF 提取时间 < 500ms

---

### 3.5 Word/Excel 文本提取

**优先级：** P1  
**工作量：** 3-4 天  
**依赖：** 3.3  
**状态：** 待开始

#### 实现步骤

**Step 1: 添加依赖**（0.5 天）

```toml
# aidaguard-detector/Cargo.toml

[dependencies]
# Office 文档
zip = "2"
quick-xml = "0.37"
calamine = "0.26"           # Excel 读取
umya-spreadsheet = "2"     # Excel 读写
docx-rs = "0.4"            # Word 文档
```

**Step 2: 实现 Word 处理器**（1 天）

```rust
// crates/aidaguard-detector/src/document/word.rs

use docx_rs::*;
use zip::ZipArchive;
use quick_xml::{Reader, Events, Event};
use std::io::Cursor;

pub struct WordProcessor;

impl WordProcessor {
    /// 提取 Word 文档文本
    pub fn extract_text(data: &[u8]) -> Result<String, DocError> {
        // 方法 1: 使用 docx-rs
        let doc = read_docx(data)
            .map_err(|e| DocError::WordParse(e.to_string()))?;
        
        let mut text = String::new();
        
        for paragraph in doc.document.paragraphs {
            for run in paragraph.runs {
                if let Some(t) = run.text {
                    text.push_str(&t);
                    text.push(' ');
                }
            }
            text.push('\n');
        }
        
        Ok(text)
    }
    
    /// 提取文本及位置（段落/表格单元格）
    pub fn extract_with_positions(data: &[u8]) -> Result<Vec<TextLocation>, DocError> {
        let mut archive = ZipArchive::new(Cursor::new(data))?;
        let mut document = archive.by_name("word/document.xml")?;
        
        let mut locations = Vec::new();
        let mut reader = Reader::from_reader(&mut document);
        
        let mut current_paragraph = 0;
        let mut in_text = false;
        
        for event in reader.read_events() {
            match event {
                Event::Start(ref e) if e.name() == QName(b"w:p") => {
                    current_paragraph += 1;
                }
                Event::Start(ref e) if e.name() == QName(b"w:t") => {
                    in_text = true;
                }
                Event::Text(ref t) if in_text => {
                    let text = t.unescape_and_decode(&reader).unwrap_or_default();
                    locations.push(TextLocation::WordParagraph {
                        paragraph: current_paragraph,
                        text,
                    });
                }
                Event::End(ref e) if e.name() == QName(b"w:t") => {
                    in_text = false;
                }
                _ => {}
            }
        }
        
        Ok(locations)
    }
    
    /// 脱敏并重建文档
    pub fn sanitize(
        data: &[u8],
        replacements: &[(usize, usize, String)],
    ) -> Result<Vec<u8>, DocError> {
        // 1. 解压 docx
        // 2. 解析 word/document.xml
        // 3. 应用替换
        // 4. 重新打包
        // ... 实现细节
    }
}
```

**Step 3: 实现 Excel 处理器**（1 天）

```rust
// crates/aidaguard-detector/src/document/excel.rs

use calamine::{Reader, Xlsx, open_workbook_from_rs, DataType};
use std::io::Cursor;

pub struct ExcelProcessor;

impl ExcelProcessor {
    /// 提取 Excel 所有单元格文本
    pub fn extract_text(data: &[u8]) -> Result<String, DocError> {
        let cells = Self::extract_cells(data)?;
        
        let text = cells
            .iter()
            .map(|c| c.text.clone())
            .collect::<Vec<_>>()
            .join(" ");
        
        Ok(text)
    }
    
    /// 提取单元格内容
    pub fn extract_cells(data: &[u8]) -> Result<Vec<CellContent>, DocError> {
        let cursor = Cursor::new(data);
        let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)
            .map_err(|e| DocError::ExcelParse(e.to_string()))?;
        
        let mut cells = Vec::new();
        
        for sheet_name in workbook.sheet_names() {
            let range = workbook.worksheet_range(&sheet_name)
                .map_err(|e| DocError::ExcelRange(e.to_string()))?;
            
            for (row, col, value) in range.used_cells() {
                let text = match value {
                    DataType::String(s) => s.clone(),
                    DataType::Float(f) => f.to_string(),
                    DataType::Int(i) => i.to_string(),
                    DataType::Bool(b) => b.to_string(),
                    _ => continue,
                };
                
                cells.push(CellContent {
                    sheet: sheet_name.clone(),
                    row,
                    col,
                    text,
                });
            }
        }
        
        Ok(cells)
    }
    
    /// 脱敏单元格
    pub fn sanitize_cells(
        data: &[u8],
        replacements: &HashMap<(String, u32, u32), String>,
    ) -> Result<Vec<u8>, DocError> {
        // 使用 umya-spreadsheet 修改单元格
        // ... 实现细节
    }
}

#[derive(Debug, Clone)]
pub struct CellContent {
    pub sheet: String,
    pub row: u32,
    pub col: u32,
    pub text: String,
}
```

**Step 4: 集成到拦截器**（0.5 天）

```rust
impl DocumentInterceptor {
    async fn process_word(&self, part: &mut Value, bytes: &[u8]) -> Result<(), ProxyError> {
        // 提取文本
        let text = WordProcessor::extract_text(bytes)?;
        
        // 检测
        let matches = self.detector.scan(&text);
        
        if matches.is_empty() {
            return Ok(());
        }
        
        if self.config.sanitize_word {
            // 脱敏
            let replacements = self.build_replacements(&matches);
            let sanitized = WordProcessor::sanitize(bytes, &replacements)?;
            let new_base64 = general_purpose::STANDARD.encode(&sanitized);
            
            // 更新请求
            self.update_data_url(part, "application/vnd.openxmlformats-officedocument.wordprocessingml.document", &new_base64);
        }
        
        Ok(())
    }
    
    async fn process_excel(&self, part: &mut Value, bytes: &[u8]) -> Result<(), ProxyError> {
        // 类似 Word 处理
        // ...
    }
}
```

**Step 5: 单元测试**（0.5 天）

```rust
#[test]
fn test_word_extract() {
    let docx = include_bytes!("../tests/fixtures/sample.docx");
    let text = WordProcessor::extract_text(docx).unwrap();
    assert!(text.contains("Expected content"));
}

#[test]
fn test_excel_extract() {
    let xlsx = include_bytes!("../tests/fixtures/sample.xlsx");
    let cells = ExcelProcessor::extract_cells(xlsx).unwrap();
    assert!(!cells.is_empty());
}
```

#### 验收标准

- [ ] 能提取 Word 文档文本
- [ ] 能提取 Excel 单元格内容
- [ ] 能处理多 Sheet Excel
- [ ] 能脱敏 Word 文档
- [ ] 能脱敏 Excel 单元格
- [ ] 性能：100 页 Word 提取 < 1s

---

### 3.6 文档脱敏重建

**优先级：** P2  
**工作量：** 5-7 天  
**依赖：** 3.4, 3.5  
**状态：** 待开始

#### 目标

对检测到敏感数据的文档进行精确脱敏，并重建文档保持格式。

#### 实现步骤

**Step 1: 定义脱敏策略**（1 天）

```rust
// crates/aidaguard-detector/src/document/sanitizer.rs

/// 脱敏策略
#[derive(Debug, Clone)]
pub enum SanitizeStrategy {
    /// 占位符替换
    Placeholder {
        template: String,  // 如 "[[身份证号]]"
    },
    /// 部分遮盖
    PartialMask {
        keep_prefix: usize,
        keep_suffix: usize,
        mask_char: char,
    },
    /// 完全删除
    Remove,
    /// 假数据替换
    FakeData {
        entity_type: EntityType,
    },
}

impl Default for SanitizeStrategy {
    fn default() -> Self {
        Self::Placeholder {
            template: "[[已脱敏]]".into(),
        }
    }
}

/// 文档脱敏器
pub struct DocumentSanitizer {
    strategy: SanitizeStrategy,
}

impl DocumentSanitizer {
    /// 对文本应用脱敏
    pub fn sanitize_text(&self, text: &str, matches: &[Match]) -> String {
        let mut result = text.to_string();
        
        // 从后往前替换，避免位置偏移
        for m in matches.iter().rev() {
            let replacement = self.generate_replacement(m);
            result.replace_range(m.start..m.end, &replacement);
        }
        
        result
    }
    
    fn generate_replacement(&self, m: &Match) -> String {
        match &self.strategy {
            SanitizeStrategy::Placeholder { template } => {
                format!("[[{}]]", m.entity_type)
            }
            SanitizeStrategy::PartialMask { keep_prefix, keep_suffix, mask_char } => {
                let text = &m.text;
                let len = text.chars().count();
                if len <= keep_prefix + keep_suffix {
                    "*".repeat(len)
                } else {
                    let prefix: String = text.chars().take(*keep_prefix).collect();
                    let suffix: String = text.chars().skip(len - keep_suffix).collect();
                    let middle = mask_char.to_string().repeat(len - keep_prefix - keep_suffix);
                    format!("{}{}{}", prefix, middle, suffix)
                }
            }
            SanitizeStrategy::FakeData { entity_type } => {
                self.generate_fake_data(entity_type)
            }
            SanitizeStrategy::Remove => String::new(),
        }
    }
    
    fn generate_fake_data(&self, entity_type: &EntityType) -> String {
        match entity_type {
            EntityType::IdCard => "310101199001010001".into(),
            EntityType::Phone => "13800138000".into(),
            EntityType::Email => "example@example.com".into(),
            EntityType::CreditCard => "6222021234567890".into(),
            _ => "[[已脱敏]]".into(),
        }
    }
}
```

**Step 2: Word 文档脱敏**（2 天）

```rust
impl WordProcessor {
    pub fn sanitize(
        data: &[u8],
        matches: &[Match],
        strategy: &SanitizeStrategy,
    ) -> Result<Vec<u8>, DocError> {
        // 1. 解压 docx
        let mut archive = ZipArchive::new(Cursor::new(data))?;
        
        // 2. 读取 document.xml
        let mut document_xml = {
            let mut file = archive.by_name("word/document.xml")?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            content
        };
        
        // 3. 应用替换
        for m in matches.iter().rev() {
            let replacement = strategy.generate_replacement(m);
            // 在 XML 中找到对应位置并替换
            document_xml = replace_in_xml(&document_xml, m.start, m.end, &replacement);
        }
        
        // 4. 更新 archive
        // ... 重新打包
        
        // 5. 返回新的 docx
        Ok(new_docx_bytes)
    }
}
```

**Step 3: Excel 单元格脱敏**（2 天）

```rust
impl ExcelProcessor {
    pub fn sanitize_cells(
        data: &[u8],
        cell_matches: &HashMap<(String, u32, u32), Vec<Match>>,
        strategy: &SanitizeStrategy,
    ) -> Result<Vec<u8>, DocError> {
        // 使用 umya-spreadsheet
        let mut book = umya_spreadsheet::reader::xlsx::read(data)
            .map_err(|e| DocError::ExcelParse(e.to_string()))?;
        
        for ((sheet_name, row, col), matches) in cell_matches {
            let sheet = book.get_sheet_by_name_mut(sheet_name)
                .ok_or(DocError::SheetNotFound)?;
            
            let cell = sheet.get_cell_mut((col + 1, row + 1));  // umya 是 1-indexed
            
            if let Some(value) = cell.get_value() {
                let new_value = strategy.sanitize_text(&value, matches);
                cell.set_value(new_value);
            }
        }
        
        // 保存到内存
        let mut output = Vec::new();
        umya_spreadsheet::writer::xlsx::write(&book, &mut output)?;
        
        Ok(output)
    }
}
```

**Step 4: PDF 标注层脱敏**（1 天）

```rust
impl PdfProcessor {
    /// 使用黑色矩形标注遮盖敏感数据
    pub fn sanitize_with_redaction(
        data: &[u8],
        matches: &[Match],
    ) -> Result<Vec<u8>, DocError> {
        let mut doc = Document::load_mem(data)?;
        
        for m in matches {
            // 找到文本在 PDF 中的位置
            let rect = find_text_position(&doc, &m.text)?;
            
            // 添加黑色矩形标注
            add_redaction(&mut doc, rect)?;
        }
        
        // 保存
        let mut output = Vec::new();
        doc.save_to(&mut output)?;
        
        Ok(output)
    }
}

fn add_redaction(doc: &mut Document, rect: Rect) -> Result<(), DocError> {
    // 添加一个黑色填充的矩形
    // ... PDF 操作细节
}
```

**Step 5: 集成测试**（1 天）

```rust
#[test]
fn test_word_sanitize() {
    let docx = include_bytes!("../tests/fixtures/contract.docx");
    let text = WordProcessor::extract_text(docx).unwrap();
    
    // 检测
    let matches = detector.scan(&text);
    
    // 脱敏
    let sanitized = WordProcessor::sanitize(docx, &matches, &SanitizeStrategy::default()).unwrap();
    
    // 验证
    let sanitized_text = WordProcessor::extract_text(&sanitized).unwrap();
    assert!(!sanitized_text.contains("310101199001011234"));
    assert!(sanitized_text.contains("[[身份证号]]"));
}
```

#### 验收标准

- [ ] Word 文档脱敏后格式保持
- [ ] Excel 单元格脱敏后格式保持
- [ ] PDF 标注层脱敏有效
- [ ] 脱敏后文档可正常打开
- [ ] 敏感数据完全移除

---

### 3.7 插件动态加载

**优先级：** P3  
**工作量：** 5-7 天  
**依赖：** 3.1  
**状态：** 待开始（可选）

#### 说明

此功能为可选增强，可在后续版本实施。

#### 目标

支持运行时加载工具适配器插件，无需重新编译。

#### 实现概要

```rust
// 插件目录结构
~/.aidaguard/plugins/
├── cursor.json        # 元数据
├── cursor.dylib       # macOS
├── cursor.so          # Linux
└── cursor.dll         # Windows

// 插件接口 (C ABI)
#[repr(C)]
pub struct PluginVTable {
    pub id: fn() -> *const i8,
    pub name: fn() -> *const i8,
    pub detect: fn() -> bool,
    pub configure: fn(proxy_url: *const i8) -> i32,
    pub restore: fn() -> i32,
}

// 加载器
pub struct PluginLoader {
    plugins: HashMap<String, DynamicPlugin>,
}

impl PluginLoader {
    pub fn load(&mut self, path: &Path) -> Result<(), PluginError> {
        let lib = Library::new(path)?;
        let vtable = load_vtable(&lib)?;
        // ...
    }
}
```

---

## 三、实施时间线

### Week 1 (Day 1-7)

```
Day 1-2: 3.3 Base64 文档拦截
Day 3-4: 3.4 PDF 文本提取
Day 5-7: 3.1 依赖关系重构
```

### Week 2 (Day 8-14)

```
Day 8-10: 3.5 Word/Excel 文本提取
Day 11-14: 3.2 AuditStorage trait 抽象
```

### Week 3 (Day 15-21)

```
Day 15-18: 3.6 文档脱敏重建（Word/Excel）
Day 19-21: 3.6 文档脱敏重建（PDF）
```

### Week 4+ (Day 22+)

```
Day 22-28: 3.7 插件动态加载（可选）
```

---

## 四、里程碑

| 里程碑 | 版本 | 内容 | 预计时间 |
|--------|------|------|----------|
| M1 | v0.5.0-alpha | Base64 拦截 + PDF/Word/Excel 提取 | Week 2 |
| M2 | v0.5.0-beta | 文档脱敏 + 架构重构 | Week 3 |
| M3 | v0.5.0 | 完整 Phase 3 功能 | Week 4 |
| M4 | v0.6.0 | 插件动态加载（可选） | Week 5+ |

---

## 五、风险与缓解

| 风险 | 等级 | 缓解措施 |
|------|------|----------|
| PDF 解析复杂 | 高 | 先实现文本提取检测，脱敏作为可选 |
| Word/Excel 格式兼容 | 中 | 单元测试覆盖主流版本 |
| 性能影响 | 中 | 异步处理，大小限制 |
| Base64 内存占用 | 中 | 流式解码，大小限制 |

---

## 六、验收标准总览

### 功能验收

- [ ] 能拦截 Base64 编码的文档
- [ ] 能提取 PDF/Word/Excel 文本
- [ ] 能检测文档中的敏感数据
- [ ] 能脱敏 Word/Excel 文档
- [ ] 能标注 PDF 敏感数据
- [ ] 架构清晰，无反向依赖
- [ ] 支持多种存储后端

### 性能验收

- [ ] 10MB 文档处理 < 1s
- [ ] 内存占用 < 100MB（处理 10MB 文档）
- [ ] 不影响正常文本请求延迟

### 质量验收

- [ ] 单元测试覆盖率 > 70%
- [ ] 集成测试通过
- [ ] 文档完整
- [ ] 无编译警告
