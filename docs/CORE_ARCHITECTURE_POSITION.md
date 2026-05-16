# AIDAGuard Core 架构定位分析

**日期：** 2026-05-16  
**版本：** v0.4.0

---

## 一、架构分层

```
┌─────────────────────────────────────────────────────────────────┐
│                     应用层 (Application Layer)                   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                 aidaguard-tauri                          │   │
│  │         Tauri 桌面应用 + React 前端                       │   │
│  └─────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                     服务层 (Service Layer)                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│  │   proxy     │  │  upstream   │  │  plugins    │            │
│  │  HTTP 代理  │  │  上游管理   │  │  工具适配   │            │
│  └─────────────┘  └─────────────┘  └─────────────┘            │
├─────────────────────────────────────────────────────────────────┤
│                     业务层 (Business Layer)                      │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                  aidaguard-detector                      │   │
│  │         检测引擎（正则 + NLP + 匿名化）                    │   │
│  └─────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                     基础层 (Foundation Layer)                    │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   aidaguard-core   ← 分析目标            │   │
│  │   类型定义 + 配置 + 接口 trait + 基础实现                 │   │
│  └─────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                  aidaguard-storage                       │   │
│  │              持久化存储（SQLite + 加密）                   │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 二、Core 的定位

### 2.1 定义

> **aidaguard-core 是基础层，提供核心类型定义、配置管理、接口抽象和基础实现。**

### 2.2 核心职责

| 职责 | 说明 | 对应模块 |
|------|------|----------|
| **类型定义** | 定义整个系统共享的核心数据类型 | `entity.rs` |
| **接口抽象** | 定义其他模块需要实现的 trait | `engine.rs` |
| **配置管理** | 集中管理所有配置项 | `config.rs` |
| **错误定义** | 定义结构化的错误类型 | `error.rs` |
| **基础实现** | 提供开箱即用的基础功能 | `detector/`, `replacer/` |

### 2.3 设计原则

```
Core 的设计原则：

1. 无业务依赖
   - core 不应依赖其他 aidaguard-* crate
   - 只依赖外部库（serde, regex, tokio 等）

2. 接口定义者
   - 定义 trait，由其他 crate 实现
   - 如：DetectionEngine trait → detector 实现

3. 类型提供者
   - 定义共享的数据结构
   - 如：EntityType, Config, DetectionRecord

4. 配置中心
   - 集中管理所有配置项
   - 提供默认值和验证

5. 错误规范化
   - 定义统一的错误类型
   - 提供恢复提示
```

---

## 三、Core 与其他 Crate 的关系

### 3.1 正确的依赖方向

```
                    ┌─────────────┐
                    │    tauri    │
                    └──────┬──────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
         ▼                 ▼                 ▼
    ┌─────────┐      ┌─────────┐      ┌─────────┐
    │  proxy  │      │upstream │      │ plugins │
    └────┬────┘      └────┬────┘      └─────────┘
         │                │
         │    ┌─────┐     │
         └───▶│core │◀────┘
              └──┬──┘
                 │
         ┌───────┴───────┐
         │               │
         ▼               ▼
    ┌──────────┐   ┌──────────┐
    │ detector │   │ storage  │
    └──────────┘   └──────────┘

依赖方向：所有 crate 依赖 core，core 不依赖任何业务 crate
```

### 3.2 当前问题

```
当前实际的依赖关系：

    aidaguard-core
         │
         ▼
    aidaguard-storage   ← 反向依赖！违反设计原则
```

### 3.3 各 Crate 对 Core 的使用

| Crate | 使用的 Core 内容 | 依赖类型 |
|-------|------------------|----------|
| `detector` | `DetectionEngine` trait, `entity.rs` 类型 | 接口实现 |
| `proxy` | `Config`, `DetectionEngine`, `replacer` | 服务组装 |
| `upstream` | `UpstreamConfig` | 配置使用 |
| `storage` | 应使用 `AuditStorage` trait, `DetectionRecord` | 接口实现 |
| `tauri` | `DetectionEvent`, 所有类型 | 应用集成 |

---

## 四、Core 内部结构

### 4.1 模块分层

```
aidaguard-core/
│
├── 接口层
│   ├── engine.rs          # DetectionEngine trait
│   └── storage.rs (待创建) # AuditStorage trait
│
├── 类型层
│   ├── entity.rs          # EntityType, EntityCategory
│   └── storage_types (待迁移) # DetectionRecord, AuditStats
│
├── 配置层
│   └── config.rs          # Config, NlpConfig, UpstreamConfig
│
├── 错误层
│   └── error.rs           # DetectionError, StorageError, ProxyError
│
├── 实现层
│   ├── detector/          # Detector, RuleDef, Match
│   └── replacer/          # PlaceholderMap, replace, restore
│
└── 入口
    └── lib.rs             # 模块组织，公共导出
```

### 4.2 模块职责

```
┌────────────────────────────────────────────────────────────────┐
│                          aidaguard-core                         │
├────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ 接口层 - 定义 trait                                      │   │
│  │                                                         │   │
│  │   DetectionEngine trait                                 │   │
│  │     - detect()                                          │   │
│  │     - rule_count()                                      │   │
│  │     - reload()                                          │   │
│  │                                                         │   │
│  │   AuditStorage trait (待创建)                           │   │
│  │     - record()                                          │   │
│  │     - list()                                            │   │
│  │     - stats()                                           │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ 类型层 - 定义共享数据结构                                │   │
│  │                                                         │   │
│  │   EntityType (40+ 种)                                   │   │
│  │   EntityCategory (Structured/Unstructured/Network)     │   │
│  │   DetectionRecord (待迁移)                              │   │
│  │   AuditStats (待迁移)                                   │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ 配置层 - 集中管理配置                                    │   │
│  │                                                         │   │
│  │   Config (端口、规则目录、上游、NLP、存储)              │   │
│  │   NlpConfig (模型、语言、启用状态)                      │   │
│  │   UpstreamConfig (URL、API Key、协议)                   │   │
│  │   StorageConfig (路径、加密密钥)                        │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ 错误层 - 结构化错误                                      │   │
│  │                                                         │   │
│  │   DetectionError + recovery_hint()                      │   │
│  │   StorageError + recovery_hint()                        │   │
│  │   ProxyError + recovery_hint()                          │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ 实现层 - 提供基础实现                                    │   │
│  │                                                         │   │
│  │   Detector - 基础正则检测器                             │   │
│  │   RuleDef, CompiledRule, Match                          │   │
│  │   VersionedDetector - 版本化管理                        │   │
│  │                                                         │   │
│  │   PlaceholderMap - 占位符映射                           │   │
│  │   replace() / restore() - 替换/还原                     │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```

---

## 五、Core 的边界

### 5.1 Core 应该包含

| 应该包含 | 原因 |
|----------|------|
| 公共类型定义 | 被多个 crate 使用 |
| 接口 trait | 定义抽象，由其他 crate 实现 |
| 配置管理 | 集中配置，避免散落各处 |
| 错误定义 | 统一错误处理 |
| 基础实现 | 提供开箱即用的功能 |

### 5.2 Core 不应包含

| 不应包含 | 原因 | 应该放在 |
|----------|------|----------|
| 具体存储实现 | 是实现细节 | `aidaguard-storage` |
| NLP 模型加载 | 重量级依赖 | `aidaguard-detector` |
| HTTP 服务逻辑 | 业务逻辑 | `aidaguard-proxy` |
| 工具适配逻辑 | 业务逻辑 | `aidaguard-plugins` |
| UI 相关代码 | 应用层 | `aidaguard-tauri` |

---

## 六、重构目标

### 6.1 当前问题

```
问题：core → storage 反向依赖

aidaguard-core/Cargo.toml:
  aidaguard-storage = { path = "../aidaguard-storage" }

aidaguard-core/src/storage/mod.rs:
  pub use aidaguard_storage::*;
```

### 6.2 重构目标

```
目标：消除反向依赖，让 storage 实现 core 定义的 trait

aidaguard-core/src/storage.rs:
  pub trait AuditStorage { ... }
  // 无 aidaguard-storage 依赖

aidaguard-storage/src/lib.rs:
  use aidaguard_core::storage::AuditStorage;
  impl AuditStorage for Storage { ... }
```

### 6.3 重构后的依赖图

```
重构后：

    aidaguard-core (无内部依赖)
         ▲
         │
    ┌────┴────┐
    │         │
    ▼         ▼
detector   storage
    │         │
    └────┬────┘
         │
         ▼
      proxy
         │
         ▼
       tauri

所有依赖方向正确：从外向内，指向 core
```

---

## 七、Core 定位总结

### 7.1 一句话定义

> **aidaguard-core 是架构的基础层，定义类型、配置和接口，不依赖任何业务模块。**

### 7.2 核心价值

| 价值 | 说明 |
|------|------|
| **解耦** | 定义接口，隔离实现 |
| **复用** | 共享类型，避免重复定义 |
| **统一** | 集中配置，便于管理 |
| **规范** | 错误类型 + 恢复提示 |

### 7.3 设计约束

```
Core 的设计约束：

1. ✅ 可以依赖外部库
2. ❌ 不能依赖 aidaguard-* crate
3. ✅ 定义 trait 供其他 crate 实现
4. ✅ 提供基础实现（如 Detector）
5. ❌ 不包含重量级依赖（如 NLP 模型）
6. ✅ 保持稳定，修改需谨慎
```

### 7.4 重构意义

重构 `storage/` 模块消除反向依赖后：

1. **架构清晰** - 依赖方向统一，易于理解
2. **编译加速** - 修改 storage 不触发 core 重编译
3. **测试便利** - core 可独立测试
4. **扩展灵活** - 可实现其他存储后端
5. **符合原则** - core 真正成为基础层
