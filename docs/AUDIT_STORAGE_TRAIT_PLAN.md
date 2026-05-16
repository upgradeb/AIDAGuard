# AuditStorage Trait 抽象详细规划

**工作项：** 3.2  
**优先级：** P1  
**工作量：** 3-4 天  
**依赖：** 3.1 依赖重构

---

## 一、目标

将存储实现抽象为 trait，支持多种存储后端：
- SQLite（当前实现）
- Memory（测试用）
- PostgreSQL（未来扩展）
- 云存储（未来扩展）

---

## 二、当前状态

### 2.1 当前实现

```rust
// aidaguard-storage/src/lib.rs

pub struct Storage {
    conn: Mutex<Connection>,
    cipher: Aes256Gcm,
}

impl Storage {
    pub fn open(db_path: &Path, encryption_key: &str) -> Result<Self>;
    pub fn record(&self, ...) -> Result<()>;
    pub fn batch_record(&self, records: &[DetectionRecord]) -> Result<usize>;
    pub fn list(&self, limit: usize, offset: usize) -> Result<Vec<DetectionRecord>>;
    pub fn list_filtered(&self, ...) -> Result<Vec<DetectionRecord>>;
    pub fn stats(&self) -> Result<AuditStats>;
    pub fn delete(&self, id: &str) -> Result<bool>;
    // ...
}
```

### 2.2 问题

1. **具体类型耦合** - 使用方直接依赖 `Storage` 具体类型
2. **难以测试** - 测试时必须使用真实数据库
3. **无法扩展** - 不支持其他存储后端

---

## 三、Trait 设计

### 3.1 核心 Trait

```rust
// aidaguard-core/src/storage.rs

use crate::entity::{DetectionRecord, AuditStats, AuditGroup};
use crate::error::StorageError;

/// 审计存储接口
///
/// 所有存储后端必须实现此 trait。
/// 提供检测记录的存储、查询、统计功能。
pub trait AuditStorage: Send + Sync {
    // ── 写入操作 ──
    
    /// 记录单条检测结果
    fn record(
        &self,
        rule_id: &str,
        rule_name: &str,
        strategy: &str,
        placeholder: &str,
        original: &str,
        context: &str,
        request_path: &str,
        sanitized_body: &str,
        response_status: u16,
        tool_name: &str,
    ) -> Result<(), StorageError>;
    
    /// 批量记录检测结果（高性能写入）
    fn batch_record(&self, records: &[DetectionRecord]) -> Result<usize, StorageError>;
    
    // ── 查询操作 ──
    
    /// 分页查询记录（按时间倒序）
    fn list(&self, limit: usize, offset: usize) -> Result<Vec<DetectionRecord>, StorageError>;
    
    /// 条件查询记录
    fn list_filtered(
        &self,
        limit: usize,
        offset: usize,
        filter: AuditFilter,
    ) -> Result<Vec<DetectionRecord>, StorageError>;
    
    /// 按 ID 查询单条记录
    fn get_by_id(&self, id: &str) -> Result<Option<DetectionRecord>, StorageError>;
    
    /// 查询最近记录
    fn list_recent(&self, limit: usize) -> Result<Vec<DetectionRecord>, StorageError>;
    
    // ── 分组查询 ──
    
    /// 按规则和策略分组查询
    fn list_grouped(
        &self,
        limit: usize,
        offset: usize,
        filter: AuditFilter,
    ) -> Result<Vec<AuditGroup>, StorageError>;
    
    /// 分组计数
    fn count_grouped(&self, filter: AuditFilter) -> Result<usize, StorageError>;
    
    // ── 统计操作 ──
    
    /// 总记录数
    fn count(&self) -> Result<usize, StorageError>;
    
    /// 条件计数
    fn count_filtered(&self, filter: AuditFilter) -> Result<usize, StorageError>;
    
    /// 统计信息
    fn stats(&self) -> Result<AuditStats, StorageError>;
    
    // ── 删除操作 ──
    
    /// 删除单条记录
    fn delete(&self, id: &str) -> Result<bool, StorageError>;
    
    /// 清理过期记录
    fn purge_before(&self, timestamp_ms: i64) -> Result<usize, StorageError>;
}
```

### 3.2 查询过滤器

```rust
/// 查询过滤器
#[derive(Debug, Clone, Default)]
pub struct AuditFilter {
    /// 规则 ID 过滤
    pub rule_id: Option<String>,
    
    /// 请求路径过滤（模糊匹配）
    pub path: Option<String>,
    
    /// 起始时间（毫秒时间戳）
    pub date_from_ms: Option<i64>,
    
    /// 结束时间（毫秒时间戳）
    pub date_to_ms: Option<i64>,
    
    /// 策略过滤
    pub strategy: Option<String>,
    
    /// 工具名称过滤
    pub tool_name: Option<String>,
}

impl AuditFilter {
    /// 创建空过滤器
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 按规则 ID 过滤
    pub fn by_rule(rule_id: impl Into<String>) -> Self {
        Self {
            rule_id: Some(rule_id.into()),
            ..Default::default()
        }
    }
    
    /// 按时间范围过滤
    pub fn by_date_range(from_ms: i64, to_ms: i64) -> Self {
        Self {
            date_from_ms: Some(from_ms),
            date_to_ms: Some(to_ms),
            ..Default::default()
        }
    }
    
    /// 链式设置规则 ID
    pub fn with_rule(mut self, rule_id: impl Into<String>) -> Self {
        self.rule_id = Some(rule_id.into());
        self
    }
    
    /// 链式设置路径
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
    
    /// 链式设置时间范围
    pub fn with_date_range(mut self, from_ms: i64, to_ms: i64) -> Self {
        self.date_from_ms = Some(from_ms);
        self.date_to_ms = Some(to_ms);
        self
    }
}
```

---

## 四、实现方案

### 4.1 SQLite 实现（现有）

```rust
// aidaguard-storage/src/sqlite.rs

use aidaguard_core::storage::{AuditStorage, AuditFilter};
use aidaguard_core::entity::{DetectionRecord, AuditStats, AuditGroup};
use aidaguard_core::error::StorageError;

/// SQLite 存储实现
pub struct SqliteStorage {
    conn: Mutex<Connection>,
    cipher: Aes256Gcm,
}

impl SqliteStorage {
    /// 打开数据库
    pub fn open(db_path: &Path, encryption_key: &str) -> Result<Self, StorageError> {
        // 现有实现
    }
    
    /// 创建内存数据库（用于测试）
    pub fn in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()?;
        // 初始化表结构
        // ...
    }
}

impl AuditStorage for SqliteStorage {
    fn record(&self, ...) -> Result<(), StorageError> {
        // 现有实现
    }
    
    // ... 其他方法
}
```

### 4.2 Memory 实现（新增）

```rust
// aidaguard-storage/src/memory.rs

use std::sync::Arc;
use tokio::sync::RwLock;
use aidaguard_core::storage::{AuditStorage, AuditFilter};
use aidaguard_core::entity::{DetectionRecord, AuditStats, AuditGroup};
use aidaguard_core::error::StorageError;

/// 内存存储实现（用于测试）
pub struct MemoryStorage {
    records: Arc<RwLock<Vec<DetectionRecord>>>,
}

impl MemoryStorage {
    /// 创建新的内存存储
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

impl AuditStorage for MemoryStorage {
    fn record(&self, ...) -> Result<(), StorageError> {
        let mut records = self.records.write();
        records.push(DetectionRecord {
            id: Uuid::new_v4().to_string(),
            timestamp_ms: current_timestamp_ms(),
            // ... 其他字段
        });
        Ok(())
    }
    
    fn batch_record(&self, records: &[DetectionRecord]) -> Result<usize, StorageError> {
        let mut storage = self.records.write();
        storage.extend(records.iter().cloned());
        Ok(records.len())
    }
    
    fn list(&self, limit: usize, offset: usize) -> Result<Vec<DetectionRecord>, StorageError> {
        let records = self.records.read();
        Ok(records.iter()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect())
    }
    
    fn list_filtered(&self, limit: usize, offset: usize, filter: AuditFilter) 
        -> Result<Vec<DetectionRecord>, StorageError> 
    {
        let records = self.records.read();
        let filtered = records.iter()
            .filter(|r| Self::matches_filter(r, &filter))
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();
        Ok(filtered)
    }
    
    fn stats(&self) -> Result<AuditStats, StorageError> {
        let records = self.records.read();
        
        let now = current_timestamp_ms();
        let day_ms = 24 * 60 * 60 * 1000;
        let today_start = now - (now % day_ms);
        let week_start = today_start - 7 * day_ms;
        
        Ok(AuditStats {
            total_count: records.len(),
            today_count: records.iter().filter(|r| r.timestamp_ms >= today_start).count(),
            week_count: records.iter().filter(|r| r.timestamp_ms >= week_start).count(),
            rule_distribution: Self::compute_rule_distribution(&records),
            db_size_bytes: 0, // 内存存储无文件大小
        })
    }
    
    // ... 其他方法
}

impl MemoryStorage {
    fn matches_filter(record: &DetectionRecord, filter: &AuditFilter) -> bool {
        if let Some(ref rule_id) = filter.rule_id {
            if record.rule_id != *rule_id { return false; }
        }
        if let Some(ref path) = filter.path {
            if !record.request_path.contains(path) { return false; }
        }
        if let Some(from) = filter.date_from_ms {
            if record.timestamp_ms < from { return false; }
        }
        if let Some(to) = filter.date_to_ms {
            if record.timestamp_ms > to { return false; }
        }
        if let Some(ref strategy) = filter.strategy {
            if record.strategy != *strategy { return false; }
        }
        if let Some(ref tool) = filter.tool_name {
            if record.tool_name != *tool { return false; }
        }
        true
    }
    
    fn compute_rule_distribution(records: &[DetectionRecord]) -> Vec<RuleCount> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for r in records {
            *counts.entry(r.rule_id.clone()).or_insert(0) += 1;
        }
        counts.into_iter()
            .map(|(rule_id, count)| RuleCount { rule_id, count })
            .collect()
    }
}
```

### 4.3 存储工厂

```rust
// aidaguard-storage/src/factory.rs

use aidaguard_core::storage::AuditStorage;
use aidaguard_core::config::StorageConfig;
use aidaguard_core::error::StorageError;

/// 存储工厂
pub struct StorageFactory;

impl StorageFactory {
    /// 根据配置创建存储实例
    pub async fn create(config: &StorageConfig) -> Result<Box<dyn AuditStorage>, StorageError> {
        match config.storage_type.as_str() {
            "sqlite" => {
                let storage = SqliteStorage::open(
                    &PathBuf::from(&config.db_path),
                    config.encryption_key.as_deref().unwrap_or_default()
                )?;
                Ok(Box::new(storage))
            }
            "memory" => {
                let storage = MemoryStorage::new();
                Ok(Box::new(storage))
            }
            #[cfg(feature = "postgres")]
            "postgres" => {
                let url = config.postgres_url.as_ref()
                    .ok_or(StorageError::MissingConfig("postgres_url"))?;
                let storage = PostgresStorage::connect(url).await?;
                Ok(Box::new(storage))
            }
            _ => Err(StorageError::UnknownType(config.storage_type.clone())),
        }
    }
    
    /// 创建默认的 SQLite 存储
    pub fn default_sqlite() -> Result<Box<dyn AuditStorage>, StorageError> {
        let config = StorageConfig::default();
        SqliteStorage::open(
            &PathBuf::from(&config.db_path),
            config.encryption_key.as_deref().unwrap_or_default()
        ).map(|s| Box::new(s) as Box<dyn AuditStorage>)
    }
    
    /// 创建内存存储（测试用）
    pub fn in_memory() -> Box<dyn AuditStorage> {
        Box::new(MemoryStorage::new())
    }
}
```

---

## 五、配置更新

### 5.1 StorageConfig 扩展

```rust
// aidaguard-core/src/config.rs

/// 存储配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    /// 存储类型: "sqlite" | "memory" | "postgres"
    #[serde(default = "default_storage_type")]
    pub storage_type: String,
    
    /// SQLite 数据库路径
    #[serde(default = "default_sqlite_path")]
    pub db_path: String,
    
    /// 加密密钥
    #[serde(default)]
    pub encryption_key: Option<String>,
    
    /// PostgreSQL 连接 URL（feature = "postgres" 时可用）
    #[cfg(feature = "postgres")]
    #[serde(default)]
    pub postgres_url: Option<String>,
}

fn default_storage_type() -> String {
    "sqlite".into()
}

fn default_sqlite_path() -> String {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("aidaguard")
        .join("audit.db")
        .to_string_lossy()
        .into()
}
```

---

## 六、使用示例

### 6.1 通过 Trait 使用

```rust
use aidaguard_core::storage::AuditStorage;
use aidaguard_storage::{SqliteStorage, MemoryStorage, StorageFactory};

// 使用工厂创建
let config = StorageConfig::default();
let storage = StorageFactory::create(&config).await?;

// 通过 trait 调用
storage.record(
    "id_card_cn",
    "中国身份证号",
    "mask",
    "[[ID_CARD]]",
    "310101199001011234",
    "用户输入",
    "/api/chat",
    "...",
    200,
    "cursor"
)?;

let records = storage.list(10, 0)?;
let stats = storage.stats()?;
```

### 6.2 测试中使用 Memory

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use aidaguard_storage::MemoryStorage;
    
    #[test]
    fn test_detection_flow() {
        // 使用内存存储，无需真实数据库
        let storage = MemoryStorage::new();
        
        // 测试记录
        storage.record(...).unwrap();
        
        // 验证
        let records = storage.list(10, 0).unwrap();
        assert_eq!(records.len(), 1);
    }
}
```

---

## 七、文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `aidaguard-core/src/storage.rs` | 新增 | Trait 定义 |
| `aidaguard-core/src/config.rs` | 修改 | 扩展 StorageConfig |
| `aidaguard-storage/src/sqlite.rs` | 新增 | SQLite 实现（从 lib.rs 拆分） |
| `aidaguard-storage/src/memory.rs` | 新增 | Memory 实现 |
| `aidaguard-storage/src/factory.rs` | 新增 | 存储工厂 |
| `aidaguard-storage/src/lib.rs` | 修改 | 导出新模块 |

---

## 八、验收标准

- [ ] `AuditStorage` trait 定义完整
- [ ] `SqliteStorage` 实现 trait
- [ ] `MemoryStorage` 实现 trait
- [ ] `StorageFactory` 可创建不同后端
- [ ] 单元测试覆盖率 > 80%
- [ ] 通过 trait 使用存储的代码可编译
