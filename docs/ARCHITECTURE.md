# Aidaguard 项目架构

**版本：** 0.5.0
**状态：** 活跃开发中

7 个 crate 的 Rust 工作空间 + Tauri 2.x 桌面前端（React 18 + shadcn/ui + Tailwind CSS）。

## 目录

- [概述](#概述)
- [架构分层](#架构分层)
- [Crate 依赖关系图](#crate-依赖关系图)
- [各 Crate 详解](#各-crate-详解)
  - [1. aidaguard-core — 基础层](#1-aidaguard-core--基础层)
  - [2. aidaguard-detector — 检测管线](#2-aidaguard-detector--检测管线)
  - [3. aidaguard-proxy — HTTP 代理引擎](#3-aidaguard-proxy--http-代理引擎)
  - [4. aidaguard-upstream — LLM 提供商管理](#4-aidaguard-upstream--llm-提供商管理)
  - [5. aidaguard-plugins — AI 工具适配器](#5-aidaguard-plugins--ai-工具适配器)
  - [6. aidaguard-storage — 加密审计日志](#6-aidaguard-storage--加密审计日志)
  - [7. aidaguard-tauri — 桌面应用](#7-aidaguard-tauri--桌面应用)
- [规则目录布局](#规则目录布局)
- [检测策略](#检测策略)
- [测试](#测试)
- [配置文件](#配置文件)
- [特性开关](#特性开关)
- [关键设计决策](#关键设计决策)
- [架构优化方案](#架构优化方案)

---

## 概述

Aidaguard 是一个本地 LLM 代理，在敏感数据离开你的设备之前对其进行检测和脱敏。它通过正则模式识别、校验和验证以及基于 BERT 的 NLP NER 来扫描发出的 API 请求中的 PII、凭据和其他敏感实体。检测结果审计记录在加密的 SQLite 存储中。桌面 UI 负责管理配置、规则、上游 LLM 提供商和 AI 工具插件适配器。

## 架构分层

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
│  │                   aidaguard-core                         │   │
│  │   类型定义 + 配置 + 接口 trait + 基础实现                 │   │
│  └─────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                  aidaguard-storage                       │   │
│  │              持久化存储（SQLite + 加密）                   │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Crate 依赖关系图

```
aidaguard-tauri (Tauri 桌面应用)
  ├── aidaguard-core
  ├── aidaguard-detector
  ├── aidaguard-storage
  ├── aidaguard-plugins
  └── aidaguard-proxy

aidaguard-proxy (HTTP 代理服务器)
  ├── aidaguard-core
  ├── aidaguard-detector
  ├── aidaguard-storage
  └── aidaguard-upstream

aidaguard-detector (检测引擎)
  └── aidaguard-core

aidaguard-upstream (LLM 提供商管理)
  └── aidaguard-core

aidaguard-plugins (AI 工具适配器)
  （无内部依赖）

aidaguard-storage (加密审计数据库)
  （无内部依赖）

aidaguard-core (基础类型 + 配置)
  └── aidaguard-storage (重新导出)  ← 反向依赖，需重构
```

**正确的依赖方向：** 所有 crate 应依赖 core，core 不依赖任何业务 crate。

```
                ┌─────────────┐
                │    tauri    │
                └──────┬──────┘
                       │
         ┌─────────────┼─────────────┐
         │             │             │
         ▼             ▼             ▼
    ┌─────────┐   ┌─────────┐   ┌─────────┐
    │  proxy  │   │upstream │   │ plugins │
    └────┬────┘   └────┬────┘   └─────────┘
         │             │
         │    ┌─────┐  │
         └───▶│core │◀─┘
              └──┬──┘
                 │
         ┌───────┴───────┐
         │               │
         ▼               ▼
    ┌──────────┐   ┌──────────┐
    │ detector │   │ storage  │
    └──────────┘   └──────────┘
```

---

## 各 Crate 详解

### 1. `aidaguard-core` — 基础层

**功能：** 核心类型、配置、正则检测引擎，以及所有检测后端实现的 `DetectionEngine` 接口。

#### Core 的定位

> **aidaguard-core 是架构的基础层，定义类型、配置和接口，不依赖任何业务模块。**

**核心职责：**

| 职责 | 说明 | 对应模块 |
|------|------|----------|
| **类型定义** | 定义整个系统共享的核心数据类型 | `entity.rs` |
| **接口抽象** | 定义其他模块需要实现的 trait | `engine.rs` |
| **配置管理** | 集中管理所有配置项 | `config.rs` |
| **错误定义** | 定义结构化的错误类型 | `error.rs` |
| **基础实现** | 提供开箱即用的基础功能 | `detector/`, `replacer/` |

**设计原则：**

1. 无业务依赖 — core 不应依赖其他 aidaguard-* crate，只依赖外部库（serde, regex, tokio 等）
2. 接口定义者 — 定义 trait，由其他 crate 实现（如：DetectionEngine trait → detector 实现）
3. 类型提供者 — 定义共享的数据结构（如：EntityType, Config, DetectionRecord）
4. 配置中心 — 集中管理所有配置项，提供默认值和验证
5. 错误规范化 — 定义统一的错误类型，提供恢复提示

**设计约束：**

- 可以依赖外部库
- 不能依赖 aidaguard-* crate（当前违反：依赖 aidaguard-storage）
- 定义 trait 供其他 crate 实现
- 提供基础实现（如 Detector）
- 不包含重量级依赖（如 NLP 模型）
- 保持稳定，修改需谨慎

#### Core 内部结构

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

**模块分层：**

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

#### 各模块分析

##### config.rs - 配置模块

- 定义配置结构体：`Config`, `StorageConfig`, `UpstreamConfig`, `NlpConfig`
- 提供默认值函数
- 配置加载和验证
- 依赖：`serde`, `serde_yaml`, `toml`，无其他 aidaguard-* 依赖
- 状态：无需重构

##### entity.rs - 实体类型模块

- 定义敏感数据类型：`EntityType`（40+ 种实体）
- 定义实体分类：`EntityCategory`（Structured/Unstructured/Network）
- 实体类型与分类的映射关系
- 依赖：`serde`，无其他 aidaguard-* 依赖
- 状态：无需重构（但存储相关类型 `DetectionRecord`, `AuditStats` 等需从 storage 迁移到此）

##### engine.rs - 检测引擎接口

- 定义 `DetectionEngine` trait，抽象检测、规则管理、重载接口
- 依赖：`anyhow`，无其他 aidaguard-* 依赖
- 状态：无需重构

```rust
pub trait DetectionEngine: Send + Sync {
    fn detect(&self, text: &str) -> Vec<Match>;
    fn rule_count(&self) -> usize;
    fn rule_name(&self, id: &str) -> Option<&str>;
    fn reload(&mut self, dir: &Path) -> Result<usize, anyhow::Error>;
    fn reload_presets(&mut self, base_dir: &Path, presets: &[String]) -> Result<usize, anyhow::Error>;
}
```

##### error.rs - 错误类型模块

- 定义结构化错误类型：`DetectionError`, `StorageError`, `ProxyError`
- 提供恢复提示 `recovery_hint()`
- 依赖：`thiserror`，无其他 aidaguard-* 依赖
- 状态：无需重构

##### detector/ - 检测器模块

- `mod.rs` — `Detector`（基础正则检测器）、`RuleDef`、`CompiledRule`、`Match`、`Strategy`/`Mode` 枚举、`watch_rules()` 热加载
- `versioned.rs` — `RuleSnapshot`（规则快照）、`VersionedDetector`（支持原子切换和回滚的版本化检测器）、规则完整性校验（SHA-256 checksum）
- 依赖：`regex`, `notify`, `serde`, `serde_yaml`, `sha2`，无其他 aidaguard-* 依赖
- 状态：无需重构

##### replacer/ - 替换器模块

- `PlaceholderMap` — 占位符到原始文本的映射
- `replace()` — 替换匹配项为占位符或掩码
- `restore()` — 还原占位符为原始文本
- `mask_value()` — 部分掩码处理
- 依赖：`uuid`，无其他 aidaguard-* 依赖
- 状态：无需重构

##### storage/ - 存储模块 (需重构)

- 当前功能：重新导出 `aidaguard-storage` 的所有类型
- 问题：`pub use aidaguard_storage::*;` 构成 core → storage 反向依赖，违反设计原则
- 状态：需要重构

#### 反向依赖问题

当前 `aidaguard-core` 依赖 `aidaguard-storage`：

```rust
// aidaguard-core/Cargo.toml
aidaguard-storage = { path = "../aidaguard-storage" }

// aidaguard-core/src/storage/mod.rs
pub use aidaguard_storage::*;  // ← 反向依赖
```

**影响：**
- 依赖图不够清晰，违背 core 作为基础层的定位
- 潜在的循环依赖风险
- 增加编译时间（修改 storage 触发 core 重编译）
- core 无法独立测试

**重构目标：** 消除反向依赖，让 storage 实现 core 定义的 trait。

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

**类型迁移计划：**

| 类型 | 当前位置 | 迁移到 | 原因 |
|------|----------|--------|------|
| `DetectionRecord` | storage | `entity.rs` | 核心实体类型 |
| `AuditStats` | storage | `entity.rs` | 统计类型 |
| `AuditGroup` | storage | `entity.rs` | 分组类型 |
| `RuleCount` | storage | `entity.rs` | 计数类型 |

**重构后依赖图：**

```
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

#### 关键文件

| 文件 | 说明 |
|------|------|
| [config.rs](../crates/aidaguard-core/src/config.rs) | `Config` 结构体（端口、规则目录、上游、NLP、存储、地区/行业预设）；`Config::load()`、`Config::rule_presets()`、TOML 序列化 |
| [engine.rs](../crates/aidaguard-core/src/engine.rs) | `DetectionEngine` trait — `detect()`、`rule_count()`、`rule_name()`、`reload()`、`reload_presets()` |
| [detector/mod.rs](../crates/aidaguard-core/src/detector/mod.rs) | `Detector` 结构体 — 基于正则的 YAML 规则引擎；`RuleDef`、`CompiledRule`、`Match`、`Strategy`/`Mode` 枚举；带 ReDoS 保护的 `compile_regex()`；`watch_rules()` 热加载 |
| [entity.rs](../crates/aidaguard-core/src/entity.rs) | `EntityType` 枚举（30 个变体，分为 Structured/Unstructured/Network 三类）；`EntityCategory` 枚举；SCREAMING_SNAKE_CASE 字符串 ID |
| [replacer/mod.rs](../crates/aidaguard-core/src/replacer/mod.rs) | `PlaceholderMap` — UUID 占位符生成/还原；`replace()` — 匹配项替换；`restore()` — 占位符复原；`mask_value()` — 部分掩码 |
| [storage/mod.rs](../crates/aidaguard-core/src/storage/mod.rs) | 重新导出 `aidaguard-storage` 的类型 |

#### 对外导出

- `DetectionEngine` trait
- `EntityType`、`EntityCategory` 枚举
- `DetectionEvent` 结构体（代理 → 前端广播）
- `config::Config`、`config::NlpConfig`
- `detector::Detector`、`detector::Match`
- `replacer::PlaceholderMap`、`replacer::replace`、`replacer::restore`

#### Core 的边界

**Core 应该包含：**

| 应该包含 | 原因 |
|----------|------|
| 公共类型定义 | 被多个 crate 使用 |
| 接口 trait | 定义抽象，由其他 crate 实现 |
| 配置管理 | 集中配置，避免散落各处 |
| 错误定义 | 统一错误处理 |
| 基础实现 | 提供开箱即用的功能 |

**Core 不应包含：**

| 不应包含 | 原因 | 应该放在 |
|----------|------|----------|
| 具体存储实现 | 是实现细节 | `aidaguard-storage` |
| NLP 模型加载 | 重量级依赖 | `aidaguard-detector` |
| HTTP 服务逻辑 | 业务逻辑 | `aidaguard-proxy` |
| 工具适配逻辑 | 业务逻辑 | `aidaguard-plugins` |
| UI 相关代码 | 应用层 | `aidaguard-tauri` |

#### 各 Crate 对 Core 的使用

| Crate | 使用的 Core 内容 | 依赖类型 |
|-------|------------------|----------|
| `detector` | `DetectionEngine` trait, `entity.rs` 类型 | 接口实现 |
| `proxy` | `Config`, `DetectionEngine`, `replacer` | 服务组装 |
| `upstream` | `UpstreamConfig` | 配置使用 |
| `storage` | 应使用 `AuditStorage` trait, `DetectionRecord` | 接口实现 |
| `tauri` | `DetectionEvent`, 所有类型 | 应用集成 |

---

### 2. `aidaguard-detector` — 检测管线

**功能：** 多策略检测引擎，组合了正则模式识别器、校验和验证器、上下文评分以及可选的基于 BERT 的 NLP NER。

**特性开关：**
- `nlp` — 启用 `candle`/`transformers` 机器学习依赖（首次使用需下载约 400 MB 模型）

**模块结构：**

```
recognizers/
├── mod.rs              → pub mod pattern; #[cfg(feature = "nlp")] pub mod nlp;
├── pattern/
│   ├── mod.rs          → 子模块声明
│   ├── pattern_recognizer.rs → PatternRecognizer 结构体（正则 + 上下文词评分）
│   ├── credit_card.rs  → VISA/MC/Amex/Discover 正则 + Luhn 校验
│   ├── email.rs        → RFC 5322 邮箱正则
│   ├── phone.rs        → 国际电话号码
│   ├── id_card_cn.rs   → 中国身份证号 + 校验和
│   ├── passport_cn.rs  → 中国护照
│   ├── us_ssn.rs       → 美国社会安全号
│   ├── uk_nino.rs      → 英国国民保险号
│   ├── iban.rs         → 国际银行账号
│   ├── swift_code.rs   → SWIFT/BIC 代码
│   ├── car_plate.rs    → 车牌号
│   ├── bank_account.rs → 银行账号
│   ├── amount.rs       → 金额
│   ├── crypto_address.rs → 加密货币地址
│   ├── ip_address.rs   → IPv4/IPv6 地址
│   ├── mac_address.rs  → MAC 地址
│   ├── url.rs          → URL
│   ├── api_key.rs      → 通用 API 密钥
│   ├── jwt.rs          → JSON Web Token
│   ├── aws_access_key.rs → AWS 访问密钥
│   └── private_key.rs  → RSA/SSH 私钥
└── nlp/  (nlp feature 启用时编译)
    ├── mod.rs          → NlpRecognizer 结构体、InferenceCache、延迟模型加载
    ├── registry.rs     → ModelRegistry 单例（hf-hub 下载 + candle safetensors 加载）
    ├── engine.rs       → NlpEngine（BERT 分词 → 前向传播 → softmax → BIO 解码 → 实体片段）
    └── mapping.rs      → LabelMapping（B-PER/I-PER→PersonName, B-LOC/I-LOC→Address 等）

core/
├── mod.rs
├── recognizer.rs       → Recognizer trait (entity_type(), name(), analyze(), context_words(), supported_languages())
├── recognizer_registry.rs → RecognizerRegistry (register, load_predefined, load_nlp_recognizers, analyze_all)
├── result.rs           → RecognizerResult (entity_type, start, end, text, score, recognizer_name)
└── confidence.rs       → ConfidenceScorer（上下文词加分、重叠项仲裁）

validation/
├── mod.rs
├── luhn.rs             → Luhn 算法（信用卡/身份证校验）
├── mod_n.rs            → Mod-N 校验（中国身份证）
├── iban.rs             → IBAN 结构验证
├── id_card_cn.rs       → 中国身份证校验和验证
└── context.rs          → 上下文词验证框架

anonymizer/
├── mod.rs              → AnonymizerOperator 枚举 (Replace/Mask/Hash/Encrypt/Redact)
├── replace.rs          → 基于占位符的替换
├── mask.rs             → 部分字符掩码
├── hash.rs             → SHA-256 截断哈希
└── encrypt.rs          → AES-256-GCM 加密

pipeline.rs             → AnalyzerEngine + AnalyzerEngineBuilder
```

**核心类型：**

- `Recognizer` trait — 所有检测器（模式识别器、NLP 识别器）均实现此接口
- `PatternRecognizer` — 正则 + 上下文词评分 + 可选校验和验证
- `NlpRecognizer` — 用于非结构化实体的 BERT NER（人名、地址、机构名等）
- `AnalyzerEngine` — 协调识别器注册表 + 传统 YAML 规则 + 重叠仲裁 + 置信度过滤
- `AnalyzerEngineBuilder` — 引擎构建的 Builder 模式（`with_all_pattern_recognizers()`、`with_nlp_config()`、`with_config_rules()` 等）

**检测管线流程：**
1. 传统 YAML 正则规则（来自 `rules_dir` 预设目录）
2. 模式识别器（正则 + 校验和 + 上下文词）
3. NLP NER 识别器（BERT 推理，按文本缓存结果）
4. 重叠项仲裁（置信度高的优先）
5. 最低置信度过滤

**NLP 模型：**

| 语言 | HuggingFace 模型 | 标签 |
|----------|------------------|--------|
| `en` | `dslim/bert-base-NER` | PER, LOC, ORG, MISC |
| `zh` | `ckiplab/bert-base-chinese-ner` | PER, LOC, ORG |

---

### 3. `aidaguard-proxy` — HTTP 代理引擎

**功能：** 基于 Axum 的反向代理，拦截 LLM API 请求，检测敏感数据，替换为占位符，转发到真实的 LLM 端点，并在响应中还原占位符。

**关键文件：**

| 文件 | 说明 |
|------|------|
| [server.rs](../crates/aidaguard-proxy/src/server.rs) | Axum HTTP 服务器：`/health` 端点、`proxy_handler` — 检测→替换→转发→还原管线、`start()` 和 `start_with_state()` 入口函数 |
| [forwarder.rs](../crates/aidaguard-proxy/src/forwarder.rs) | `Forwarder` — 封装 reqwest 的 HTTP 客户端，注入认证头 |
| [stream.rs](../crates/aidaguard-proxy/src/stream.rs) | SSE 流式直通，对流式 LLM 响应进行占位符还原 |

**请求处理流程（每个请求）：**
1. 从 User-Agent 头提取工具名称
2. 构建目标 URL（路径透传）
3. 剥离 hop-by-hop 头和原始 Authorization
4. 读取请求体（受大小限制）
5. 从 JSON 请求体中提取模型名称
6. 对请求体文本运行 `AnalyzerEngine::detect()`
7. 区分 filter 模式命中（替换）和 detect 模式命中（仅记录）
8. 对请求体执行占位符/掩码替换
9. 将修改后的请求转发到上游 LLM
10. 在加密存储中记录审计事件
11. 向 Tauri 前端广播检测事件
12. 流式请求：SSE 直通 + 占位符还原
13. 非流式请求：读取完整响应，还原占位符，返回

---

### 4. `aidaguard-upstream` — LLM 提供商管理

**功能：** 声明式 LLM 提供商定义（OpenAI、Anthropic、DeepSeek、Qwen、Zhipu、Groq、Gemini）、上游配置和统一 HTTP 客户端。

**关键文件：**

| 文件 | 说明 |
|------|------|
| [types.rs](../crates/aidaguard-upstream/src/types.rs) | `ProviderConfig`、`UpstreamConfig`、`ModelInfo`、`ProtocolType`（OpenAiCompatible/AnthropicCompatible）、`AuthType`（BearerToken/ApiKeyHeader） |
| [provider.rs](../crates/aidaguard-upstream/src/provider.rs) | `ProviderRegistry` — 存储从 YAML 加载的内置提供商定义 |
| [manager.rs](../crates/aidaguard-upstream/src/manager.rs) | `UpstreamManager` — 高级管理器，组合提供商注册表 + 用户上游；`resolve()`、`find_by_endpoint()`、`create_client()` |
| [client.rs](../crates/aidaguard-upstream/src/client.rs) | `UpstreamClient` — 使用协议特定的头、认证、超时配置 reqwest |

**内置提供商：** `openai.yaml`、`anthropic.yaml`、`deepseek.yaml`、`qwen.yaml`、`zhipu.yaml`、`groq.yaml`、`gemini.yaml` — 通过 `include_str!` 在编译时嵌入。

---

### 5. `aidaguard-plugins` — AI 工具适配器

**功能：** 插件系统，用于检测、配置和备份 AI 编程工具的配置（Cursor、Cline、Claude Code 等）。

**关键文件：**

| 文件 | 说明 |
|------|------|
| [registry.rs](../crates/aidaguard-plugins/src/registry.rs) | `PluginRegistry` — 启用/禁用生命周期，持久化到 `plugins.json`；`Plugin` trait 继承 `ToolAdapter` + `PluginManifest` |
| [adapters/](../crates/aidaguard-plugins/src/adapters/) | 31 个工具适配器：25 个声明式适配器（从 `manifests/*.json` 编译时嵌入）+ 6 个复杂适配器（`aider`、`codex`、`hermes_agent`、`gemini`、`codewhisperer`、`jetbrains_ai`） |
| [declarative/](../crates/aidaguard-plugins/src/declarative/) | 声明式适配器引擎：`manifest.rs`（数据结构）、`json_path.rs`（JSON 路径）、`engine.rs`（通用实现）、`loader.rs`（编译时加载） |
| [manifests/](../crates/aidaguard-plugins/manifests/) | 25 个 JSON 清单，定义每种工具的检测、读写和恢复策略 |
| [backup.rs](../crates/aidaguard-plugins/src/backup.rs) | 工具适配器的配置备份/还原 |

**`ToolAdapter` trait 方法：** `id()`、`name()`、`config_path()`、`detect()`、`current_endpoint()`、`current_model()`、`backup()`、`configure()`、`restore()`、`is_configured()`

---

### 6. `aidaguard-storage` — 加密审计日志

**功能：** 基于 SQLite 的加密审计存储，使用 AES-256-GCM。通过 PBKDF2-HMAC-SHA256 (600,000 次迭代) 从用户提供的密码派生加密密钥。

**核心类型：**
- `Storage` — 数据库连接 + 加密器，通过 `Mutex<Connection>` 保证线程安全
- `DetectionRecord` — 完整审计记录（id、timestamp、rule_id、rule_name、strategy、placeholder、原文、上下文、request_path、脱敏后的 body、tool_name）
- `AuditGroup` — 按 (rule_id, strategy) 分组，含计数和最新时间戳
- `AuditStats` — 总计/今日/本周计数 + 规则分布 + 数据库文件大小
- `RuleCount` — 每条规则的命中计数

**操作：** `record()`、`list()`、`list_recent()`、`list_grouped()`、`list_filtered()`、`get_by_id()`、`delete()`、`stats()`

**安全设计：** `original` 和 `context` 字段在写入前加密，读取时解密。随机 salt 存储在 `{db_path}.salt` 文件中，用于密钥派生。

---

### 7. `aidaguard-tauri` — 桌面应用

**功能：** Tauri v2 桌面应用，带托盘图标、系统通知集成、React/TypeScript 前端。

**后端 (Rust)：**

| 文件 | 说明 |
|------|------|
| [main.rs](../crates/aidaguard-tauri/src-tauri/src/main.rs) | 入口：构建引擎、注册 Tauri 命令、初始化 tracing |
| [state.rs](../crates/aidaguard-tauri/src-tauri/src/state.rs) | `AppState` — 共享状态：配置、检测引擎、存储、代理句柄、插件注册表 |
| [events.rs](../crates/aidaguard-tauri/src-tauri/src/events.rs) | 后台事件处理（检测事件路由） |
| [tray.rs](../crates/aidaguard-tauri/src-tauri/src/tray.rs) | 系统托盘图标及菜单（显示/隐藏、启动/停止代理、退出） |
| [lib.rs](../crates/aidaguard-tauri/src-tauri/src/lib.rs) | `resolve_storage_path()`、`resolve_rules_dir()` — 路径解析辅助函数 |

**Tauri 命令：**

| 模块 | 命令 |
|--------|----------|
| `commands/config.rs` | `get_config`、`save_config`、`get_app_version` |
| `commands/proxy.rs` | `start_proxy`、`stop_proxy`、`proxy_status` |
| `commands/rules.rs` | `get_rules`、`reload_rules`、`add_rule`、`update_rule`、`delete_rule`、`test_rule`、`get_rule_categories`、`create_category`、`rename_category`、`delete_category`、`generate_rule` |
| `commands/audit.rs` | `get_audit_records`、`get_audit_grouped`、`get_audit_stats`、`export_csv`、`export_json`、`delete_audit_record` |
| `commands/tools.rs` | `get_tools`、`configure_tool`、`restore_tool`、`restore_all_tools`、`set_plugin_enabled` |
| `commands/upstream.rs` | `get_upstreams`、`add_upstream`、`update_upstream`、`delete_upstream`、`set_default_upstream`、`test_connectivity` |

**前端 (React/TypeScript)：**

```
src/
├── main.tsx                 → 入口，带 ConfigProvider + i18n 的 React 根节点
├── App.tsx                  → Tauri 窗口事件 + 通知权限 + 布局
├── globals.css              → Tailwind CSS 全局样式
├── lib/utils.ts             → shadcn/ui 工具函数
├── types/index.ts           → 镜像 Rust 类型的 TypeScript 接口
├── api/                     → Tauri invoke 封装层
│   ├── config.ts            → getConfig、saveConfig、getAppVersion
│   ├── proxy.ts             → startProxy、stopProxy、proxyStatus
│   ├── rules.ts             → 规则 CRUD 操作
│   ├── audit.ts             → 审计记录查询 + 导出
│   ├── tools.ts             → 工具插件管理
│   ├── upstream.ts          → LLM 上游管理
│   └── events.ts            → 检测事件监听
├── store/                   → Zustand 状态管理（按功能模块划分）
│   └── useThemeStore.ts     → 主题状态
├── i18n/
│   ├── index.ts             → i18next 配置（en/zh、语言检测）
│   ├── en.ts                → 英文翻译
│   └── zh.ts                → 中文翻译
├── pages/
│   ├── Dashboard.tsx        → 代理控制、统计卡片、事件推送
│   ├── Settings.tsx         → 代理、检测策略、NLP、存储、日志、通知、外观
│   ├── Rules.tsx            → 规则管理：分类、CRUD、测试面板、AI生成规则
│   ├── AuditLog.tsx         → 审计表格、过滤器、详情面板、导出
│   ├── ToolsConfig.tsx      → 插件列表：检测、配置、还原 AI 工具
│   └── Upstreams.tsx        → LLM 提供商管理
├── components/
│   ├── StatCard.tsx         → 指标卡片
│   ├── EventFeed.tsx        → 实时检测事件列表
│   ├── AuditTable.tsx       → 分页审计记录表格
│   ├── AuditDetailPanel.tsx → 审计记录详情
│   ├── RuleEditor.tsx       → 规则创建/编辑表单
│   ├── RuleTestPanel.tsx    → 测试文本输入 + 匹配结果展示
│   ├── RuleHitChart.tsx     → 规则命中分布可视化
│   ├── GenerateRuleModal.tsx→ AI 规则生成对话框
│   ├── PresetSwitcher.tsx   → 规则预设切换器
│   ├── OperationGuide.tsx   → 新用户设置引导
│   ├── ThemeSwitcher.tsx    → 浅色/深色/跟随系统主题切换
│   ├── Logo.tsx             → 应用 Logo
│   └── ui/                  → shadcn/ui 组件（button, card, dialog 等）
└── hooks/
    └── useNotification.ts   → 桌面通知权限 + 发送
```

**技术栈：**
- React 18 + TypeScript 6
- shadcn/ui (Radix UI) + Tailwind CSS 3.4
- Zustand 状态管理
- Recharts 图表
- React Hook Form + Zod 表单验证
- date-fns 日期处理
- Sonner 通知

**UI 页面概览：**
1. **Dashboard（仪表盘）** — 代理启停、实时统计（今日/总计/本周）、事件推送
2. **Settings（设置）** — 代理端口/规则目录/最大请求体、地区/行业检测策略、NLP NER 设置、存储数据库、日志、通知、主题
3. **Rules（规则管理）** — 规则分类（创建/重命名/删除）、规则 CRUD、测试面板、AI 生成规则
4. **AuditLog（审计日志）** — 过滤审计记录、详情面板、CSV/JSON 导出
5. **ToolsConfig（工具配置）** — AI 编程工具检测、代理配置、备份/还原
6. **Upstreams（大模型接入）** — LLM 提供商管理：增/改/删、设为默认、测试连通性

---

## 规则目录布局

```
rules/
├── global/                 # 始终作为基线加载
│   ├── credentials.yaml    # API 密钥、JWT、AWS 密钥、私钥
│   ├── finance.yaml        # 信用卡、IBAN、SWIFT、金额、加密货币
│   └── identifiers.yaml    # 邮箱、IP、MAC、URL、电话号码
├── cn/                     # 中国 PIPL 地区
│   ├── general.yaml         # 中国身份证、护照、车牌号
│   ├── finance.yaml         # 中国银行账号
│   ├── medical.yaml         # 医疗实体正则
│   └── personal.yaml        # 个人信息标识符
├── eu/                     # 欧盟 GDPR 地区
│   ├── general.yaml
│   └── finance.yaml
├── gb/                     # 英国 UK DPA 地区
│   └── general.yaml
└── us/                     # 美国 CCPA/HIPAA 地区
    ├── general.yaml         # 美国 SSN、英国 NINO
    ├── finance.yaml
    └── medical.yaml
```

每个 YAML 文件包含一个 `rules` 数组，其中 `RuleDef` 对象包含 `id`、`name`、`pattern`、`enabled`、`strategy`、`mode`、`priority` 和 `compliance` 字段。检测器还支持 `exclude` 排除正则，用于误报过滤。

地区预设由 `Config::rule_presets()` 计算：始终包含 `global`，再加上选定的区域目录以及每个启用的行业对应的 region/industry 子目录。

---

## 检测策略

| 策略 | 行为 | 示例 |
|----------|----------|---------|
| `Placeholder` | 将整个匹配替换为 `[[RULE_a1b2c3d4]]` | `[[EMAIL_a1b2c3d4]]` |
| `Mask` | 部分掩码，保留首尾各约 1/3 可见 | `138****5678` |

| 模式 | 行为 |
|------|----------|
| `Filter` | 在请求体中替换匹配内容，记录审计 |
| `Detect` | 仅记录，不替换 |

---

## 测试

```
tests/                        # 集成测试 crate
├── tests/core_detector.rs   # 检测器 + 规则加载测试

crates/aidaguard-detector/tests/
├── rules_load.rs             # 从目录加载规则
├── recognizer_tests.rs       # 各模式识别器单元测试
├── detector_pipeline.rs      # 完整管线测试
├── detector_validation.rs   # 校验和/验证测试
└── nlp_e2e.rs               # NLP 端到端：引擎构建、模式检测、
                              # NLP 识别器注册、传统 YAML 加载、
                              # BERT NER 推理（需启用 nlp feature）
```

---

## 配置文件

`config.toml` 示例：

```toml
port = 19000
rules_dir = "./rules"
region = "cn"
rule_industries = ["finance", "medical"]
log_level = "info"
max_body_size_mb = 10

[storage]
enabled = true
db_path = "./data/aidaguard.db"
encryption_key = "my-secret-key"

[nlp]
enabled = true
default_language = "en"

[notification]
enabled = true
rate_limit_secs = 60

[[upstreams]]
name = "qianfan-pro"
url = "https://qianfan.baidubce.com/v2"
api_key = "sk-xxx"
default = true
timeout_secs = 300
rate_limit_qps = 0
protocol = "openai"
```

---

## 特性开关

| Crate | Feature | 说明 |
|-------|---------|-------------|
| `aidaguard-detector` | `nlp` | 通过 `candle`/`tokenizers`/`hf-hub` 启用 BERT NER。首次使用需下载约 400 MB 模型。未启用此特性时，NLP 识别器返回空结果。 |

## 关键设计决策

1. **选择 `candle` 而非 `ort`**：纯 Rust 机器学习框架，避免 C++ ONNX Runtime 依赖 — 构建更简洁，无需为 Tauri 打包共享库
2. **单一共享 NLP 模型**：每种语言使用一个 BERT NER 模型为全部 10 个 NlpRecognizer 实例服务。推理结果按文本缓存
3. **双路径检测**：模式识别器（正则 + 校验和 + 上下文词）与传统 YAML 规则共存，由管线合并并去重
4. **加密审计**：原始敏感数据通过 AES-256-GCM 加密存储，密钥通过 PBKDF2 配合每个数据库独有的随机 salt 派生
5. **声明式 LLM 提供商**：提供商定义（端点、认证方式、模型列表）以 YAML 格式存储并在编译时嵌入
6. **插件架构**：AI 工具适配器实现统一的 `ToolAdapter` trait，具备检测/配置/备份/还原生命周期
