# AIDAGuard Phase 3 - 架构重构规划

**版本：** v0.5.0 目标  
**日期：** 2026-05-16  
**状态：** 规划中

---

## 一、Phase 3 优化项概览

| 编号 | 优化项 | 优先级 | 复杂度 | 预期收益 |
|------|--------|--------|--------|----------|
| 3.1 | 依赖关系重构 | 中 | 中 | 架构清晰，编译加速 |
| 3.2 | AuditStorage trait 抽象 | 中 | 中 | 可扩展存储后端 |
| 3.3 | 插件系统增强（动态加载） | 低 | 高 | 运行时扩展能力 |
| 3.4 | 文档类过滤支持 | 高 | 高 | 新功能 - Office/PDF 敏感数据检测 |

---

## 二、详细规划

### 3.1 依赖关系重构

**目标：** 消除 `aidaguard-core` → `aidaguard-storage` 的反向依赖

**当前问题：**
```
aidaguard-core (基础层)
  └── aidaguard-storage (存储实现)  ← 反向依赖
```

**重构后：**
```
aidaguard-core (纯基础层，无内部依赖)
  ├── entity types (EntityType, EntityCategory)
  ├── config (Config, NlpConfig, StorageConfig)
  ├── DetectionEngine trait
  ├── AuditStorage trait (新增)
  └── errors (DetectionError, StorageError)

aidaguard-storage (存储实现)
  └── impl AuditStorage for SqliteStorage
```

**实现步骤：**
1. 在 `aidaguard-core` 中定义 `AuditStorage` trait
2. 移除 `aidaguard-core/src/storage/mod.rs` 的 re-export
3. 在 `aidaguard-storage` 中实现 trait
4. 更新所有依赖方的 import

**工作量：** 2-3 天

---

### 3.2 AuditStorage trait 抽象

**目标：** 支持可插拔的存储后端（SQLite / PostgreSQL / 内存 / 云存储）

**Trait 定义：**

```rust
// aidaguard-core/src/storage.rs
use async_trait::async_trait;

#[async_trait]
pub trait AuditStorage: Send + Sync {
    /// 记录检测结果
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
pub struct AuditFilter {
    pub rule_id: Option<String>,
    pub path: Option<String>,
    pub date_from_ms: Option<i64>,
    pub date_to_ms: Option<i64>,
    pub strategy: Option<String>,
}
```

**实现方案：**

| 存储后端 | 适用场景 | 实现复杂度 |
|----------|----------|------------|
| SqliteStorage | 单机桌面应用 | ✅ 已实现 |
| MemoryStorage | 测试 / 轻量场景 | 低 |
| PostgresStorage | 企业部署 | 中 |
| S3Storage | 云存储归档 | 中 |

**工作量：** 3-5 天

---

### 3.3 插件系统增强（动态加载）

**目标：** 支持运行时加载工具适配器插件

**当前状态：**
- 插件硬编码在 `aidaguard-plugins/src/adapters/`
- 新增工具需要修改代码并重新编译

**增强方案：**

```rust
// aidaguard-plugins/src/dynamic.rs
use libloading::{Library, Symbol};
use std::path::PathBuf;

/// 动态加载的插件
pub struct DynamicPlugin {
    library: Library,
    manifest: PluginManifest,
    vtable: PluginVTable,
}

/// 插件虚函数表 (C ABI)
#[repr(C)]
pub struct PluginVTable {
    pub id: fn() -> *const i8,
    pub name: fn() -> *const i8,
    pub detect: fn() -> bool,
    pub configure: fn(proxy_url: *const i8) -> i32,
    pub restore: fn() -> i32,
}

/// 插件加载器
pub struct PluginLoader {
    plugin_dir: PathBuf,
    loaded: HashMap<String, DynamicPlugin>,
}

impl PluginLoader {
    /// 扫描并加载插件目录中的动态库
    pub fn scan_and_load(&mut self) -> Result<Vec<String>, PluginError> {
        // 扫描 ~/.aidaguard/plugins/*.dylib / *.so / *.dll
        // 加载符号表
        // 验证版本兼容性
        // 注册到 PluginRegistry
    }
}
```

**插件目录结构：**
```
~/.aidaguard/plugins/
├── cursor.json        # 插件元数据
├── cursor.dylib       # macOS 动态库
├── cursor.so          # Linux 动态库
├── cursor.dll         # Windows 动态库
└── ...
```

**安全考虑：**
- 插件签名验证
- 沙箱隔离（可选）
- 权限声明

**工作量：** 5-7 天

---

### 3.4 文档类过滤支持（技术可行性分析）

**目标：** 支持 Word / Excel / PDF / PPT 等办公文档的敏感数据检测和脱敏

---

## 三、文档过滤技术可行性分析

### 3.4.1 需求分析

**用户场景：**
1. 用户上传 Word 文档到 AI 工具进行分析
2. AI 工具读取 Excel 表格数据进行处理
3. 用户通过 AI 处理 PDF 合同
4. 企业用户处理包含敏感数据的 PPT 演示文稿

**敏感数据类型：**
- 身份证号、手机号、邮箱
- 信用卡号、银行账号
- 客户姓名、地址
- 公司机密信息

---

### 3.4.2 技术方案对比

#### 方案 A：文档转文本后检测

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│ 文档文件     │ ──▶ │ 文本提取器   │ ──▶ │ 敏感数据检测  │
│ .docx/.xlsx │     │ (Rust crate) │     │ (现有引擎)    │
└─────────────┘     └──────────────┘     └──────────────┘
```

**优点：**
- 复用现有检测引擎
- 实现简单
- 内存占用低

**缺点：**
- 无法保留格式进行脱敏
- 图片/图表中的文字丢失
- 表格结构信息丢失

#### 方案 B：结构化解析 + 位置感知脱敏

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ 文档文件     │ ──▶ │ 结构化解析   │ ──▶ │ 位置感知检测  │ ──▶ │ 精确脱敏输出  │
│ .docx/.xlsx │     │ (保留位置)   │     │ (标记位置)   │     │ (重新打包)   │
└─────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
```

**优点：**
- 保留文档格式
- 精确脱敏（不影响非敏感内容）
- 支持表格单元格级别检测

**缺点：**
- 实现复杂
- 需要完整的文档解析和重建

#### 方案 C：嵌入式检测（推荐）

```
┌──────────────────────────────────────────────────────────────────┐
│                        AIDAGuard Proxy                           │
├──────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐     ┌──────────────┐     ┌──────────────┐      │
│  │ HTTP 请求    │ ──▶ │ 文档解析器   │ ──▶ │ 敏感数据检测  │      │
│  │ (含文档)    │     │ (多格式支持) │     │ (文本+位置)   │      │
│  └─────────────┘     └──────────────┘     └──────────────┘      │
│                              │                                    │
│                              ▼                                    │
│  ┌─────────────┐     ┌──────────────┐     ┌──────────────┐      │
│  │ HTTP 响应    │ ◀── │ 文档重建器   │ ◀── │ 脱敏处理     │      │
│  │ (脱敏文档)  │     │ (保留格式)   │     │ (占位替换)   │      │
│  └─────────────┘     └──────────────┘     └──────────────┘      │
└──────────────────────────────────────────────────────────────────┘
```

**优点：**
- 对用户透明，无需客户端修改
- 支持多种文档格式
- 可选择性脱敏

---

### 3.4.3 各格式技术方案

#### Word (.docx)

**文件格式：** ZIP 压缩包，内含 XML 文件

**解析库：** 
- `zip` + `quick-xml` (Rust 原生)
- `docx-rs` (Rust 专用库)

**实现示例：**

```rust
// crates/aidaguard-detector/src/document/word.rs
use zip::ZipArchive;
use quick_xml::{Reader, Events, Event};

pub struct WordProcessor;

impl WordProcessor {
    /// 提取 Word 文档中的文本及位置
    pub fn extract_text_with_positions(data: &[u8]) -> Result<Vec<TextSpan>, DocError> {
        let mut archive = ZipArchive::new(Cursor::new(data))?;
        let mut document = archive.by_name("word/document.xml")?;
        
        let mut content = Vec::new();
        let mut reader = Reader::from_reader(&mut document);
        
        // 解析 <w:t> 文本节点，记录位置
        for event in reader.read_events() {
            if let Event::Start(ref e) = event {
                if e.name() == QName(b"w:t") {
                    // 提取文本和 XML 位置
                }
            }
        }
        
        Ok(content)
    }
    
    /// 对 Word 文档进行脱敏后重建
    pub fn sanitize_and_rebuild(
        data: &[u8],
        replacements: &[(usize, usize, String)], // (start, end, replacement)
    ) -> Result<Vec<u8>, DocError> {
        // 1. 解压 docx
        // 2. 修改 word/document.xml
        // 3. 重新打包
    }
}
```

**复杂度：** ⭐⭐⭐ 中等

---

#### Excel (.xlsx)

**文件格式：** ZIP 压缩包，内含 XML 工作表文件

**解析库：**
- `calamine` (Rust，纯读取)
- `umya-spreadsheet` (Rust，读写支持)

**实现示例：**

```rust
// crates/aidaguard-detector/src/document/excel.rs
use calamine::{Reader, Xlsx, open_workbook_from_rs};

pub struct ExcelProcessor;

impl ExcelProcessor {
    /// 按单元格提取文本
    pub fn extract_cells(data: &[u8]) -> Result<Vec<CellContent>, DocError> {
        let cursor = Cursor::new(data);
        let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)?;
        
        let mut cells = Vec::new();
        
        for sheet_name in workbook.sheet_names() {
            let range = workbook.worksheet_range(&sheet_name)?;
            
            for (row, col, value) in range.used_cells() {
                if let DataType::String(text) = value {
                    cells.push(CellContent {
                        sheet: sheet_name.clone(),
                        row,
                        col,
                        text,
                    });
                }
            }
        }
        
        Ok(cells)
    }
    
    /// 按单元格脱敏
    pub fn sanitize_cells(
        data: &[u8],
        cell_replacements: &HashMap<(String, u32, u32), String>, // (sheet, row, col) -> replacement
    ) -> Result<Vec<u8>, DocError> {
        // 使用 umya-spreadsheet 修改单元格并保存
    }
}
```

**复杂度：** ⭐⭐⭐ 中等

---

#### PDF (.pdf)

**文件格式：** 复杂二进制格式，包含文本流、字体、布局

**解析库：**
- `pdf-extract` (Rust，纯文本提取)
- `lopdf` (Rust，底层 PDF 操作)
- `pdf` crate (Rust，解析器)

**挑战：**
1. PDF 文本是无序的，需要重建阅读顺序
2. 字体子集化导致字符映射问题
3. 图片中的 OCR 需要额外处理

**实现方案：**

```rust
// crates/aidaguard-detector/src/document/pdf.rs
use pdf_extract::extract_text_from_mem;
use lopdf::Document;

pub struct PdfProcessor;

impl PdfProcessor {
    /// 提取 PDF 文本（纯检测用）
    pub fn extract_text(data: &[u8]) -> Result<String, DocError> {
        let text = extract_text_from_mem(data)?;
        Ok(text)
    }
    
    /// PDF 脱敏（创建标注层）
    pub fn sanitize_with_annotation(
        data: &[u8],
        matches: &[Match],
    ) -> Result<Vec<u8>, DocError> {
        let mut doc = Document::load_mem(data)?;
        
        // 为每个匹配添加黑色矩形标注
        for m in matches {
            let rect = find_text_position(&doc, m.start, m.end)?;
            add_redaction_annotation(&mut doc, rect)?;
        }
        
        doc.save_to_mem()
    }
}
```

**复杂度：** ⭐⭐⭐⭐ 高

**替代方案：**
- 仅提取文本检测，不修改 PDF
- 或使用外部工具（如 `pdftk`、`qpdf`）

---

#### PowerPoint (.pptx)

**文件格式：** ZIP 压缩包，类似 Word

**解析库：**
- `zip` + `quick-xml`
- 或基于 `pptx-rs` (Rust 社区库)

**实现示例：**

```rust
// crates/aidaguard-detector/src/document/powerpoint.rs
pub struct PowerPointProcessor;

impl PowerPointProcessor {
    /// 提取幻灯片中的文本框内容
    pub fn extract_slides(data: &[u8]) -> Result<Vec<SlideContent>, DocError> {
        let mut archive = ZipArchive::new(Cursor::new(data))?;
        
        let mut slides = Vec::new();
        let slide_count = count_slides(&archive)?;
        
        for i in 1..=slide_count {
            let path = format!("ppt/slides/slide{}.xml", i);
            let mut slide_file = archive.by_name(&path)?;
            // 解析 <a:t> 文本节点
        }
        
        Ok(slides)
    }
}
```

**复杂度：** ⭐⭐⭐ 中等

---

### 3.4.4 统一文档处理接口

```rust
// crates/aidaguard-detector/src/document/mod.rs
use async_trait::async_trait;

/// 文档格式类型
#[derive(Debug, Clone)]
pub enum DocumentFormat {
    Word,
    Excel,
    PowerPoint,
    Pdf,
    Text,
    Unknown,
}

/// 文档文本片段（带位置信息）
#[derive(Debug, Clone)]
pub struct TextSpan {
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub location: TextLocation,
}

/// 文本位置
#[derive(Debug, Clone)]
pub enum TextLocation {
    PlainText,
    WordParagraph { paragraph: usize },
    ExcelCell { sheet: String, row: u32, col: u32 },
    PdfPage { page: usize, x: f32, y: f32 },
    SlideText { slide: usize, shape: usize },
}

/// 文档处理器 trait
#[async_trait]
pub trait DocumentProcessor: Send + Sync {
    /// 检测文档格式
    fn detect_format(&self, data: &[u8]) -> DocumentFormat;
    
    /// 提取文本内容（用于检测）
    fn extract_text(&self, data: &[u8]) -> Result<String, DocError>;
    
    /// 提取文本及位置（用于精确脱敏）
    fn extract_with_positions(&self, data: &[u8]) -> Result<Vec<TextSpan>, DocError>;
    
    /// 脱敏并重建文档
    fn sanitize(&self, data: &[u8], replacements: &[TextReplacement]) -> Result<Vec<u8>, DocError>;
    
    /// 支持的格式
    fn supported_formats(&self) -> Vec<DocumentFormat>;
}

/// 文本替换指令
#[derive(Debug, Clone)]
pub struct TextReplacement {
    pub location: TextLocation,
    pub original: String,
    pub replacement: String,
}

/// 文档处理器工厂
pub struct DocumentProcessorFactory {
    processors: HashMap<DocumentFormat, Box<dyn DocumentProcessor>>,
}

impl DocumentProcessorFactory {
    pub fn new() -> Self {
        let mut processors: HashMap<DocumentFormat, Box<dyn DocumentProcessor>> = HashMap::new();
        processors.insert(DocumentFormat::Word, Box::new(WordProcessor));
        processors.insert(DocumentFormat::Excel, Box::new(ExcelProcessor));
        processors.insert(DocumentFormat::PowerPoint, Box::new(PowerPointProcessor));
        processors.insert(DocumentFormat::Pdf, Box::new(PdfProcessor));
        processors.insert(DocumentFormat::Text, Box::new(TextProcessor));
        
        Self { processors }
    }
    
    pub fn get_processor(&self, format: DocumentFormat) -> Option<&dyn DocumentProcessor> {
        self.processors.get(&format).map(|p| p.as_ref())
    }
}
```

---

### 3.4.5 与代理层集成

```rust
// crates/aidaguard-proxy/src/document_handler.rs
use aidaguard_detector::document::{DocumentProcessorFactory, DocumentFormat};

/// 在代理请求处理中添加文档检测
pub async fn process_document_request(
    body: &[u8],
    content_type: &str,
    detector: &AnalyzerEngine,
) -> Result<ProcessedDocument, ProxyError> {
    let factory = DocumentProcessorFactory::new();
    
    // 1. 检测文档格式
    let format = detect_format_from_content_type(content_type)
        .unwrap_or_else(|| factory.detect_format(body));
    
    // 2. 获取处理器
    let processor = factory.get_processor(format)
        .ok_or(ProxyError::UnsupportedFormat)?;
    
    // 3. 提取文本
    let text = processor.extract_text(body)?;
    
    // 4. 运行敏感数据检测
    let matches = detector.scan_parallel(&text);
    
    if matches.is_empty() {
        return Ok(ProcessedDocument::Unchanged(body.to_vec()));
    }
    
    // 5. 生成替换指令
    let replacements = matches.iter().map(|m| TextReplacement {
        location: TextLocation::PlainText, // 简化处理
        original: m.text.clone(),
        replacement: format!("[[{}]]", m.entity_type),
    }).collect();
    
    // 6. 脱敏重建
    let sanitized = processor.sanitize(body, &replacements)?;
    
    Ok(ProcessedDocument::Sanitized(sanitized))
}
```

---

### 3.4.6 实现路线图

| 阶段 | 内容 | 工作量 | 优先级 |
|------|------|--------|--------|
| **Phase 1** | 文本提取 + 检测（只读） | 3-5 天 | 高 |
| Word/Excel 文本提取 | 使用 calamine + docx-rs | | |
| 敏感数据检测 | 复用现有引擎 | | |
| **Phase 2** | 脱敏重建 | 5-7 天 | 中 |
| Word/XML 修改 | 解压 → 修改 → 重打包 | | |
| Excel 单元格替换 | umya-spreadsheet | | |
| **Phase 3** | PDF 支持 | 5-7 天 | 低 |
| PDF 文本提取 | pdf-extract | | |
| PDF 标注层脱敏 | lopdf | | |
| **Phase 4** | 高级功能 | 3-5 天 | 低 |
| OCR 图片文字 | tesseract-rs | | |
| 嵌入对象检测 | 检测嵌入的图片/图表 | | |

---

### 3.4.7 依赖库清单

```toml
# Cargo.toml 新增依赖

[dependencies]
# Word/Excel/PowerPoint (ZIP + XML)
zip = "2"
quick-xml = "0.37"

# Excel 专用
calamine = "0.26"           # 读取
umya-spreadsheet = "2"     # 读写

# Word 专用
docx-rs = "0.4"            # Word 文档操作

# PDF
pdf-extract = "0.7"        # 文本提取
lopdf = "0.34"             # PDF 操作

# 可选：OCR
tesseract-rs = "0.3"       # 图片 OCR (可选)
```

---

### 3.4.8 风险评估

| 风险 | 等级 | 影响 | 缓解措施 |
|------|------|------|----------|
| PDF 格式复杂 | 高 | 脱敏不完整 | 先实现文本提取检测，脱敏作为可选 |
| 大文档内存 | 中 | OOM | 流式处理，限制文档大小 |
| 格式兼容性 | 中 | 特定版本不支持 | 单元测试覆盖主流格式 |
| 性能影响 | 中 | 增加延迟 | 异步处理，后台队列 |
| 加密文档 | 低 | 无法解析 | 返回错误提示 |

---

### 3.4.9 建议实施顺序

1. **先实现检测功能**（文本提取 + 现有引擎检测）
   - Word: `docx-rs` 文本提取
   - Excel: `calamine` 单元格读取
   - PDF: `pdf-extract` 纯文本

2. **再实现脱敏功能**
   - Word: XML 节点替换
   - Excel: 单元格重写
   - PDF: 标注层覆盖（或跳过）

3. **最后集成到代理**
   - Content-Type 判断
   - 文档处理中间件
   - 缓存优化

---

## 四、Phase 3 实施计划

### 时间线

```
Week 1-2: 3.1 依赖关系重构 + 3.2 AuditStorage trait
Week 3-4: 3.4 文档过滤 Phase 1 (检测)
Week 5-6: 3.4 文档过滤 Phase 2 (脱敏)
Week 7-8: 3.3 插件系统增强 (可选)
```

### 里程碑

| 版本 | 内容 | 发布时间 |
|------|------|----------|
| v0.5.0 | 架构重构 + 文档检测 | +4 周 |
| v0.6.0 | 文档脱敏 + 插件增强 | +6 周 |

---

## 五、总结

Phase 3 重点：

1. **架构清理**：依赖关系重构，trait 抽象
2. **新功能**：办公文档敏感数据检测
3. **可扩展性**：插件动态加载

文档过滤技术可行性：**✅ 可行**

- Word/Excel/PPT：技术成熟，Rust 有现成库
- PDF：检测可行，脱敏有挑战（建议先检测后脱敏）
- 建议分阶段实施，优先检测功能
