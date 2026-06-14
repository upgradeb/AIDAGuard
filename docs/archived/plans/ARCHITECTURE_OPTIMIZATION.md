# AIDAGuard 架构优化方案

**版本：** 0.3.0 优化建议  
**日期：** 2026-05-16  
**状态：** 待评审

---

## 一、现有架构评估

### 1.1 架构优势

AIDAGuard 的现有架构设计已经具备以下优点：

| 方面 | 优点 |
|------|------|
| **模块化** | 7 个独立 crate，职责边界清晰，依赖单向 |
| **安全性** | 审计数据 AES-256-GCM 加密，PBKDF2 60万次迭代 |
| **扩展性** | Recognizer trait + Plugin trait 支持灵活扩展 |
| **本地优先** | 无云依赖，敏感数据不离开设备 |
| **技术选型** | Candle 纯 Rust ML，避免 FFI 复杂性 |

### 1.2 架构评分

| 维度 | 评分 | 说明 |
|------|------|------|
| 可维护性 | ⭐⭐⭐⭐ | 模块边界清晰，但存在循环依赖隐患 |
| 性能潜力 | ⭐⭐⭐ | 单线程检测，高并发场景受限 |
| 可测试性 | ⭐⭐⭐⭐ | trait 抽象良好，但集成测试覆盖不足 |
| 扩展性 | ⭐⭐⭐⭐ | 插件系统完善，但规则热加载机制可优化 |
| 安全性 | ⭐⭐⭐⭐⭐ | 加密审计、无云依赖、本地处理 |

---

## 二、核心优化建议

### 2.1 【高优先级】检测引擎并发化

**问题：** 当前 `AnalyzerEngine::scan()` 在单线程中顺序执行所有 recognizer：

```rust
// 当前实现 (aidaguard-detector/src/pipeline.rs)
pub fn scan(&self, text: &str) -> Vec<RecognizerResult> {
    let mut results = self.registry.analyze_all(text);  // 顺序执行
    results = ConfidenceScorer::resolve_overlaps(results);
    results.retain(|r| r.score >= self.min_confidence);
    results
}
```

**影响：** 
- 20+ 个 pattern recognizer + 10 个 NLP recognizer 顺序执行
- 大文本场景（如 10KB+）延迟明显
- 无法利用多核 CPU

**优化方案：**

```rust
// 方案 A：Rayon 并行检测
use rayon::prelude::*;

impl RecognizerRegistry {
    pub fn analyze_all_parallel(&self, text: &str) -> Vec<RecognizerResult> {
        self.recognizers
            .par_iter()  // 并行执行
            .flat_map(|r| r.analyze(text))
            .collect()
    }
}

// 方案 B：异步检测 + join
impl AnalyzerEngine {
    pub async fn scan_async(&self, text: &str) -> Vec<RecognizerResult> {
        let tasks: Vec<_> = self.registry.iter()
            .map(|r| tokio::task::spawn_blocking(move || r.analyze(text)))
            .collect();
        
        let results: Vec<_> = futures::future::join_all(tasks)
            .await
            .into_iter()
            .flat_map(|r| r.unwrap_or_default())
            .collect();
        
        ConfidenceScorer::resolve_overlaps(results)
    }
}
```

**收益预估：**
- 4 核 CPU 上检测吞吐提升 **2-3x**
- 大文本场景延迟降低 **40-60%**

**风险：**
- 需要确保所有 `Recognizer` 实现是 `Sync`
- NLP 模型推理需测试线程安全性

---

### 2.2 【高优先级】代理层流式优化

**问题：** 当前非流式响应需要完整读取响应体后再还原占位符：

```rust
// 当前实现 (aidaguard-proxy/src/server.rs)
let resp_bytes = upstream_resp.bytes().await?;  // 阻塞等待完整响应
let resp_text = if let Some(ref map) = placeholder_map {
    replacer::restore(&resp_text, map)  // 还原后才能返回
} else { ... };
```

**影响：**
- 大模型长回复场景（如代码生成）用户等待时间长
- 无法实现真正的 "边接收边展示"
- 内存峰值较高（需缓存完整响应）

**优化方案：**

```rust
// 方案：增量还原流式响应
pub struct IncrementalRestorer {
    map: PlaceholderMap,
    buffer: String,
    pending: Option<String>,
}

impl IncrementalRestorer {
    /// 处理增量数据块，返回可立即发送的内容
    pub fn process_chunk(&mut self, chunk: &[u8]) -> String {
        self.buffer.push_str(&String::from_utf8_lossy(chunk));
        
        // 检查是否有完整的占位符可还原
        let mut ready = String::new();
        while let Some(pos) = self.buffer.find("[[") {
            if let Some(end) = self.buffer[pos..].find("]]") {
                let placeholder = &self.buffer[pos..=pos+end+1];
                if let Some(original) = self.map.get(placeholder) {
                    ready.push_str(&self.buffer[..pos]);
                    ready.push_str(original);
                    self.buffer = self.buffer[pos+end+2..].to_string();
                } else {
                    // 不完整的占位符，等待更多数据
                    break;
                }
            } else {
                break;
            }
        }
        
        // 返回缓冲区开头确定无占位符的部分
        if let Some(safe_len) = self.buffer.find("[[") {
            if safe_len > 0 {
                ready.push_str(&self.buffer[..safe_len]);
                self.buffer = self.buffer[safe_len..].to_string();
            }
        }
        
        ready
    }
    
    /// 流结束时处理剩余缓冲区
    pub fn finish(&mut self) -> String {
        std::mem::take(&mut self.buffer)
    }
}
```

**收益预估：**
- 首字节延迟降低 **70-90%**
- 内存占用降低 **50-80%**
- 用户体验显著提升（实时看到还原内容）

---

### 2.3 【中优先级】规则热加载机制增强

**问题：** 当前规则热加载依赖 `notify` crate 的文件监听，但存在局限：

```rust
// 当前 Detector 使用 notify 监听
// 但 AnalyzerEngine 层面的 reload 逻辑不够健壮
fn reload_presets(&mut self, base_dir: &Path, presets: &[String]) -> Result<usize, ...> {
    // 仅在调用时加载，无增量更新
    // 无版本校验，无法回滚
}
```

**影响：**
- 规则文件损坏时无保护
- 无法快速回滚到上一版本
- 无规则变更审计

**优化方案：**

```rust
// 方案：规则快照 + 版本管理
pub struct RuleSnapshot {
    version: u64,
    timestamp_ms: i64,
    rules: Vec<CompiledRule>,
    checksum: String,  // SHA-256
}

pub struct VersionedDetector {
    current: Arc<RuleSnapshot>,
    history: VecDeque<Arc<RuleSnapshot>>,  // 保留最近 10 个版本
    max_history: usize,
}

impl VersionedDetector {
    /// 原子切换到新规则版本
    pub fn atomic_swap(&mut self, new_rules: Vec<CompiledRule>) -> Result<u64, Error> {
        let checksum = compute_checksum(&new_rules);
        let snapshot = Arc::new(RuleSnapshot {
            version: self.current.version + 1,
            timestamp_ms: now_ms(),
            rules: new_rules,
            checksum,
        });
        
        // 验证新规则有效性
        self.validate(&snapshot)?;
        
        // 保存历史
        self.history.push_back(self.current.clone());
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
        
        // 原子切换
        self.current = snapshot;
        Ok(self.current.version)
    }
    
    /// 回滚到上一版本
    pub fn rollback(&mut self) -> Result<u64, Error> {
        if let Some(prev) = self.history.pop_back() {
            self.current = prev;
            Ok(self.current.version)
        } else {
            Err(Error::NoHistory)
        }
    }
}
```

**收益：**
- 规则变更可追溯
- 快速回滚故障规则
- 原子切换保证一致性

---

### 2.4 【中优先级】存储层性能优化

**问题：** 当前 SQLite 审计存储在高频写入场景存在瓶颈：

```rust
// 当前实现：每次检测都立即写入
pub fn record(&self, ...) -> Result<()> {
    let conn = self.conn.lock().unwrap();  // 全局锁
    conn.execute(...)?;
    Ok(())
}
```

**影响：**
- 高并发请求时写入成为瓶颈
- SQLite WAL 模式未启用
- 无批量写入优化

**优化方案：**

```rust
// 方案 A：启用 WAL + 批量写入
impl Storage {
    pub fn open(db_path: &Path, encryption_key: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        
        // 启用 WAL 模式
        conn.execute_batch("
            PRAGMA journal_mode=WAL;
            PRAGMA synchronous=NORMAL;
            PRAGMA cache_size=-64000;  // 64MB cache
            PRAGMA busy_timeout=5000;
        ")?;
        
        // ...
    }
}

// 方案 B：异步批量写入
pub struct AsyncStorage {
    tx: tokio::sync::mpsc::Sender<WriteRequest>,
}

struct WriteRequest {
    record: DetectionRecord,
    done: oneshot::Sender<Result<()>>,
}

impl AsyncStorage {
    pub fn spawn(db_path: PathBuf, encryption_key: String) -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1000);
        
        tokio::spawn(async move {
            let storage = Storage::open(&db_path, &encryption_key).unwrap();
            let mut batch = Vec::with_capacity(100);
            let mut last_flush = Instant::now();
            
            while let Some(req) = rx.recv().await {
                batch.push(req);
                
                // 批量写入条件：100 条或 1 秒
                if batch.len() >= 100 || last_flush.elapsed() > Duration::from_secs(1) {
                    let batch_batch = std::mem::take(&mut batch);
                    storage.batch_record(&batch_batch);
                    last_flush = Instant::now();
                }
            }
        });
        
        Self { tx }
    }
    
    pub async fn record(&self, record: DetectionRecord) -> Result<()> {
        let (done, rx) = oneshot::channel();
        self.tx.send(WriteRequest { record, done }).await?;
        rx.await?
    }
}
```

**收益预估：**
- 写入吞吐提升 **5-10x**
- 请求延迟降低 **80%**
- 支持更高并发

---

### 2.5 【中优先级】NLP 推理优化

**问题：** 当前 BERT NER 推理无 GPU 加速，大文本场景慢：

```rust
// 当前实现：CPU 推理
pub fn analyze(&self, text: &str) -> Vec<RecognizerResult> {
    // 对整个文本做推理，无分块
    let tokens = self.tokenizer.encode(text, true).ok()?;
    let output = self.model.forward(&tokens)?;
    // ...
}
```

**优化方案：**

```rust
// 方案 A：文本分块并行推理
impl NlpRecognizer {
    pub fn analyze(&self, text: &str) -> Vec<RecognizerResult> {
        const CHUNK_SIZE: usize = 512;  // BERT 标准长度
        const OVERLAP: usize = 50;      // 重叠避免边界问题
        
        let chunks = self.split_into_chunks(text, CHUNK_SIZE, OVERLAP);
        
        // 使用 rayon 并行推理
        let results: Vec<_> = chunks
            .par_iter()
            .flat_map(|(chunk, offset)| {
                self.infer_chunk(chunk)
                    .into_iter()
                    .map(|mut r| {
                        r.start += offset;
                        r.end += offset;
                        r
                    })
            })
            .collect();
        
        self.merge_adjacent_entities(results)
    }
}

// 方案 B：缓存热点文本（适用于重复 prompt 场景）
pub struct CachedNlpRecognizer {
    inner: NlpRecognizer,
    cache: LruCache<String, Vec<RecognizerResult>>,
}

impl CachedNlpRecognizer {
    pub fn analyze(&self, text: &str) -> Vec<RecognizerResult> {
        let hash = blake3::hash(text.as_bytes());
        let key = hash.to_hex().to_string();
        
        if let Some(cached) = self.cache.get(&key) {
            return cached.clone();
        }
        
        let results = self.inner.analyze(text);
        self.cache.put(key, results.clone());
        results
    }
}
```

**收益预估：**
- 大文本（10KB+）推理加速 **2-3x**
- 缓存命中的重复文本延迟降低 **95%**

---

### 2.6 【低优先级】依赖关系重构

**问题：** 当前 `aidaguard-core` 重新导出 `aidaguard-storage`：

```rust
// aidaguard-core/src/storage/mod.rs
pub use aidaguard_storage::*;
```

这导致 `aidaguard-core` → `aidaguard-storage` 的反向依赖，违背了 core 作为基础层的定位。

**影响：**
- 依赖图不够清晰
- 潜在的循环依赖风险
- 增加编译时间

**优化方案：**

```
// 方案：将 storage 接口抽象为 trait

// aidaguard-core/src/storage.rs
pub trait AuditStorage: Send + Sync {
    fn record(&self, record: &DetectionRecord) -> Result<(), Error>;
    fn list(&self, limit: usize, offset: usize) -> Result<Vec<DetectionRecord>, Error>;
    // ...
}

// aidaguard-storage/src/lib.rs
pub struct SqliteStorage { ... }
impl AuditStorage for SqliteStorage { ... }
```

**优化后的依赖图：**

```
aidaguard-tauri
  ├── aidaguard-core (trait 定义)
  ├── aidaguard-detector
  ├── aidaguard-storage (trait 实现)
  └── aidaguard-proxy
        ├── aidaguard-core
        ├── aidaguard-detector
        └── aidaguard-storage

aidaguard-core (纯基础层，无内部依赖)
  ├── entity types
  ├── config
  ├── DetectionEngine trait
  └── AuditStorage trait (新增)
```

---

### 2.7 【中优先级】UI 界面美化

**问题：** 当前 UI 界面功能完整但视觉体验有提升空间：

**现状分析：**
- 使用 React + Ant Design，基础组件风格统一
- 支持浅色/深色主题切换
- 中英文国际化完善
- 但整体视觉层次感不足，交互反馈不够直观

**优化方案：**

```tsx
// 方案：视觉层级优化 + 微交互动效

// 1. Dashboard 页面优化
// - 添加渐变背景卡片，突出关键指标
// - StatCard 增加数字滚动动画
// - 实时事件流增加入场动画

// 2. 审计日志页面优化
// - 规则命中分布图表增加交互 tooltip
// - 表格行增加 hover 高亮效果
// - 详情面板增加滑入动画

// 3. 规则管理页面优化
// - 规则分类树增加展开/折叠动画
// - 规则测试面板实时高亮匹配文本
// - 规则编辑器增加语法高亮

// 4. 整体风格优化
// - 品牌色系定义：主色 + 辅助色 + 强调色
// - 统一圆角、阴影、间距规范
// - 加载状态骨架屏替换 Spin
// - 错误状态增加友好插图
```

**具体实现建议：**

```css
/* 主题色系定义 */
:root {
  --primary-color: #1890ff;
  --primary-hover: #40a9ff;
  --success-color: #52c41a;
  --warning-color: #faad14;
  --error-color: #ff4d4f;
  --background-gradient: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

/* 卡片样式增强 */
.stat-card {
  border-radius: 12px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
  transition: transform 0.3s ease, box-shadow 0.3s ease;
}

.stat-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
}

/* 数字滚动动画 */
@keyframes countUp {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.stat-number {
  animation: countUp 0.5s ease-out;
}
```

**UI 组件库升级建议：**

| 组件 | 当前状态 | 优化方向 |
|------|----------|----------|
| StatCard | 纯数字展示 | 渐变背景 + 图标 + 动画 |
| EventFeed | 简单列表 | 时间线样式 + 入场动画 |
| AuditTable | 标准表格 | 斑马纹 + hover 高亮 + 固定列 |
| RuleEditor | 纯文本输入 | Monaco Editor 语法高亮 |
| ThemeSwitcher | 简单按钮 | 平滑过渡动画 |

**新增 UI 特性：**

```tsx
// 1. 骨架屏加载状态
<Skeleton active paragraph={{ rows: 4 }} />

// 2. 空状态友好提示
<Empty
  image="./assets/no-data.svg"
  description="暂无审计记录"
>
  <Button type="primary">开始使用</Button>
</Empty>

// 3. 操作反馈 Toast
message.success({
  content: '规则保存成功',
  icon: <CheckCircleOutlined />,
  duration: 2,
});

// 4. 危险操作二次确认
Modal.confirm({
  title: '确认删除规则？',
  content: '删除后无法恢复',
  okText: '确认',
  cancelText: '取消',
  okButtonProps: { danger: true },
});
```

**收益预估：**
- 用户体验满意度提升 **30-50%**
- 新用户上手时间缩短 **20%**
- 视觉一致性和品牌感增强

**风险：**
- 动画性能影响，需注意低端设备适配
- 设计改动可能需要用户适应期

---

### 2.8 【低优先级】错误处理增强

**问题：** 当前大量使用 `anyhow::Error`，错误信息不够结构化：

```rust
// 当前实现
pub fn reload(&mut self, dir: &Path) -> Result<usize, anyhow::Error> {
    // 错误类型不明确
}
```

**优化方案：**

```rust
// 方案：引入 thiserror 定义结构化错误
#[derive(Debug, thiserror::Error)]
pub enum DetectionError {
    #[error("Rule compilation failed: {0}")]
    RuleCompilation(String),
    
    #[error("Invalid regex pattern '{pattern}': {reason}")]
    InvalidRegex { pattern: String, reason: String },
    
    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("NLP model not loaded for language: {0}")]
    ModelNotLoaded(String),
}

// 为每种错误类型提供恢复建议
impl DetectionError {
    pub fn recovery_hint(&self) -> &'static str {
        match self {
            Self::RuleCompilation(_) => "Check YAML syntax and regex validity",
            Self::InvalidRegex { .. } => "Use a regex tester to validate pattern",
            Self::Io(_) => "Check file permissions and disk space",
            Self::ModelNotLoaded(_) => "Run with `nlp` feature and ensure network access",
        }
    }
}
```

---

## 三、架构演进路线图

```
Phase 1 (v0.4.0) - 性能优化
├── 检测引擎并发化 (2.1)
├── 代理层流式优化 (2.2)
└── 存储层 WAL + 批量写入 (2.4)

Phase 2 (v0.5.0) - 可靠性 + 用户体验
├── 规则版本管理 (2.3)
├── UI 界面美化 (2.7)
├── 错误处理增强 (2.8)
└── NLP 分块推理 (2.5)

Phase 3 (v0.6.0) - 架构重构
├── 依赖关系重构 (2.6)
├── AuditStorage trait 抽象
└── 插件系统增强（动态加载）
```

---

## 四、性能基准测试建议

建议添加以下基准测试：

```rust
// benches/detection_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_detection(c: &mut Criterion) {
    let engine = AnalyzerEngine::builder()
        .with_all_pattern_recognizers()
        .build()
        .unwrap();
    
    let small_text = include_str!("../tests/fixtures/small.txt");    // ~1KB
    let medium_text = include_str!("../tests/fixtures/medium.txt");  // ~10KB
    let large_text = include_str!("../tests/fixtures/large.txt");    // ~100KB
    
    c.bench_function("detect_small", |b| {
        b.iter(|| engine.scan(black_box(small_text)))
    });
    
    c.bench_function("detect_medium", |b| {
        b.iter(|| engine.scan(black_box(medium_text)))
    });
    
    c.bench_function("detect_large", |b| {
        b.iter(|| engine.scan(black_box(large_text)))
    });
}

criterion_group!(benches, bench_detection);
criterion_main!(benches);
```

**目标指标：**

| 场景 | 当前预期 | 优化目标 |
|------|----------|----------|
| 1KB 文本检测 | ~5ms | ~2ms |
| 10KB 文本检测 | ~50ms | ~15ms |
| 100KB 文本检测 | ~500ms | ~100ms |
| 1000 并发审计写入 | ~10s | ~1s |

---

## 五、风险评估

| 优化项 | 风险等级 | 主要风险 | 缓解措施 |
|--------|----------|----------|----------|
| 2.1 并发检测 | 中 | 线程安全问题 | 充分测试 NLP 模型线程安全 |
| 2.2 流式优化 | 中 | 占位符边界处理 | 完善单元测试覆盖边界 case |
| 2.3 规则版本 | 低 | 内存占用增加 | 限制历史版本数量 |
| 2.4 存储优化 | 中 | 数据丢失风险 | WAL + 定期 checkpoint |
| 2.5 NLP 优化 | 低 | 精度下降 | 分块重叠策略 |
| 2.6 依赖重构 | 低 | API 变更 | 保留兼容层 |
| 2.7 UI 美化 | 低 | 动画性能 | 低端设备降级 |

---

## 六、总结

AIDAGuard 的架构设计整体优秀，主要优化方向集中在 **性能** 和 **可靠性** 两个方面：

1. **性能优化**：并发检测、流式处理、批量写入可显著提升吞吐
2. **可靠性**：规则版本管理、错误处理增强可提高系统健壮性
3. **用户体验**：UI 美化、交互动效提升使用满意度
4. **架构清理**：依赖关系重构使代码更清晰，但优先级较低

建议按 Phase 1 → Phase 2 → Phase 3 顺序推进，每个版本聚焦核心目标，避免大爆炸式重构。

---

**附录：参考资源**

- [Rayon 并行计算](https://docs.rs/rayon/)
- [SQLite WAL 模式](https://www.sqlite.org/wal.html)
- [Candle ML 框架](https://github.com/huggingface/candle)
- [Axum 流式响应](https://docs.rs/axum/latest/axum/response/struct.Sse.html)
