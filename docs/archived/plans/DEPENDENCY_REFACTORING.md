# AIDAGuard 依赖重构详细分析

**日期：** 2026-05-16  
**状态：** 待实施

---

## 一、当前依赖关系

### 1.1 Workspace 结构图

```
aidaguard (workspace)
├── aidaguard-core       (基础层)
├── aidaguard-storage    (存储层)
├── aidaguard-detector   (检测层)
├── aidaguard-upstream   (上游管理)
├── aidaguard-proxy      (代理层)
├── aidaguard-plugins    (插件系统)
└── aidaguard-tauri      (桌面应用)
```

### 1.2 当前依赖图

```
aidaguard-core
├── aidaguard-storage  ← 反向依赖！
├── tokio, regex, serde, ...
└── (其他依赖)

aidaguard-storage
├── rusqlite, aes-gcm, pbkdf2, ...
└── (无 aidaguard-* 依赖)

aidaguard-detector
├── aidaguard-core  ✅ 正确
├── rayon, candle-* (optional), ...
└── (其他依赖)

aidaguard-proxy
├── aidaguard-core     ✅ 正确
├── aidaguard-detector ✅ 正确
├── aidaguard-storage  ⚠️ 直接依赖存储实现
├── aidaguard-upstream ✅ 正确
└── axum, reqwest, ...

aidaguard-upstream
├── aidaguard-core  ✅ 正确
└── reqwest, ...

aidaguard-tauri
├── aidaguard-core
├── aidaguard-detector
├── aidaguard-proxy
├── aidaguard-storage
└── tauri, ...
```

### 1.3 问题：反向依赖

**问题代码：**

```rust
// aidaguard-core/Cargo.toml
[dependencies]
aidaguard-storage = { path = "../aidaguard-storage" }  // ← 不应该存在

// aidaguard-core/src/storage/mod.rs
pub use aidaguard_storage::*;  // ← 反向依赖
```

**为什么这是问题：**

1. **架构违背**：core 是基础层，不应依赖 storage 实现层
2. **依赖方向错误**：应该是 storage 依赖 core，而非相反
3. **编译影响**：修改 storage 会触发 core 重新编译，进而触发所有依赖 core 的 crate 重新编译
4. **测试困难**：无法为 core 编写独立测试，必须依赖 storage

---

## 二、依赖的使用情况分析

### 2.1 aidaguard-core 从 storage 导出的类型

```rust
// aidaguard-storage/src/lib.rs 导出：
pub struct DetectionRecord { ... }
pub struct RuleCount { ... }
pub struct AuditGroup { ... }
pub struct AuditStats { ... }
pub struct Storage { ... }

// aidaguard-core/src/storage/mod.rs 重新导出：
pub use aidaguard_storage::*;
// 使得其他 crate 可以通过 aidaguard_core::storage::* 使用这些类型
```

### 2.2 使用这些类型的地方

**aidaguard-proxy/src/server.rs：**
```rust
use aidaguard_storage::Storage;  // 直接使用 Storage
```

**aidaguard-tauri：**
```rust
// 使用 DetectionRecord、AuditStats 等类型
```

---

## 三、重构方案

### 3.1 目标依赖图

```
aidaguard-core (纯基础层，无 aidaguard-* 依赖)
├── 定义类型：DetectionRecord, AuditStats, AuditGroup, RuleCount
├── 定义 trait：AuditStorage, DetectionEngine
└── tokio, serde, thiserror, ...

aidaguard-storage (实现层)
├── aidaguard-core  ✅ 正向依赖
├── impl AuditStorage for Storage
└── rusqlite, aes-gcm, ...

aidaguard-detector (不变)
├── aidaguard-core
└── ...

aidaguard-proxy
├── aidaguard-core
├── aidaguard-detector
├── aidaguard-storage  (通过 trait 使用)
├── aidaguard-upstream
└── ...
```

### 3.2 类型迁移计划

| 类型 | 当前位置 | 迁移到 | 说明 |
|------|----------|--------|------|
| `DetectionRecord` | storage | **core** | 实体类型，属于核心 |
| `AuditStats` | storage | **core** | 统计类型 |
| `AuditGroup` | storage | **core** | 分组类型 |
| `RuleCount` | storage | **core** | 计数类型 |
| `Storage` | storage | storage | 实现类，保留在 storage |

### 3.3 新增 trait 定义

```rust
// aidaguard-core/src/storage.rs (新文件)
pub trait AuditStorage: Send + Sync {
    fn record(&self, ...) -> Result<(), StorageError>;
    fn batch_record(&self, ...) -> Result<usize, StorageError>;
    fn list(&self, ...) -> Result<Vec<DetectionRecord>, StorageError>;
    fn list_filtered(&self, ...) -> Result<Vec<DetectionRecord>, StorageError>;
    fn stats(&self) -> Result<AuditStats, StorageError>;
    fn delete(&self, id: &str) -> Result<bool, StorageError>;
    // ...
}
```

---

## 四、具体实施步骤

### Step 1: 迁移类型定义到 core

**创建新文件：**

```rust
// aidaguard-core/src/entity/storage_types.rs (新文件)

use serde::{Deserialize, Serialize};

/// 一条检测审计记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionRecord {
    pub id: String,
    pub timestamp_ms: i64,
    pub rule_id: String,
    pub rule_name: String,
    pub strategy: String,
    pub placeholder: String,
    pub original: String,
    pub context: String,
    pub request_path: String,
    pub sanitized_body: String,
    pub response_status: u16,
    pub tool_name: String,
}

/// 规则命中统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleCount {
    pub rule_id: String,
    pub rule_name: String,
    pub count: usize,
}

/// 按 (rule_id, strategy) 分组的审计摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditGroup {
    pub rule_id: String,
    pub rule_name: String,
    pub strategy: String,
    pub count: usize,
    pub latest_timestamp_ms: i64,
}

/// 审计汇总统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditStats {
    pub total_count: usize,
    pub today_count: usize,
    pub week_count: usize,
    pub rule_distribution: Vec<RuleCount>,
    pub db_size_bytes: u64,
}
```

**更新 entity.rs：**

```rust
// aidaguard-core/src/entity.rs

mod storage_types;

pub use storage_types::{DetectionRecord, RuleCount, AuditGroup, AuditStats};
// 保留现有的 EntityType, EntityCategory 等
```

### Step 2: 定义 AuditStorage trait

```rust
// aidaguard-core/src/storage.rs (新文件)

use crate::entity::{DetectionRecord, AuditStats, AuditGroup};
use crate::error::StorageError;

/// 审计存储接口
pub trait AuditStorage: Send + Sync {
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
    
    /// 批量记录
    fn batch_record(&self, records: &[DetectionRecord]) -> Result<usize, StorageError>;
    
    /// 分页查询
    fn list(&self, limit: usize, offset: usize) -> Result<Vec<DetectionRecord>, StorageError>;
    
    /// 条件查询
    fn list_filtered(
        &self,
        limit: usize,
        offset: usize,
        rule_id_filter: Option<&str>,
        path_filter: Option<&str>,
        date_from_ms: Option<i64>,
        date_to_ms: Option<i64>,
        strategy_filter: Option<&str>,
    ) -> Result<Vec<DetectionRecord>, StorageError>;
    
    /// 查询总数
    fn count(&self) -> Result<usize, StorageError>;
    
    /// 条件统计
    fn count_filtered(
        &self,
        rule_id_filter: Option<&str>,
        path_filter: Option<&str>,
        date_from_ms: Option<i64>,
        date_to_ms: Option<i64>,
        strategy_filter: Option<&str>,
    ) -> Result<usize, StorageError>;
    
    /// 分组查询
    fn list_grouped(
        &self,
        limit: usize,
        offset: usize,
        rule_id_filter: Option<&str>,
        path_filter: Option<&str>,
        date_from_ms: Option<i64>,
        date_to_ms: Option<i64>,
    ) -> Result<Vec<AuditGroup>, StorageError>;
    
    /// 分组计数
    fn count_grouped(
        &self,
        rule_id_filter: Option<&str>,
        path_filter: Option<&str>,
        date_from_ms: Option<i64>,
        date_to_ms: Option<i64>,
    ) -> Result<usize, StorageError>;
    
    /// 按ID查询
    fn get_by_id(&self, id: &str) -> Result<Option<DetectionRecord>, StorageError>;
    
    /// 删除记录
    fn delete(&self, id: &str) -> Result<bool, StorageError>;
    
    /// 统计信息
    fn stats(&self) -> Result<AuditStats, StorageError>;
    
    /// 最近记录
    fn list_recent(&self, limit: usize) -> Result<Vec<DetectionRecord>, StorageError>;
}
```

### Step 3: 更新 storage crate

```rust
// aidaguard-storage/src/lib.rs

// 导入 core 的类型
use aidaguard_core::entity::{DetectionRecord, RuleCount, AuditGroup, AuditStats};
use aidaguard_core::storage::AuditStorage;
use aidaguard_core::error::StorageError;

// 实现类型定义已移到 core，这里只保留 Storage 实现
pub struct Storage { ... }

impl AuditStorage for Storage {
    // 实现所有 trait 方法
    fn record(&self, ...) -> Result<(), StorageError> {
        // 现有实现
    }
    // ... 其他方法
}

// 重新导出 Storage
pub use Storage;
```

```toml
# aidaguard-storage/Cargo.toml

[dependencies]
aidaguard-core = { path = "../aidaguard-core" }  # 新增
# ... 其他依赖
```

### Step 4: 移除 core 对 storage 的依赖

```rust
// 删除 aidaguard-core/src/storage/mod.rs

// 更新 aidaguard-core/src/lib.rs
pub mod config;
pub mod detector;
pub mod engine;
pub mod entity;
pub mod error;
pub mod replacer;
pub mod storage;  // 新增（定义 trait）

// 导出
pub use entity::*;
pub use storage::AuditStorage;  // 新增
```

```toml
# aidaguard-core/Cargo.toml

[dependencies]
# 移除这行
# aidaguard-storage = { path = "../aidaguard-storage" }

# 保留其他依赖
tokio.workspace = true
serde.workspace = true
# ...
```

### Step 5: 更新依赖方

```rust
// aidaguard-proxy/src/server.rs

// 修改前
use aidaguard_storage::Storage;

// 修改后
use aidaguard_storage::Storage;
use aidaguard_core::storage::AuditStorage;  // 新增，通过 trait 使用
```

---

## 五、文件变更清单

### 5.1 aidaguard-core

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/entity/storage_types.rs` | **新增** | 迁移存储相关类型 |
| `src/entity.rs` | 修改 | 导出新类型 |
| `src/storage.rs` | **新增** | AuditStorage trait 定义 |
| `src/storage/mod.rs` | **删除** | 移除 re-export |
| `src/lib.rs` | 修改 | 更新导出 |
| `Cargo.toml` | 修改 | 移除 aidaguard-storage 依赖 |

### 5.2 aidaguard-storage

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/lib.rs` | 修改 | 导入 core 类型，实现 trait |
| `Cargo.toml` | 修改 | 添加 aidaguard-core 依赖 |

### 5.3 aidaguard-proxy

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/server.rs` | 修改 | 更新 import |
| `src/lib.rs` | 检查 | 确认无直接依赖 storage 类型 |

### 5.4 aidaguard-tauri

| 文件 | 操作 | 说明 |
|------|------|------|
| `src-tauri/src/main.rs` | 检查 | 更新 import 路径 |
| `src/src/*.tsx` | 检查 | 前端可能需要更新 API 调用 |

---

## 六、验证步骤

### 6.1 编译验证

```bash
# 清理并重新编译
cargo clean
cargo build --workspace

# 检查依赖树
cargo tree -p aidaguard-core
# 应该不再显示 aidaguard-storage

cargo tree -p aidaguard-storage
# 应该显示 aidaguard-core 作为依赖
```

### 6.2 测试验证

```bash
# 运行所有测试
cargo test --workspace

# 特定模块测试
cargo test -p aidaguard-core
cargo test -p aidaguard-storage
```

### 6.3 功能验证

```bash
# 启动代理服务
cargo run -p aidaguard-proxy

# 测试检测功能
curl -X POST http://localhost:7890/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"gpt-4","messages":[{"role":"user","content":"test"}]}'
```

---

## 七、风险评估

| 风险 | 等级 | 影响 | 缓解措施 |
|------|------|------|----------|
| 类型迁移遗漏 | 中 | 编译失败 | 使用 grep 检查所有使用点 |
| API 不兼容 | 低 | 调用方需要修改 | 保持接口签名不变 |
| 循环依赖 | 低 | 编译失败 | 确保 core 不依赖其他 aidaguard-* |
| 测试失败 | 中 | 功能回归 | 分步提交，每步验证 |

---

## 八、回滚方案

如果重构出现问题：

1. **保留旧文件**：先不删除 `aidaguard-core/src/storage/mod.rs`，改为 deprecated
2. **Git 分支**：在 feature 分支开发，验证后再合并
3. **增量提交**：每步一个 commit，便于定位问题

---

## 九、预期收益

| 收益 | 说明 |
|------|------|
| 架构清晰 | 依赖方向正确，core → storage 变为 storage → core |
| 编译加速 | 修改 storage 不再触发 core 重新编译 |
| 测试便利 | 可以为 core 编写独立测试，使用 Mock Storage |
| 扩展性 | 支持实现其他存储后端（Memory、Postgres） |
| 代码理解 | 新开发者更容易理解架构 |
