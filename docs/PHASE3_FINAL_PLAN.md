# AIDAGuard Phase 3 - 最终规划

**版本：** v0.5.0 目标  
**日期：** 2026-05-16  
**状态：** 待实施

---

## 一、项目定位确认

### 1.1 AIDAGuard 的核心场景

> **AIDAGuard 是 API 代理层，专注于 API 流量的敏感数据过滤**
> 
> - ✅ 保护通过 API 发送的数据
> - ❌ 不关注 Web 网页场景

### 1.2 API 场景分析

| 场景 | 文档发送方式 | AIDAGuard 覆盖 |
|------|--------------|----------------|
| AI 编程工具 (Cursor, Claude Code, etc.) | 纯文本 JSON | ✅ **已 100% 覆盖** |
| 自研 AI 应用 | 纯文本 JSON | ✅ **已 100% 覆盖** |
| Web AI (ChatGPT 网页) | 浏览器 → 服务器 | ❌ **非目标场景** |

### 1.3 关键结论

**API 流量中的文档内容：**

```
用户复制文档内容 → 粘贴到 AI 工具 → 工具解析为纯文本 → API 发送 JSON
                                                          ↓
                                                   AIDAGuard 检测 ✅
```

**结论：API 流量中不存在二进制文档，所有内容都是纯文本，当前已完全覆盖。**

---

## 二、Phase 3 重新定位

### 2.1 去除文档过滤功能

**原因：**
1. API 流量中没有二进制文档
2. 文档内容在客户端已解析为纯文本
3. 现有检测引擎已覆盖纯文本场景

**调整：** 删除文档过滤相关的工作项

### 2.2 聚焦架构优化

Phase 3 的核心目标：

1. **架构重构** - 提高代码质量
2. **存储抽象** - 支持多种存储后端
3. **插件增强** - 扩展性提升

---

## 三、Phase 3 工作项

### 3.1 工作项总览

| 编号 | 工作项 | 优先级 | 工作量 | 收益 |
|------|--------|--------|--------|------|
| 3.1 | 依赖关系重构 | **P1** | 2-3 天 | 架构清晰，编译加速 |
| 3.2 | AuditStorage trait 抽象 | **P1** | 3-4 天 | 可扩展存储后端 |
| 3.3 | DetectionEngine trait 优化 | **P2** | 1-2 天 | 可扩展检测引擎 |
| 3.4 | 插件动态加载 | **P2** | 5-7 天 | 运行时扩展 |
| 3.5 | 错误处理增强 | **P2** | 2-3 天 | 更友好的错误提示 |

**总工作量：** 13-19 天（约 2-3 周）

---

## 四、详细工作分解

### 4.1 依赖关系重构（P1）

**工作量：** 2-3 天

#### 目标

消除 `aidaguard-core` → `aidaguard-storage` 的反向依赖

```
重构前：
aidaguard-core → aidaguard-storage (反向依赖)

重构后：
aidaguard-core (定义 trait)
    ↑
aidaguard-storage (实现 trait)
```

#### 实现步骤

**Step 1: 定义 AuditStorage trait**

```rust
// aidaguard-core/src/storage.rs
use async_trait::async_trait;
use crate::entity::DetectionRecord;
use crate::error::StorageError;

#[async_trait]
pub trait AuditStorage: Send + Sync {
    async fn record(&self, record: &DetectionRecord) -> Result<(), StorageError>;
    async fn batch_record(&self, records: &[DetectionRecord]) -> Result<usize, StorageError>;
    async fn list(&self, limit: usize, offset: usize) -> Result<Vec<DetectionRecord>, StorageError>;
    async fn list_filtered(&self, limit: usize, offset: usize, filter: AuditFilter) -> Result<Vec<DetectionRecord>, StorageError>;
    async fn stats(&self) -> Result<AuditStats, StorageError>;
    async fn delete(&self, id: &str) -> Result<(), StorageError>;
    async fn purge_before(&self, timestamp_ms: i64) -> Result<usize, StorageError>;
}
```

**Step 2: 移除反向依赖**

```rust
// 删除 aidaguard-core/src/storage/mod.rs 的 re-export
// 更新 Cargo.toml 移除 aidaguard-storage 依赖
```

**Step 3: 实现 trait**

```rust
// aidaguard-storage/src/lib.rs
impl AuditStorage for SqliteStorage {
    // 实现所有方法
}
```

**Step 4: 更新依赖方**

```rust
// aidaguard-proxy/src/lib.rs
use aidaguard_storage::SqliteStorage;
use aidaguard_core::storage::AuditStorage;
```

#### 验收标准

- [ ] `cargo tree` 显示无反向依赖
- [ ] 所有测试通过
- [ ] 编译时间不增加

---

### 4.2 AuditStorage trait 抽象（P1）

**工作量：** 3-4 天

#### 目标

支持可插拔的存储后端，便于测试和未来扩展。

#### 实现内容

**存储配置：**

```rust
// aidaguard-core/src/config.rs
#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "default_storage_type")]
    pub storage_type: String,  // "sqlite" | "memory"
    
    #[serde(default = "default_sqlite_path")]
    pub sqlite_path: PathBuf,
}
```

**存储工厂：**

```rust
// aidaguard-storage/src/factory.rs
pub struct StorageFactory;

impl StorageFactory {
    pub async fn create(config: StorageConfig) -> Result<Box<dyn AuditStorage>, StorageError> {
        match config.storage_type.as_str() {
            "sqlite" => Ok(Box::new(SqliteStorage::new(&config.sqlite_path).await?)),
            "memory" => Ok(Box::new(MemoryStorage::new())),
            _ => Err(StorageError::UnknownStorageType(config.storage_type)),
        }
    }
}
```

**内存存储（用于测试）：**

```rust
// aidaguard-storage/src/memory.rs
pub struct MemoryStorage {
    records: Arc<RwLock<Vec<DetectionRecord>>>,
}

#[async_trait]
impl AuditStorage for MemoryStorage {
    async fn record(&self, record: &DetectionRecord) -> Result<(), StorageError> {
        self.records.write().await.push(record.clone());
        Ok(())
    }
    // ... 其他方法
}
```

#### 验收标准

- [ ] MemoryStorage 实现完整
- [ ] SqliteStorage 实现 trait
- [ ] 单元测试覆盖率 > 80%
- [ ] 配置文件支持存储类型切换

---

### 4.3 DetectionEngine trait 优化（P2）

**工作量：** 1-2 天

#### 目标

规范化检测引擎接口，支持未来扩展其他检测引擎。

#### 实现

```rust
// aidaguard-core/src/detector.rs
#[async_trait]
pub trait DetectionEngine: Send + Sync {
    /// 检测文本中的敏感数据
    fn scan(&self, text: &str) -> Vec<Match>;
    
    /// 并行检测
    fn scan_parallel(&self, text: &str) -> Vec<Match>;
    
    /// 获取支持的实体类型
    fn supported_entities(&self) -> Vec<EntityType>;
    
    /// 获取引擎名称
    fn name(&self) -> &str;
}

// aidaguard-detector/src/pipeline.rs
impl DetectionEngine for AnalyzerEngine {
    fn scan(&self, text: &str) -> Vec<Match> {
        // 现有实现
    }
}
```

---

### 4.4 插件动态加载（P2）

**工作量：** 5-7 天

#### 目标

支持运行时加载工具适配器插件，无需重新编译。

#### 实现概要

```rust
// aidaguard-plugins/src/dynamic.rs
use libloading::{Library, Symbol};

/// 插件虚函数表 (C ABI)
#[repr(C)]
pub struct PluginVTable {
    pub id: fn() -> *const i8,
    pub name: fn() -> *const i8,
    pub detect: fn() -> bool,
    pub configure: fn(proxy_url: *const i8) -> i32,
    pub restore: fn() -> i32,
}

pub struct PluginLoader {
    plugin_dir: PathBuf,
    loaded: HashMap<String, DynamicPlugin>,
}

impl PluginLoader {
    pub fn scan_and_load(&mut self) -> Result<Vec<String>, PluginError> {
        // 扫描 ~/.aidaguard/plugins/*.dylib
        // 加载符号表
        // 注册到 PluginRegistry
    }
}
```

**插件目录结构：**

```
~/.aidaguard/plugins/
├── cursor.json        # 元数据
├── cursor.dylib       # macOS
├── cursor.so          # Linux
└── cursor.dll         # Windows
```

---

### 4.5 错误处理增强（P2）

**工作量：** 2-3 天

#### 目标

提供更友好的错误提示，便于排查问题。

#### 实现

```rust
// aidaguard-core/src/error.rs

#[derive(Debug, thiserror::Error)]
pub enum AidaGuardError {
    #[error("检测错误: {0}")]
    Detection(#[from] DetectionError),
    
    #[error("存储错误: {0}")]
    Storage(#[from] StorageError),
    
    #[error("代理错误: {0}")]
    Proxy(#[from] ProxyError),
    
    #[error("配置错误: {0}")]
    Config(String),
    
    #[error("插件错误: {0}")]
    Plugin(String),
}

// 用户友好的错误信息
impl AidaGuardError {
    pub fn user_message(&self) -> String {
        match self {
            Self::Detection(e) => format!("数据检测失败: {}", e),
            Self::Storage(e) => format!("数据存储失败: {}", e),
            Self::Proxy(e) => format!("代理服务错误: {}", e),
            Self::Config(msg) => format!("配置错误: {}", msg),
            Self::Plugin(msg) => format!("插件错误: {}", msg),
        }
    }
}
```

---

## 五、实施时间线

```
Week 1: 架构重构
├── Day 1-2: 依赖关系重构
├── Day 3-5: AuditStorage trait 抽象
└── Day 6-7: DetectionEngine trait 优化

Week 2: 扩展增强
├── Day 8-10: 错误处理增强
├── Day 11-14: 插件动态加载
└── Day 15: 测试 + 文档
```

---

## 六、里程碑

| 里程碑 | 版本 | 内容 | 时间 |
|--------|------|------|------|
| M1 | v0.5.0-alpha | 架构重构完成 | Week 1 |
| M2 | v0.5.0 | 所有功能完成 | Week 2 |

---

## 七、验收标准

### 功能验收

- [ ] 无反向依赖
- [ ] AuditStorage trait 完整
- [ ] MemoryStorage 可用
- [ ] DetectionEngine trait 完整
- [ ] 插件可动态加载

### 性能验收

- [ ] 编译时间不增加
- [ ] 运行时性能不受影响

### 质量验收

- [ ] 单元测试覆盖率 > 70%
- [ ] 无编译警告
- [ ] 文档完整

---

## 八、总结

### 核心调整

| 原规划 | 新规划 | 原因 |
|--------|--------|------|
| 文档过滤（PDF/Word/Excel） | **删除** | API 流量无二进制文档 |
| Base64 拦截 | **删除** | 非 API 场景 |
| 依赖重构 | **P1** | 架构基础 |
| 存储抽象 | **P1** | 可测试性 |
| 插件动态加载 | **P2** | 扩展性 |

### 工作量对比

| 对比项 | 原规划 | 新规划 |
|--------|--------|--------|
| 工作项 | 6 个 | 5 个 |
| 工作量 | 16-24 天 | **13-19 天** |
| 时间线 | 3-4 周 | **2-3 周** |

### 核心价值

1. **架构更清晰** - 依赖关系正确
2. **可测试性提升** - MemoryStorage 支持
3. **扩展性增强** - 插件动态加载
4. **开发效率** - 工作量减少 20%
