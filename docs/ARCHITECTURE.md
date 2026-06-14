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

---

## 架构优化方案

### 现有架构评估

#### 架构优势

| 方面 | 优点 |
|------|------|
| **模块化** | 7 个独立 crate，职责边界清晰，依赖单向 |
| **安全性** | 审计数据 AES-256-GCM 加密，PBKDF2 60万次迭代 |
| **扩展性** | Recognizer trait + Plugin trait 支持灵活扩展 |
| **本地优先** | 无云依赖，敏感数据不离开设备 |
| **技术选型** | Candle 纯 Rust ML，避免 FFI 复杂性 |

#### 架构评分

| 维度 | 评分 | 说明 |
|------|------|------|
| 可维护性 | ⭐⭐⭐⭐ | 模块边界清晰，但存在循环依赖隐患 |
| 性能潜力 | ⭐⭐⭐ | 单线程检测，高并发场景受限 |
| 可测试性 | ⭐⭐⭐⭐ | trait 抽象良好，但集成测试覆盖不足 |
| 扩展性 | ⭐⭐⭐⭐ | 插件系统完善，但规则热加载机制可优化 |
| 安全性 | ⭐⭐⭐⭐⭐ | 加密审计、无云依赖、本地处理 |

---

### 优化 1：【高优先级】检测引擎并发化 📋

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

### 优化 2：【高优先级】代理层流式优化 📋

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

### 优化 3：【中优先级】规则热加载机制增强 📋

**问题：** 当前规则热加载依赖 `notify` crate 的文件监听，但存在局限：

```rust
// 当前 AnalyzerEngine 层面的 reload 逻辑不够健壮
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

### 优化 4：【中优先级】存储层性能优化 📋

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

### 优化 5：【中优先级】NLP 推理优化 📋

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

### 优化 6：【低优先级】依赖关系重构 📋

**问题：** 当前 `aidaguard-core` 重新导出 `aidaguard-storage`，导致 `aidaguard-core` → `aidaguard-storage` 的反向依赖，违背了 core 作为基础层的定位。

（详见 [反向依赖问题](#反向依赖问题) 章节）

**优化方案：**

```rust
// 将 storage 接口抽象为 trait

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

### 优化 7：【已完成】UI 界面升级 ✅

**已完成：** 使用 shadcn/ui + Tailwind CSS 重构前端 UI。

**变更内容：**
- 从 Ant Design 迁移到 shadcn/ui (Radix UI)
- 使用 Tailwind CSS 替代自定义 CSS
- 添加全局主题支持（浅色/深色/跟随系统）
- 统一组件样式和交互体验

**技术栈：**
| 组件 | 实现 |
|------|------|
| UI 组件库 | shadcn/ui (Radix UI + Tailwind) |
| 样式方案 | Tailwind CSS 3.4 |
| 表单 | React Hook Form + Zod |
| 图表 | Recharts |
| 通知 | Sonner |

---

### 优化 8：【低优先级】错误处理增强 📋

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

### 架构演进路线图

```
Phase 1 (v0.4.0) - ✅ 已完成
├── 优化 7：UI 升级到 shadcn/ui + Tailwind CSS ✅

Phase 2 (v0.5.0) - 性能优化
├── 优化 1：检测引擎并发化 📋
├── 优化 2：代理层流式优化 📋
└── 优化 4：存储层 WAL + 批量写入 📋

Phase 3 (v0.6.0) - 可靠性增强
├── 优化 3：规则版本管理 📋
├── 优化 8：错误处理增强 📋
└── 优化 5：NLP 分块推理 📋

Phase 4 (v0.7.0) - 架构重构
├── 优化 6：依赖关系重构 📋
├── AuditStorage trait 抽象
└── 插件系统增强（动态加载）
```

---

### 性能基准测试建议

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

### 风险评估

| 优化项 | 风险等级 | 主要风险 | 缓解措施 |
|--------|----------|----------|----------|
| 优化 1 并发检测 | 中 | 线程安全问题 | 充分测试 NLP 模型线程安全 |
| 优化 2 流式优化 | 中 | 占位符边界处理 | 完善单元测试覆盖边界 case |
| 优化 3 规则版本 | 低 | 内存占用增加 | 限制历史版本数量 |
| 优化 4 存储优化 | 中 | 数据丢失风险 | WAL + 定期 checkpoint |
| 优化 5 NLP 优化 | 低 | 精度下降 | 分块重叠策略 |
| 优化 6 依赖重构 | 低 | API 变更 | 保留兼容层 |
| 优化 7 UI 美化 | 低 | 动画性能 | 低端设备降级 |
