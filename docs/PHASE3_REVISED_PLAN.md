# AIDAGuard Phase 3 - 重新规划

**版本：** v0.5.0 目标  
**日期：** 2026-05-16  
**状态：** 待实施

---

## 一、核心发现回顾

### 1.1 AI 工具文档发送行为分析结论

> **关键发现：AI 编程工具不会直接发送二进制文档文件到大模型**

| 工具类型 | Word | Excel | PDF | 图片 | 纯文本 | AIDAGuard 覆盖 |
|----------|------|-------|-----|------|--------|----------------|
| CLI 工具 (Claude Code, Codex, OpenClaw) | ❌ | ❌ | ❌ | ❌ | ✅ | **100%** |
| IDE 工具 (Cursor, Cline, Windsurf) | ❌ | ❌ | ❌ | ❌ | ✅ | **100%** |
| Aider | ❌ | ❌ | ❌ | ✅ | ✅ | 需 OCR |
| Web AI (ChatGPT, Claude Web) | ✅ | ⚠️ | ✅ | ✅ | ✅ | **需新增** |

### 1.2 文档解析发生位置

```
┌─────────────────────────────────────────────────────────────────┐
│                    AI 编程工具 (CLI/IDE)                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  用户操作              客户端处理              API 发送          │
│  ────────              ──────────              ────────          │
│  打开 Word → 复制 → 粘贴到工具 → 纯文本 → JSON → AIDAGuard ✅   │
│                                                                 │
│  结论：文档解析在客户端完成，API 请求已经是纯文本                │
│        AIDAGuard 对这些场景已 100% 覆盖                         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                    Web AI (ChatGPT/Claude Web)                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  用户操作              服务端处理              API 发送          │
│  ────────              ──────────              ────────          │
│  上传 Word → 服务端解析 → Base64/URL → JSON → AIDAGuard ⚠️     │
│                                                                 │
│  结论：文档以 Base64 编码发送，AIDAGuard 需要解码检测            │
│        这是 Phase 3 的核心场景                                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 二、Phase 3 重新定位

### 2.1 原规划问题

| 原规划 | 问题 |
|--------|------|
| 支持 Word/Excel/PPT/PDF 文档过滤 | AI 编程工具不支持这些格式上传 |
| 面向所有 AI 工具用户 | 实际仅 Web AI 用户有此需求 |
| 实现复杂的文档脱敏重建 | 投入产出比低 |

### 2.2 重新定位

**Phase 3 的真实目标：**

> 为 **Web AI 用户**（ChatGPT、Claude Web）提供文档上传场景的敏感数据保护。

**目标用户：**
- 使用 ChatGPT 网页版上传文档的用户
- 使用 Claude 网页版上传 PDF 的用户
- 企业内部部署 Web AI 应用的场景

**非目标：**
- AI 编程工具（已 100% 覆盖）
- 不上传文档的 Web AI 使用

### 2.3 调整后的优先级

| 原优先级 | 调整后 | 原因 |
|----------|--------|------|
| P1: PDF/Word/Excel 文档过滤 | **P2** | 仅 Web AI 场景，用户量小于编程工具 |
| P1: Base64 拦截 | **P1** | 前置依赖，必须实现 |
| P2: 依赖关系重构 | **P1** | 架构基础，提高可维护性 |
| P2: 文档脱敏重建 | **P3** | 投入大，收益有限，可简化 |
| P3: 插件动态加载 | **P3** | 保持不变 |

---

## 三、重新规划的工作项

### 3.1 工作项总览

| 编号 | 工作项 | 优先级 | 工作量 | 目标场景 |
|------|--------|--------|--------|----------|
| 3.1 | 架构重构（依赖关系 + AuditStorage trait） | **P1** | 3-5 天 | 代码质量 |
| 3.2 | Base64 文档拦截框架 | **P1** | 2-3 天 | Web AI |
| 3.3 | PDF 文本提取 + 检测 | **P2** | 2-3 天 | Web AI |
| 3.4 | Word/Excel 文本提取 + 检测 | **P3** | 3-4 天 | Web AI（可选） |
| 3.5 | 文档检测警告提示 | **P2** | 1-2 天 | 用户体验 |
| 3.6 | 插件动态加载 | **P3** | 5-7 天 | 扩展性（可选） |

**总工作量：** 16-24 天（约 3-4 周），相比原规划减少 30%

### 3.2 简化说明

**简化点：**

1. **文档脱敏重建 → 检测 + 警告**
   - 原计划：实现复杂的文档脱敏和重建
   - 简化后：仅提取文本检测，向用户发出警告
   - 原因：脱敏重建复杂度高，用户可自行决定是否发送

2. **Word/Excel 支持 → 可选**
   - 原计划：必须支持
   - 简化后：PDF 优先，Word/Excel 作为可选扩展
   - 原因：PDF 是 Web AI 最常见的文档格式

3. **优先架构重构**
   - 原计划：与文档功能并行
   - 调整后：优先完成，为后续开发打基础

---

## 四、详细工作分解

### 4.1 架构重构（P1）

**工作量：** 3-5 天

#### 4.1.1 依赖关系重构（2 天）

**目标：** 消除 `aidaguard-core` → `aidaguard-storage` 的反向依赖

```
重构前：
aidaguard-core → aidaguard-storage (反向依赖)

重构后：
aidaguard-core (定义 trait)
    ↑
aidaguard-storage (实现 trait)
```

**实现：**

```rust
// aidaguard-core/src/storage.rs (新文件)
#[async_trait]
pub trait AuditStorage: Send + Sync {
    async fn record(&self, record: &DetectionRecord) -> Result<(), StorageError>;
    async fn batch_record(&self, records: &[DetectionRecord]) -> Result<usize, StorageError>;
    async fn list(&self, limit: usize, offset: usize) -> Result<Vec<DetectionRecord>, StorageError>;
    // ... 其他方法
}

// aidaguard-storage/src/lib.rs
impl AuditStorage for SqliteStorage { /* 实现 */ }
```

#### 4.1.2 AuditStorage trait 抽象（2-3 天）

**目标：** 支持可插拔的存储后端

```rust
// 存储工厂
pub struct StorageFactory;

impl StorageFactory {
    pub async fn create(config: StorageConfig) -> Result<Box<dyn AuditStorage>, StorageError> {
        match config {
            StorageConfig::Sqlite { path } => Ok(Box::new(SqliteStorage::new(path).await?)),
            StorageConfig::Memory => Ok(Box::new(MemoryStorage::new())),
            // 未来：Postgres, S3
        }
    }
}
```

**收益：**
- 架构清晰，依赖方向正确
- 方便测试（使用 MemoryStorage）
- 支持未来扩展（Postgres、云存储）

---

### 4.2 Base64 文档拦截框架（P1）

**工作量：** 2-3 天

**目标：** 拦截 API 请求中的 Base64 编码文档

#### 实现要点

```rust
// crates/aidaguard-proxy/src/document_interceptor.rs

pub struct DocumentInterceptor {
    config: DocumentInterceptorConfig,
}

#[derive(Debug, Clone)]
pub struct DocumentInterceptorConfig {
    /// 是否启用文档检测
    pub enabled: bool,
    /// 最大文档大小（字节）
    pub max_size: usize,
    /// 检测到敏感数据时的行为
    pub on_sensitive: OnSensitiveAction,
}

#[derive(Debug, Clone)]
pub enum OnSensitiveAction {
    /// 仅记录日志
    LogOnly,
    /// 记录 + 返回警告
    Warn,
    /// 阻止请求
    Block,
}

impl DocumentInterceptor {
    /// 拦截请求中的 Base64 文档
    pub async fn intercept(&self, request_body: &mut Value) -> Result<InterceptResult, ProxyError> {
        // 1. 遍历 messages
        // 2. 检测 data:image/*, data:application/pdf 等
        // 3. 解码 Base64
        // 4. 提取文本
        // 5. 检测敏感数据
        // 6. 根据 on_sensitive 执行动作
    }
}

#[derive(Debug, Clone)]
pub enum InterceptResult {
    /// 无文档或无敏感数据
    Clean,
    /// 检测到敏感数据
    SensitiveDetected {
        doc_type: DocumentType,
        matches: Vec<Match>,
        action_taken: OnSensitiveAction,
    },
    /// 文档过大
    TooLarge { size: usize, max_size: usize },
}
```

**支持格式：**

| MIME 类型 | 格式 | 支持 |
|-----------|------|------|
| `application/pdf` | PDF | ✅ P2 |
| `image/png`, `image/jpeg` | 图片 | ⚠️ OCR 可选 |
| `application/vnd.openxmlformats-officedocument.*` | Office | ❌ P3 |

---

### 4.3 PDF 文本提取 + 检测（P2）

**工作量：** 2-3 天

**目标：** 从 PDF 中提取文本，检测敏感数据

#### 实现要点

```rust
// crates/aidaguard-detector/src/document/pdf.rs

pub struct PdfProcessor;

impl PdfProcessor {
    /// 提取 PDF 文本
    pub fn extract_text(data: &[u8]) -> Result<String, DocError> {
        pdf_extract::extract_text_from_mem(data)
            .map_err(|e| DocError::PdfExtract(e.to_string()))
    }
    
    /// 检测 PDF 敏感数据
    pub fn detect_sensitive(data: &[u8], detector: &AnalyzerEngine) -> Result<Vec<Match>, DocError> {
        let text = Self::extract_text(data)?;
        Ok(detector.scan(&text))
    }
}
```

**依赖：**
```toml
pdf-extract = "0.7"
```

**不实现：**
- ~~PDF 脱敏重建~~（复杂度高，简化为警告）
- ~~PDF 位置信息提取~~（用于精确脱敏，暂不需要）

---

### 4.4 Word/Excel 文本提取 + 检测（P3）

**工作量：** 3-4 天（可选）

**说明：** 
- ChatGPT Web 主要支持 PDF 上传
- Word/Excel 支持可后续扩展
- 作为 P3 优先级，可在 Phase 4 实施

---

### 4.5 文档检测警告提示（P2）

**工作量：** 1-2 天

**目标：** 当检测到文档包含敏感数据时，向用户发出警告

#### 实现方案

**方案 A：修改请求（添加警告文本）**

```rust
// 检测到敏感数据时，在 prompt 中添加警告
if !matches.is_empty() {
    // 在用户消息前添加警告
    let warning = format!(
        "⚠️ 系统检测到文档中包含 {} 处敏感数据，请确认是否继续发送。",
        matches.len()
    );
    // 注入到消息中
}
```

**方案 B：返回错误提示用户**

```rust
// 阻止请求，返回错误
Err(ProxyError::SensitiveDataDetected {
    doc_type: "PDF",
    count: matches.len(),
    matches_preview: matches.iter().take(3).map(|m| m.entity_type.clone()).collect(),
    suggestion: "请检查文档内容，移除敏感数据后重试。",
})
```

**方案 C：UI 通知（Tauri 桌面应用）**

```rust
// 通过 Tauri 事件系统通知前端
app.emit("document-warning", WarningPayload {
    doc_type: "PDF",
    matches_count: matches.len(),
    matches: matches.iter().take(5).cloned().collect(),
})?;
```

**推荐：** 方案 B（简单直接，用户可控）

---

### 4.6 插件动态加载（P3）

**工作量：** 5-7 天（可选）

**说明：** 保持原规划，可在后续版本实施。

---

## 五、简化后的实施计划

### 5.1 时间线

```
Week 1: 架构重构 + Base64 拦截框架
├── Day 1-2: 依赖关系重构
├── Day 3-4: AuditStorage trait
└── Day 5-7: Base64 拦截框架

Week 2: PDF 检测 + 警告提示
├── Day 8-10: PDF 文本提取
├── Day 11-12: 检测集成 + 警告提示
└── Day 13-14: 测试 + 文档

Week 3+: 可选扩展
├── Word/Excel 支持
└── 插件动态加载
```

### 5.2 里程碑

| 里程碑 | 版本 | 内容 | 时间 |
|--------|------|------|------|
| M1 | v0.5.0-alpha | 架构重构 + 拦截框架 | Week 1 |
| M2 | v0.5.0 | PDF 检测 + 警告 | Week 2 |
| M3 | v0.6.0 | Word/Excel + 插件 | Week 3+ |

---

## 六、与原规划对比

| 对比项 | 原规划 | 新规划 | 变化 |
|--------|--------|--------|------|
| 总工作量 | 23-34 天 | 16-24 天 | **-30%** |
| 文档脱敏 | 必须 | 不做 | **简化** |
| Word/Excel | 必须 | 可选 | **降级** |
| 架构重构 | P2 | P1 | **提升** |
| 核心场景 | 所有 AI 工具 | Web AI | **聚焦** |

---

## 七、验收标准

### 功能验收

- [ ] 架构清晰，无反向依赖
- [ ] 能拦截 Base64 编码的 PDF
- [ ] 能提取 PDF 文本并检测敏感数据
- [ ] 检测到敏感数据时能发出警告
- [ ] 支持 MemoryStorage 用于测试

### 性能验收

- [ ] 10MB PDF 处理 < 1s
- [ ] 不影响正常请求延迟

### 质量验收

- [ ] 单元测试覆盖率 > 70%
- [ ] 集成测试通过

---

## 八、总结

### 核心调整

1. **聚焦 Web AI 场景**
   - AI 编程工具已 100% 覆盖
   - Phase 3 专注于 Web AI 文档上传

2. **简化文档处理**
   - 去除复杂的脱敏重建
   - 改为检测 + 警告模式

3. **优先架构重构**
   - 为后续开发打基础
   - 提高代码质量

### 预期收益

- **工作量减少 30%**
- **风险降低**（无复杂脱敏逻辑）
- **用户体验可控**（警告模式让用户自主决定）
- **架构更清晰**（依赖重构完成）
