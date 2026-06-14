# AIDAGuard Core 模块分析与重构建议

**日期：** 2026-05-16  
**版本：** v0.4.0

---

## 一、Core 模块总览

```
aidaguard-core/src/
├── lib.rs           # 模块入口，版本定义，DetectionEvent
├── config.rs        # 配置类型定义
├── entity.rs        # 实体类型（EntityType, EntityCategory）
├── engine.rs        # DetectionEngine trait
├── error.rs         # 错误类型定义
├── detector/        # 检测器相关
│   ├── mod.rs       # Detector, RuleDef, CompiledRule, Match
│   └── versioned.rs # RuleSnapshot, VersionedDetector
├── replacer/        # 替换器
│   └── mod.rs       # PlaceholderMap, replace(), restore()
└── storage/         # 存储相关（问题模块）
    └── mod.rs       # pub use aidaguard_storage::*; ← 反向依赖
```

---

## 二、各模块功能分析

### 2.1 config.rs - 配置模块

**功能：**
- 定义配置结构体：`Config`, `StorageConfig`, `UpstreamConfig`, `NlpConfig`
- 提供默认值函数
- 配置加载和验证

**依赖：**
- `serde`, `serde_yaml`, `toml` - 序列化
- 无其他 aidaguard-* 依赖 ✅

**状态：** ✅ **无需重构**

---

### 2.2 entity.rs - 实体类型模块

**功能：**
- 定义敏感数据类型：`EntityType`（40+ 种实体）
- 定义实体分类：`EntityCategory`（Structured/Unstructured/Network）
- 实体类型与分类的映射关系

**依赖：**
- `serde` - 序列化
- 无其他 aidaguard-* 依赖 ✅

**状态：** ✅ **无需重构**

**注意：** 存储相关类型（`DetectionRecord`, `AuditStats` 等）目前在 `aidaguard-storage` 中，需要迁移到此处。

---

### 2.3 engine.rs - 检测引擎接口

**功能：**
- 定义 `DetectionEngine` trait
- 抽象检测、规则管理、重载接口

```rust
pub trait DetectionEngine: Send + Sync {
    fn detect(&self, text: &str) -> Vec<Match>;
    fn rule_count(&self) -> usize;
    fn rule_name(&self, id: &str) -> Option<&str>;
    fn reload(&mut self, dir: &Path) -> Result<usize, anyhow::Error>;
    fn reload_presets(&mut self, base_dir: &Path, presets: &[String]) -> Result<usize, anyhow::Error>;
}
```

**依赖：**
- `anyhow` - 错误处理
- 无其他 aidaguard-* 依赖 ✅

**状态：** ✅ **无需重构**

---

### 2.4 error.rs - 错误类型模块

**功能：**
- 定义结构化错误类型：`DetectionError`, `StorageError`, `ProxyError`
- 提供恢复提示 `recovery_hint()`

**依赖：**
- `thiserror` - 错误派生
- 无其他 aidaguard-* 依赖 ✅

**状态：** ✅ **无需重构**

---

### 2.5 detector/ - 检测器模块

#### 2.5.1 mod.rs - 基础检测器

**功能：**
- `Detector` - 基础正则检测器
- `RuleDef` - 规则定义（YAML 结构）
- `CompiledRule` - 编译后规则
- `Match` - 检测命中
- `Strategy` / `Mode` - 替换策略和模式
- `watch_rules()` - 规则热加载

**依赖：**
- `regex` - 正则匹配
- `notify` - 文件监听
- `serde`, `serde_yaml` - 规则解析
- 无其他 aidaguard-* 依赖 ✅

**状态：** ✅ **无需重构**

#### 2.5.2 versioned.rs - 版本化检测器

**功能：**
- `RuleSnapshot` - 规则快照
- `VersionedDetector` - 支持原子切换和回滚的版本化检测器
- 规则完整性校验（SHA-256 checksum）

**依赖：**
- `sha2` - 哈希计算
- 无其他 aidaguard-* 依赖 ✅

**状态：** ✅ **无需重构**

---

### 2.6 replacer/ - 替换器模块

**功能：**
- `PlaceholderMap` - 占位符到原始文本的映射
- `replace()` - 替换匹配项为占位符或掩码
- `restore()` - 还原占位符为原始文本
- `mask_value()` - 部分掩码处理

**依赖：**
- `uuid` - 占位符 ID 生成
- 无其他 aidaguard-* 依赖 ✅

**状态：** ✅ **无需重构**

---

### 2.7 storage/ - 存储模块 ❌

**功能：**
- 重新导出 `aidaguard-storage` 的所有类型

**问题：**
```rust
// aidaguard-core/src/storage/mod.rs
pub use aidaguard_storage::*;  // ← 反向依赖
```

**依赖：**
- `aidaguard-storage` ← **反向依赖！**

**状态：** ❌ **需要重构**

---

## 三、问题总结

### 3.1 需要重构的模块

| 模块 | 问题 | 优先级 |
|------|------|--------|
| `storage/` | 反向依赖 aidaguard-storage | **P1** |

### 3.2 无需重构的模块

| 模块 | 说明 |
|------|------|
| `config.rs` | 纯配置定义，无跨 crate 依赖 |
| `entity.rs` | 纯类型定义，但需要**迁移存储类型到此** |
| `engine.rs` | trait 定义，设计正确 |
| `error.rs` | 错误类型，设计正确 |
| `detector/` | 完整的检测器实现，无跨 crate 依赖 |
| `replacer/` | 纯文本处理，无跨 crate 依赖 |

---

## 四、重构方案

### 4.1 storage/ 模块重构

**目标：** 将 storage/ 从"re-export"改为"trait 定义"

**变更：**

```
重构前：
aidaguard-core/src/storage/mod.rs
└── pub use aidaguard_storage::*;  // 反向依赖

重构后：
aidaguard-core/src/storage.rs
├── pub trait AuditStorage { ... }
├── pub struct AuditFilter { ... }
└── pub struct AuditStats { ... }
```

### 4.2 类型迁移

**需要迁移到 core 的类型：**

| 类型 | 当前位置 | 迁移到 | 原因 |
|------|----------|--------|------|
| `DetectionRecord` | storage | `entity.rs` | 核心实体类型 |
| `AuditStats` | storage | `entity.rs` | 统计类型 |
| `AuditGroup` | storage | `entity.rs` | 分组类型 |
| `RuleCount` | storage | `entity.rs` | 计数类型 |

**保留在 storage 的内容：**

| 类型 | 位置 | 原因 |
|------|------|------|
| `Storage` | storage | 具体实现类 |
| 加密相关函数 | storage | 实现细节 |

---

## 五、重构后的模块结构

```
aidaguard-core/src/
├── lib.rs
├── config.rs           # 配置（不变）
├── entity.rs           # 实体类型 + 迁移的存储类型
│   ├── EntityType
│   ├── EntityCategory
│   ├── DetectionRecord  ← 从 storage 迁移
│   ├── AuditStats       ← 从 storage 迁移
│   ├── AuditGroup       ← 从 storage 迁移
│   └── RuleCount        ← 从 storage 迁移
├── engine.rs           # DetectionEngine trait（不变）
├── error.rs            # 错误类型（不变）
├── detector/           # 检测器（不变）
│   ├── mod.rs
│   └── versioned.rs
├── replacer/           # 替换器（不变）
│   └── mod.rs
└── storage.rs          # AuditStorage trait（新）
    ├── AuditStorage trait
    ├── AuditFilter
    └── 导入 entity 中的类型

aidaguard-core/Cargo.toml
├── 移除：aidaguard-storage 依赖
└── 保留：tokio, serde, regex, thiserror, ...

aidaguard-storage/src/
├── lib.rs              # impl AuditStorage for Storage
├── sqlite.rs           # SQLite 实现（从 lib.rs 拆分）
└── crypto.rs           # 加密相关（从 lib.rs 拆分）

aidaguard-storage/Cargo.toml
├── 新增：aidaguard-core = { path = "../aidaguard-core" }
└── 保留：rusqlite, aes-gcm, ...
```

---

## 六、其他 crate 对 core 的依赖情况

### 6.1 aidaguard-detector

```toml
[dependencies]
aidaguard-core = { path = "../aidaguard-core" }
```

**使用情况：**
- 使用 `DetectionEngine` trait
- 使用 `entity.rs` 的类型
- **状态：** ✅ 正确

### 6.2 aidaguard-proxy

```toml
[dependencies]
aidaguard-core = { path = "../aidaguard-core" }
aidaguard-detector = { path = "../aidaguard-detector" }
aidaguard-storage = { path = "../aidaguard-storage" }
aidaguard-upstream = { path = "../aidaguard-upstream" }
```

**使用情况：**
- 使用 `DetectionEngine` trait
- 使用 `config::Config`
- 使用 `Storage`（直接依赖 storage crate）
- **状态：** ✅ 正确（直接依赖 storage 是允许的）

### 6.3 aidaguard-upstream

```toml
[dependencies]
aidaguard-core = { path = "../aidaguard-core" }
```

**使用情况：**
- 使用 `config::UpstreamConfig`
- **状态：** ✅ 正确

### 6.4 aidaguard-tauri

```toml
[dependencies]
aidaguard-core = { path = "../aidaguard-core" }
aidaguard-detector = { path = "../aidaguard-detector" }
aidaguard-proxy = { path = "../aidaguard-proxy" }
aidaguard-storage = { path = "../aidaguard-storage" }
```

**使用情况：**
- 使用 `DetectionRecord`, `AuditStats` 等（需要更新 import 路径）
- **状态：** ⚠️ 重构后需更新 import

---

## 七、重构步骤

### Step 1: 创建 entity/storage_types.rs

```rust
// aidaguard-core/src/entity/storage_types.rs
// 迁移 DetectionRecord, AuditStats, AuditGroup, RuleCount
```

### Step 2: 创建 storage.rs (trait 定义)

```rust
// aidaguard-core/src/storage.rs
use crate::entity::{DetectionRecord, AuditStats, AuditGroup};

pub trait AuditStorage: Send + Sync {
    // 所有方法签名
}
```

### Step 3: 更新 storage crate

```rust
// aidaguard-storage/src/lib.rs
use aidaguard_core::storage::AuditStorage;
use aidaguard_core::entity::{DetectionRecord, ...};

impl AuditStorage for Storage { ... }
```

### Step 4: 删除反向依赖

```rust
// 删除 aidaguard-core/src/storage/mod.rs
// 更新 aidaguard-core/Cargo.toml，移除 aidaguard-storage 依赖
```

### Step 5: 更新所有 import

```rust
// aidaguard-tauri 和其他使用方
// 修改前
use aidaguard_core::storage::DetectionRecord;
// 修改后
use aidaguard_core::entity::DetectionRecord;
```

---

## 八、总结

### 8.1 Core 模块质量评估

| 模块 | 设计质量 | 需要修改 |
|------|----------|----------|
| config.rs | ✅ 优秀 | 否 |
| entity.rs | ✅ 优秀 | 是（迁移类型） |
| engine.rs | ✅ 优秀 | 否 |
| error.rs | ✅ 优秀 | 否 |
| detector/ | ✅ 优秀 | 否 |
| replacer/ | ✅ 优秀 | 否 |
| storage/ | ❌ 问题 | **是** |

### 8.2 重构范围

**仅需重构 1 个模块：** `storage/`

**其他 6 个模块无需修改：**
- config.rs
- engine.rs
- error.rs
- detector/
- replacer/
- entity.rs（仅扩展，非重构）

### 8.3 预期收益

1. **架构清晰** - 依赖方向正确
2. **编译加速** - 修改 storage 不再触发 core 重编译
3. **测试便利** - core 可独立测试
4. **扩展性** - 支持实现其他存储后端
