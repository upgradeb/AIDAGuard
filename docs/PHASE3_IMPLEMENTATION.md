# AIDAGuard Phase 3 - 实施细则

**版本：** v0.5.0  
**日期：** 2026-05-16  
**状态：** 待实施

---

## 一、实施总览

| 编号 | 工作项 | 优先级 | 工作量 | 依赖 |
|------|--------|--------|--------|------|
| 3.1 | 依赖关系重构 | P1 | 2-3 天 | 无 |
| 3.2 | AuditStorage trait 抽象 | P1 | 3-4 天 | 3.1 |
| 3.3 | DetectionEngine trait 优化 | P2 | 1-2 天 | 3.1 |
| 3.4 | 插件动态加载 | P2 | 5-7 天 | 无 |
| 3.5 | 错误处理增强 | P2 | 2-3 天 | 无 |

**总工作量：** 13-19 天

---

## 二、工作项 3.1：依赖关系重构

### 2.1 目标

消除 `aidaguard-core` → `aidaguard-storage` 的反向依赖。

### 2.2 当前问题

```rust
// aidaguard-core/src/storage/mod.rs
pub use aidaguard_storage::*;  // ← 反向依赖

// aidaguard-core/Cargo.toml
[dependencies]
aidaguard-storage = { path = "../aidaguard-storage" }  // ← 不应该存在
```

### 2.3 重构后结构

```
aidaguard-core (纯基础层)
├── src/
│   ├── lib.rs
│   ├── entity.rs          # 实体类型
│   ├── config.rs          # 配置
│   ├── error.rs           # 错误类型
│   ├── storage.rs         # AuditStorage trait (新增)
│   └── detector.rs        # DetectionEngine trait (新增)
└── Cargo.toml             # 无 aidaguard-storage 依赖

aidaguard-storage (存储实现)
├── src/
│   ├── lib.rs             # impl AuditStorage
│   ├── sqlite.rs          # SQLite 实现
│   └── memory.rs          # 内存存储 (新增)
└── Cargo.toml
    [dependencies]
    aidaguard-core = { path = "../aidaguard-core" }  # 正向依赖
```

### 2.4 实施步骤

#### Step 1: 创建 trait 定义文件

```rust
// aidaguard-core/src/storage.rs (新文件)

use async_trait::async_trait;
use std::collections::HashMap;
use crate::entity::DetectionRecord;
use crate::error::StorageError;

/// 审计存储接口
///
/// 所有存储后端必须实现此 trait
#[async_trait]
pub trait AuditStorage: Send + Sync {
    /// 记录单条检测结果
    async fn record(&self, record: &DetectionRecord) -> Result<(), StorageError>;
    
    /// 批量记录检测结果
    async fn batch_record(&self, records: &[DetectionRecord]) -> Result<usize, StorageError>;
    
    /// 分页查询记录
    async fn list(&self, limit: usize, offset: usize) -> Result<Vec<DetectionRecord>, StorageError>;
    
    /// 条件查询记录
    async fn list_filtered(
        &self,
        limit: usize,
        offset: usize,
        filter: AuditFilter,
    ) -> Result<Vec<DetectionRecord>, StorageError>;
    
    /// 获取统计信息
    async fn stats(&self) -> Result<AuditStats, StorageError>;
    
    /// 删除单条记录
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

/// 审计统计信息
#[derive(Debug, Clone)]
pub struct AuditStats {
    pub total_records: usize,
    pub records_by_rule: HashMap<String, usize>,
    pub records_by_strategy: HashMap<String, usize>,
    pub earliest_timestamp_ms: Option<i64>,
    pub latest_timestamp_ms: Option<i64>,
}
```

#### Step 2: 更新 core lib.rs

```rust
// aidaguard-core/src/lib.rs

mod entity;
mod config;
mod error;
mod storage;    // 新增
mod detector;   // 新增

pub use entity::*;
pub use config::*;
pub use error::*;
pub use storage::{AuditStorage, AuditFilter, AuditStats};  // 新增
pub use detector::DetectionEngine;  // 新增
```

#### Step 3: 移除反向依赖

```rust
// 删除 aidaguard-core/src/storage/mod.rs

// 修改 aidaguard-core/Cargo.toml
[dependencies]
# 移除这行
# aidaguard-storage = { path = "../aidaguard-storage" }

# 添加 async-trait
async-trait = "0.1"
```

#### Step 4: 实现 trait

```rust
// aidaguard-storage/src/lib.rs

use aidaguard_core::storage::{AuditStorage, AuditFilter, AuditStats};
use aidaguard_core::entity::DetectionRecord;
use aidaguard_core::error::StorageError;
use async_trait::async_trait;

mod sqlite;
mod memory;

pub use sqlite::SqliteStorage;
pub use memory::MemoryStorage;

// SqliteStorage 已在 sqlite.rs 中实现 trait
```

```rust
// aidaguard-storage/src/sqlite.rs

use async_trait::async_trait;
use aidaguard_core::storage::{AuditStorage, AuditFilter, AuditStats};
use aidaguard_core::entity::DetectionRecord;
use aidaguard_core::error::StorageError;

pub struct SqliteStorage {
    conn: rusqlite::Connection,
}

impl SqliteStorage {
    pub fn new(path: &std::path::Path) -> Result<Self, StorageError> {
        let conn = rusqlite::Connection::open(path)?;
        // 初始化表结构
        Ok(Self { conn })
    }
}

#[async_trait]
impl AuditStorage for SqliteStorage {
    async fn record(&self, record: &DetectionRecord) -> Result<(), StorageError> {
        // 现有实现逻辑
        todo!()
    }
    
    async fn batch_record(&self, records: &[DetectionRecord]) -> Result<usize, StorageError> {
        // 现有实现逻辑
        todo!()
    }
    
    // ... 其他方法
}
```

#### Step 5: 更新依赖方

```rust
// aidaguard-proxy/src/lib.rs

// 修改前
use aidaguard_core::storage::SqliteStorage;

// 修改后
use aidaguard_storage::SqliteStorage;
use aidaguard_core::storage::AuditStorage;
```

```rust
// aidaguard-tauri/src-tauri/src/main.rs

// 修改前
use aidaguard_core::storage::SqliteStorage;

// 修改后
use aidaguard_storage::SqliteStorage;
```

### 2.5 文件变更清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `aidaguard-core/src/storage.rs` | 新增 | trait 定义 |
| `aidaguard-core/src/lib.rs` | 修改 | 导出 trait |
| `aidaguard-core/src/storage/mod.rs` | 删除 | 移除 re-export |
| `aidaguard-core/Cargo.toml` | 修改 | 移除依赖，添加 async-trait |
| `aidaguard-storage/src/lib.rs` | 修改 | 导出实现 |
| `aidaguard-storage/src/sqlite.rs` | 修改 | 实现 trait |
| `aidaguard-storage/Cargo.toml` | 修改 | 添加 aidaguard-core 依赖 |
| `aidaguard-proxy/src/lib.rs` | 修改 | 更新 import |
| `aidaguard-tauri/src-tauri/src/main.rs` | 修改 | 更新 import |

### 2.6 验收标准

- [ ] `cargo tree -p aidaguard-core` 不显示 aidaguard-storage
- [ ] `cargo build` 成功
- [ ] `cargo test` 通过
- [ ] 编译时间不增加

---

## 三、工作项 3.2：AuditStorage trait 抽象

### 3.1 目标

支持可插拔的存储后端，便于测试和扩展。

### 3.2 实施步骤

#### Step 1: 实现 MemoryStorage

```rust
// aidaguard-storage/src/memory.rs

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use aidaguard_core::storage::{AuditStorage, AuditFilter, AuditStats};
use aidaguard_core::entity::DetectionRecord;
use aidaguard_core::error::StorageError;

/// 内存存储（用于测试）
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

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuditStorage for MemoryStorage {
    async fn record(&self, record: &DetectionRecord) -> Result<(), StorageError> {
        let mut records = self.records.write().await;
        records.push(record.clone());
        Ok(())
    }
    
    async fn batch_record(&self, records: &[DetectionRecord]) -> Result<usize, StorageError> {
        let mut storage = self.records.write().await;
        storage.extend(records.iter().cloned());
        Ok(records.len())
    }
    
    async fn list(&self, limit: usize, offset: usize) -> Result<Vec<DetectionRecord>, StorageError> {
        let records = self.records.read().await;
        Ok(records.iter()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect())
    }
    
    async fn list_filtered(
        &self,
        limit: usize,
        offset: usize,
        filter: AuditFilter,
    ) -> Result<Vec<DetectionRecord>, StorageError> {
        let records = self.records.read().await;
        let filtered: Vec<_> = records.iter()
            .filter(|r| {
                if let Some(ref rule_id) = filter.rule_id {
                    if r.rule_id != *rule_id { return false; }
                }
                if let Some(ref path) = filter.path {
                    if r.path != *path { return false; }
                }
                if let Some(ref strategy) = filter.strategy {
                    if r.strategy != *strategy { return false; }
                }
                true
            })
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();
        Ok(filtered)
    }
    
    async fn stats(&self) -> Result<AuditStats, StorageError> {
        let records = self.records.read().await;
        
        let mut records_by_rule = HashMap::new();
        let mut records_by_strategy = HashMap::new();
        
        for r in records.iter() {
            *records_by_rule.entry(r.rule_id.clone()).or_insert(0) += 1;
            *records_by_strategy.entry(r.strategy.clone()).or_insert(0) += 1;
        }
        
        let earliest = records.iter().map(|r| r.timestamp_ms).min();
        let latest = records.iter().map(|r| r.timestamp_ms).max();
        
        Ok(AuditStats {
            total_records: records.len(),
            records_by_rule,
            records_by_strategy,
            earliest_timestamp_ms: earliest,
            latest_timestamp_ms: latest,
        })
    }
    
    async fn delete(&self, id: &str) -> Result<(), StorageError> {
        let mut records = self.records.write().await;
        records.retain(|r| r.id != id);
        Ok(())
    }
    
    async fn purge_before(&self, timestamp_ms: i64) -> Result<usize, StorageError> {
        let mut records = self.records.write().await;
        let before_count = records.len();
        records.retain(|r| r.timestamp_ms >= timestamp_ms);
        Ok(before_count - records.len())
    }
}
```

#### Step 2: 添加存储配置

```rust
// aidaguard-core/src/config.rs

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    /// 存储类型: "sqlite" | "memory"
    #[serde(default = "default_storage_type")]
    pub storage_type: String,
    
    /// SQLite 数据库路径
    #[serde(default = "default_sqlite_path")]
    pub sqlite_path: PathBuf,
}

fn default_storage_type() -> String {
    "sqlite".into()
}

fn default_sqlite_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("aidaguard")
        .join("audit.db")
}
```

#### Step 3: 实现存储工厂

```rust
// aidaguard-storage/src/factory.rs

use aidaguard_core::config::StorageConfig;
use aidaguard_core::storage::AuditStorage;
use aidaguard_core::error::StorageError;

pub struct StorageFactory;

impl StorageFactory {
    pub async fn create(config: &StorageConfig) -> Result<Box<dyn AuditStorage>, StorageError> {
        match config.storage_type.as_str() {
            "sqlite" => {
                let storage = SqliteStorage::new(&config.sqlite_path)?;
                Ok(Box::new(storage))
            }
            "memory" => {
                let storage = MemoryStorage::new();
                Ok(Box::new(storage))
            }
            _ => Err(StorageError::UnknownType(config.storage_type.clone())),
        }
    }
}
```

#### Step 4: 添加单元测试

```rust
// aidaguard-storage/src/memory_test.rs

#[cfg(test)]
mod tests {
    use super::*;
    use aidaguard_core::entity::DetectionRecord;
    
    #[tokio::test]
    async fn test_memory_storage_record() {
        let storage = MemoryStorage::new();
        let record = DetectionRecord {
            id: "test-1".into(),
            rule_id: "id_card".into(),
            path: "/api/chat".into(),
            strategy: "mask".into(),
            timestamp_ms: 1000,
            ..Default::default()
        };
        
        storage.record(&record).await.unwrap();
        let records = storage.list(10, 0).await.unwrap();
        
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, "test-1");
    }
    
    #[tokio::test]
    async fn test_memory_storage_filter() {
        let storage = MemoryStorage::new();
        
        // 添加多条记录
        storage.record(&DetectionRecord { rule_id: "id_card".into(), ..Default::default() }).await.unwrap();
        storage.record(&DetectionRecord { rule_id: "phone".into(), ..Default::default() }).await.unwrap();
        
        // 过滤查询
        let filter = AuditFilter {
            rule_id: Some("id_card".into()),
            ..Default::default()
        };
        let filtered = storage.list_filtered(10, 0, filter).await.unwrap();
        
        assert_eq!(filtered.len(), 1);
    }
    
    #[tokio::test]
    async fn test_memory_storage_stats() {
        let storage = MemoryStorage::new();
        
        storage.record(&DetectionRecord { rule_id: "id_card".into(), ..Default::default() }).await.unwrap();
        storage.record(&DetectionRecord { rule_id: "id_card".into(), ..Default::default() }).await.unwrap();
        storage.record(&DetectionRecord { rule_id: "phone".into(), ..Default::default() }).await.unwrap();
        
        let stats = storage.stats().await.unwrap();
        
        assert_eq!(stats.total_records, 3);
        assert_eq!(stats.records_by_rule.get("id_card"), Some(&2));
        assert_eq!(stats.records_by_rule.get("phone"), Some(&1));
    }
}
```

### 3.3 文件变更清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `aidaguard-storage/src/memory.rs` | 新增 | 内存存储实现 |
| `aidaguard-storage/src/factory.rs` | 新增 | 存储工厂 |
| `aidaguard-core/src/config.rs` | 修改 | 添加 StorageConfig |
| `aidaguard-storage/src/lib.rs` | 修改 | 导出新模块 |

### 3.4 验收标准

- [ ] MemoryStorage 所有方法实现完整
- [ ] 单元测试覆盖率 > 80%
- [ ] 可通过配置切换存储类型

---

## 四、工作项 3.3：DetectionEngine trait 优化

### 4.1 目标

规范化检测引擎接口。

### 4.2 实施步骤

#### Step 1: 定义 trait

```rust
// aidaguard-core/src/detector.rs

use crate::entity::{EntityType, Match};

/// 检测引擎接口
pub trait DetectionEngine: Send + Sync {
    /// 检测文本中的敏感数据
    fn scan(&self, text: &str) -> Vec<Match>;
    
    /// 并行检测（性能优化）
    fn scan_parallel(&self, text: &str) -> Vec<Match> {
        // 默认实现：调用 scan
        self.scan(text)
    }
    
    /// 获取支持的实体类型
    fn supported_entities(&self) -> Vec<EntityType>;
    
    /// 获取引擎名称
    fn name(&self) -> &str;
}
```

#### Step 2: 实现 trait

```rust
// aidaguard-detector/src/pipeline.rs

use aidaguard_core::detector::DetectionEngine;

impl DetectionEngine for AnalyzerEngine {
    fn scan(&self, text: &str) -> Vec<Match> {
        // 现有实现
        self.analyze(text)
    }
    
    fn scan_parallel(&self, text: &str) -> Vec<Match> {
        // 现有并行实现
        self.analyze_all_parallel(text)
    }
    
    fn supported_entities(&self) -> Vec<EntityType> {
        self.registry.get_supported_entities()
    }
    
    fn name(&self) -> &str {
        "AidaGuard AnalyzerEngine"
    }
}
```

### 4.3 文件变更清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `aidaguard-core/src/detector.rs` | 新增 | trait 定义 |
| `aidaguard-core/src/lib.rs` | 修改 | 导出 trait |
| `aidaguard-detector/src/pipeline.rs` | 修改 | 实现 trait |

---

## 五、工作项 3.4：插件动态加载

### 5.1 目标

支持运行时加载工具适配器插件。

### 5.2 实施步骤

#### Step 1: 定义插件 ABI

```rust
// aidaguard-plugins/src/dynamic/abi.rs

use std::ffi::c_int;

/// 插件元数据
#[repr(C)]
pub struct PluginMeta {
    pub id: *const i8,
    pub name: *const i8,
    pub version: *const i8,
    pub description: *const i8,
}

/// 插件虚函数表 (C ABI)
#[repr(C)]
pub struct PluginVTable {
    /// 获取插件 ID
    pub id: unsafe fn() -> *const i8,
    
    /// 获取插件名称
    pub name: unsafe fn() -> *const i8,
    
    /// 检测是否安装
    pub detect: unsafe fn() -> bool,
    
    /// 配置代理
    pub configure: unsafe fn(proxy_url: *const i8) -> c_int,
    
    /// 恢复配置
    pub restore: unsafe fn() -> c_int,
    
    /// 获取当前端点
    pub current_endpoint: unsafe fn() -> *const i8,
}
```

#### Step 2: 实现插件加载器

```rust
// aidaguard-plugins/src/dynamic/loader.rs

use libloading::{Library, Symbol};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// 动态加载的插件
pub struct DynamicPlugin {
    library: Library,
    vtable: PluginVTable,
    manifest: PluginManifest,
}

/// 插件加载器
pub struct PluginLoader {
    plugin_dir: PathBuf,
    loaded: HashMap<String, DynamicPlugin>,
}

impl PluginLoader {
    pub fn new(plugin_dir: PathBuf) -> Self {
        Self {
            plugin_dir,
            loaded: HashMap::new(),
        }
    }
    
    /// 扫描并加载所有插件
    pub fn scan_and_load(&mut self) -> Result<Vec<String>, PluginError> {
        let mut loaded_ids = Vec::new();
        
        // 扫描插件目录
        for entry in std::fs::read_dir(&self.plugin_dir)? {
            let path = entry?.path();
            
            // 检查文件扩展名
            let ext = path.extension().and_then(|s| s.to_str());
            let is_plugin = match ext {
                #[cfg(target_os = "macos")]
                Some("dylib") => true,
                #[cfg(target_os = "linux")]
                Some("so") => true,
                #[cfg(target_os = "windows")]
                Some("dll") => true,
                _ => false,
            };
            
            if is_plugin {
                if let Ok(id) = self.load(&path) {
                    loaded_ids.push(id);
                }
            }
        }
        
        Ok(loaded_ids)
    }
    
    /// 加载单个插件
    pub fn load(&mut self, path: &Path) -> Result<String, PluginError> {
        // 加载动态库
        let library = unsafe { Library::new(path)? };
        
        // 加载符号
        let get_vtable: Symbol<unsafe fn() -> PluginVTable> = 
            unsafe { library.get(b"plugin_vtable")? };
        
        let vtable = unsafe { get_vtable() };
        
        // 获取插件 ID
        let id = unsafe {
            let ptr = (vtable.id)();
            std::ffi::CStr::from_ptr(ptr)
                .to_string_lossy()
                .into_owned()
        };
        
        // 加载元数据
        let manifest = self.load_manifest(&id)?;
        
        let plugin = DynamicPlugin {
            library,
            vtable,
            manifest,
        };
        
        self.loaded.insert(id.clone(), plugin);
        
        Ok(id)
    }
    
    fn load_manifest(&self, plugin_id: &str) -> Result<PluginManifest, PluginError> {
        let manifest_path = self.plugin_dir.join(format!("{}.json", plugin_id));
        let content = std::fs::read_to_string(manifest_path)?;
        let manifest: PluginManifest = serde_json::from_str(&content)?;
        Ok(manifest)
    }
    
    /// 获取已加载的插件
    pub fn get(&self, id: &str) -> Option<&DynamicPlugin> {
        self.loaded.get(id)
    }
    
    /// 列出所有已加载插件
    pub fn list(&self) -> Vec<&str> {
        self.loaded.keys().map(|s| s.as_str()).collect()
    }
}
```

#### Step 3: 集成到插件注册表

```rust
// aidaguard-plugins/src/registry.rs

impl PluginRegistry {
    /// 加载动态插件
    pub fn load_dynamic_plugins(&mut self, plugin_dir: &Path) -> Result<Vec<String>, PluginError> {
        let mut loader = PluginLoader::new(plugin_dir.to_path_buf());
        let loaded = loader.scan_and_load()?;
        
        // 注册到 registry
        for id in &loaded {
            if let Some(plugin) = loader.get(id) {
                self.register_dynamic(plugin);
            }
        }
        
        Ok(loaded)
    }
}
```

### 5.3 插件开发指南

创建一个示例插件：

```rust
// plugins/cursor/src/lib.rs

use std::ffi::{c_int, CStr, CString};
use aidaguard_plugins::dynamic::{PluginVTable, PluginMeta};

static ID: &[u8] = b"cursor\0";
static NAME: &[u8] = b"Cursor\0";

#[no_mangle]
pub unsafe extern "C" fn plugin_vtable() -> PluginVTable {
    PluginVTable {
        id: || ID.as_ptr() as *const i8,
        name: || NAME.as_ptr() as *const i8,
        detect: || {
            // 检测 Cursor 是否安装
            std::path::Path::new("~/Library/Application Support/Cursor").exists()
        },
        configure: |proxy_url| {
            // 配置代理
            let url = CStr::from_ptr(proxy_url);
            // ... 配置逻辑
            0  // 成功
        },
        restore: || {
            // 恢复配置
            0
        },
        current_endpoint: || {
            // 返回当前端点
            std::ptr::null()
        },
    }
}
```

### 5.4 文件变更清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `aidaguard-plugins/src/dynamic/mod.rs` | 新增 | 动态加载模块 |
| `aidaguard-plugins/src/dynamic/abi.rs` | 新增 | ABI 定义 |
| `aidaguard-plugins/src/dynamic/loader.rs` | 新增 | 加载器实现 |
| `aidaguard-plugins/src/registry.rs` | 修改 | 集成动态加载 |
| `aidaguard-plugins/Cargo.toml` | 修改 | 添加 libloading 依赖 |

---

## 六、工作项 3.5：错误处理增强

### 6.1 目标

提供更友好的错误提示。

### 6.2 实施步骤

#### Step 1: 统一错误类型

```rust
// aidaguard-core/src/error.rs

use thiserror::Error;

#[derive(Debug, Error)]
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
    Plugin(#[from] PluginError),
}

#[derive(Debug, Error)]
pub enum DetectionError {
    #[error("规则加载失败: {0}")]
    RuleLoad(String),
    
    #[error("NLP 模型加载失败: {0}")]
    NlpModelLoad(String),
    
    #[error("检测失败: {0}")]
    ScanFailed(String),
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("数据库连接失败: {0}")]
    Connection(String),
    
    #[error("查询失败: {0}")]
    Query(String),
    
    #[error("写入失败: {0}")]
    Write(String),
    
    #[error("未知存储类型: {0}")]
    UnknownType(String),
}

#[derive(Debug, Error)]
pub enum ProxyError {
    #[error("上游连接失败: {0}")]
    UpstreamConnection(String),
    
    #[error("请求解析失败: {0}")]
    RequestParse(String),
    
    #[error("响应处理失败: {0}")]
    ResponseProcess(String),
}

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("插件加载失败: {0}")]
    Load(String),
    
    #[error("插件未找到: {0}")]
    NotFound(String),
    
    #[error("插件配置失败: {0}")]
    Config(String),
}
```

#### Step 2: 添加用户友好消息

```rust
impl AidaGuardError {
    /// 获取用户友好的错误消息
    pub fn user_message(&self) -> String {
        match self {
            Self::Detection(e) => match e {
                DetectionError::RuleLoad(msg) => {
                    format!("⚠️ 规则加载失败: {}\n请检查规则文件格式是否正确。", msg)
                }
                DetectionError::NlpModelLoad(msg) => {
                    format!("⚠️ NLP 模型加载失败: {}\n请确保模型文件存在且完整。", msg)
                }
                DetectionError::ScanFailed(msg) => {
                    format!("⚠️ 数据检测失败: {}", msg)
                }
            }
            Self::Storage(e) => match e {
                StorageError::Connection(msg) => {
                    format!("⚠️ 数据库连接失败: {}\n请检查数据库路径是否正确。", msg)
                }
                StorageError::Query(msg) => {
                    format!("⚠️ 数据查询失败: {}", msg)
                }
                StorageError::Write(msg) => {
                    format!("⚠️ 数据写入失败: {}", msg)
                }
                StorageError::UnknownType(t) => {
                    format!("⚠️ 未知的存储类型: {}\n支持的类型: sqlite, memory", t)
                }
            }
            Self::Proxy(e) => match e {
                ProxyError::UpstreamConnection(msg) => {
                    format!("⚠️ 无法连接到上游服务: {}\n请检查网络连接和上游地址。", msg)
                }
                ProxyError::RequestParse(msg) => {
                    format!("⚠️ 请求解析失败: {}", msg)
                }
                ProxyError::ResponseProcess(msg) => {
                    format!("⚠️ 响应处理失败: {}", msg)
                }
            }
            Self::Config(msg) => {
                format!("⚠️ 配置错误: {}\n请检查配置文件格式。", msg)
            }
            Self::Plugin(e) => match e {
                PluginError::Load(msg) => {
                    format!("⚠️ 插件加载失败: {}", msg)
                }
                PluginError::NotFound(id) => {
                    format!("⚠️ 插件未找到: {}\n请确保插件已安装。", id)
                }
                PluginError::Config(msg) => {
                    format!("⚠️ 插件配置失败: {}", msg)
                }
            }
        }
    }
    
    /// 获取错误代码
    pub fn code(&self) -> &'static str {
        match self {
            Self::Detection(_) => "DETECTION_ERROR",
            Self::Storage(_) => "STORAGE_ERROR",
            Self::Proxy(_) => "PROXY_ERROR",
            Self::Config(_) => "CONFIG_ERROR",
            Self::Plugin(_) => "PLUGIN_ERROR",
        }
    }
}
```

### 6.3 文件变更清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `aidaguard-core/src/error.rs` | 重写 | 统一错误类型 |
| `aidaguard-core/Cargo.toml` | 修改 | 添加 thiserror 依赖 |

---

## 七、测试计划

### 7.1 单元测试

```bash
# 运行所有单元测试
cargo test --workspace

# 运行特定模块测试
cargo test -p aidaguard-storage --lib
```

### 7.2 集成测试

```rust
// tests/integration_test.rs

#[tokio::test]
async fn test_storage_trait_polymorphism() {
    // 测试 SQLite 存储
    let sqlite = SqliteStorage::new_in_memory().await.unwrap();
    test_storage_impl(&sqlite).await;
    
    // 测试内存存储
    let memory = MemoryStorage::new();
    test_storage_impl(&memory).await;
}

async fn test_storage_impl(storage: &dyn AuditStorage) {
    let record = DetectionRecord::default();
    storage.record(&record).await.unwrap();
    let records = storage.list(10, 0).await.unwrap();
    assert_eq!(records.len(), 1);
}
```

### 7.3 性能测试

```rust
#[test]
fn test_memory_storage_performance() {
    let storage = MemoryStorage::new();
    
    // 插入 10000 条记录
    let start = std::time::Instant::now();
    for i in 0..10000 {
        let record = DetectionRecord { id: format!("test-{}", i), ..Default::default() };
        tokio::runtime::Runtime::new().unwrap().block_on(storage.record(&record)).unwrap();
    }
    let elapsed = start.elapsed();
    
    assert!(elapsed < std::time::Duration::from_secs(1));
}
```

---

## 八、实施检查清单

### 8.1 依赖重构

- [ ] 创建 `aidaguard-core/src/storage.rs`
- [ ] 移除 `aidaguard-core/Cargo.toml` 中的 aidaguard-storage 依赖
- [ ] 实现 `AuditStorage` trait
- [ ] 更新所有 import
- [ ] `cargo tree` 验证无反向依赖
- [ ] `cargo test` 通过

### 8.2 存储抽象

- [ ] 实现 `MemoryStorage`
- [ ] 实现 `StorageFactory`
- [ ] 添加 `StorageConfig`
- [ ] 单元测试覆盖率 > 80%

### 8.3 检测引擎

- [ ] 定义 `DetectionEngine` trait
- [ ] 实现 trait
- [ ] 更新使用方

### 8.4 插件系统

- [ ] 定义插件 ABI
- [ ] 实现 `PluginLoader`
- [ ] 集成到 registry
- [ ] 创建示例插件

### 8.5 错误处理

- [ ] 定义统一错误类型
- [ ] 实现 `user_message()`
- [ ] 更新所有错误处理

---

## 九、发布检查

### 9.1 代码质量

- [ ] `cargo clippy -- -D warnings` 无警告
- [ ] `cargo fmt -- --check` 格式正确
- [ ] `cargo test` 全部通过

### 9.2 文档

- [ ] README 更新
- [ ] CHANGELOG 更新
- [ ] API 文档生成

### 9.3 版本

- [ ] 更新 Cargo.toml 版本为 0.5.0
- [ ] 更新 tauri.conf.json 版本
- [ ] Git tag v0.5.0
