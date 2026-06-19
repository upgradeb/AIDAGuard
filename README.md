# Aidaguard

**AI 时代的隐私守护者 — 本地 LLM API 网关，守护每一个 Token**

Aidaguard 是一个开源的本地 LLM API 网关代理，在敏感数据到达任何大模型 API 之前自动检测、替换，并在响应到达客户端后无缝还原。零配置，无需安装证书，所有数据本地处理。

---

## 工作原理

```
你的 AI 客户端
      │
      │  HTTP  (localhost:19000)
      ▼
 Aidaguard 代理  ←── 检测并替换敏感数据
      │
      │  HTTPS (加密)
      ▼
 OpenAI / Anthropic / DeepSeek / 任何兼容 API
      │
      ▼
 Aidaguard 代理  ←── 还原响应中的占位符
      │
      ▼
你的 AI 客户端
```

## 功能特性

- **本地 API 网关** — 无需证书，只需将 API Base URL 指向 `http://localhost:19000`
- **智能检测** — 双引擎检测：YAML 规则 + 30+ 模式识别器（含 Luhn 校验、身份证校验、IBAN 验证）
- **无缝替换** — 发送前替换敏感数据为占位符 `[[PHONE_CN@uuid]]`，响应时自动还原
- **流式支持** — 完整支持 SSE 流式响应，带滑动缓冲区还原
- **多区域规则** — 按 PIPL/GDPR/CCPA/HIPAA 等合规框架组织规则，支持 cn/us/eu/gb 等区域预设
- **文件规则** — YAML 规则文件，支持热加载，可社区扩展
- **加密存储** — 映射表本地存储，采用 AES-256-GCM 加密（PBKDF2 60万次迭代）
- **工具适配** — 自动配置 Cursor、Claude Code、Aider 等 31+ AI 工具的代理设置
- **可选 NLP** — BERT NER 命名实体识别（`--features nlp`），支持非结构化文本检测
- **跨平台** — 基于 Tauri 2.x，支持 macOS、Windows、Linux

## 支持的敏感数据类型

### 中国 (PIPL)
- 居民身份证（18 位 mod-11 校验）、护照、军官证、港澳通行证
- 手机号、车牌号、银行卡号
- 统一社会信用代码

### 美国 (CCPA/HIPAA)
- 社会安全号 (SSN)、ITIN、EIN
- 各州驾照
- HIPAA 医疗信息

### 欧盟 (GDPR)
- IBAN、SWIFT/BIC
- 各国增值税号 (VAT)
- 各国身份证

### 英国 (DPA)
- NINO、NHS 号码

### 全球通用
- 信用卡（Visa、Mastercard、Amex 等），带 Luhn 校验
- 邮箱地址、IP 地址（IPv4/IPv6）、MAC 地址
- API Key（OpenAI、AWS、GitHub、Stripe 等）
- JWT Token、私钥

## 支持的 LLM 提供商

| 提供商 | 端点 | 认证方式 |
|--------|------|----------|
| OpenAI | `api.openai.com/v1` | Bearer Token |
| Anthropic | `api.anthropic.com/v1` | x-api-key |
| DeepSeek | `api.deepseek.com/v1` | Bearer Token |
| Qwen (通义) | `dashscope.aliyuncs.com` | Bearer Token |
| Zhipu (智谱) | `open.bigmodel.cn` | Bearer Token |
| Gemini | `generativelanguage.googleapis.com` | x-goog-api-key |
| Groq | `api.groq.com` | Bearer Token |

支持任何兼容 OpenAI API 格式的提供商。

## 支持的 AI 工具

Aidaguard 可自动检测并配置以下工具的代理设置：

| 类别 | 工具 |
|------|------|
| 命令行 | Claude Code、Aider、Codex CLI、Gemini CLI、OpenCode |
| IDE | Cursor、Zed、Windsurf、JetBrains AI |
| VS Code 扩展 | Cline、Continue.dev、Codeium、Cody、Tabnine |

## 快速开始

### 前置条件

- [Rust](https://rustup.rs) (1.75+)
- [Node.js](https://nodejs.org) (18+)
- [Tauri CLI](https://tauri.app) (`cargo install tauri-cli`)

### 构建并运行

```bash
git clone https://github.com/user/aidaguard.git
cd aidaguard

# 构建并运行桌面应用
cd crates/aidaguard-tauri
cargo tauri dev
```

### 配置代理

1. 在桌面应用中添加 LLM 提供商和 API Key
2. 将 AI 客户端的 Base URL 设置为 `http://localhost:19000`
3. 或者使用「工具配置」页面自动配置 AI 工具

### 配置文件

复制 `config.example.toml` 为 `config.toml` 并按需修改：

```toml
port = 19000
region = "cn"                    # 规则区域：cn, us, eu, gb

[storage]
enabled = true
db_path = "./data/aidaguard.db"

# 大模型接入也可在桌面应用的「大模型接入」页面管理
# [[upstreams]]
# name = "deepseek"
# url = "https://api.deepseek.com/v1"
# api_key = "sk-..."
# default = true
```

## 项目结构

```
aidaguard/
├── crates/
│   ├── aidaguard-core/        # 基础层：核心类型、配置、YAML 规则检测器、替换器
│   ├── aidaguard-detector/    # 检测引擎：模式识别器 + NLP NER + 分析管线
│   ├── aidaguard-proxy/       # HTTP 反向代理 (Axum 0.7)
│   ├── aidaguard-upstream/    # LLM 提供商管理（7 个内置提供商）
│   ├── aidaguard-plugins/     # AI 工具适配器（25 声明式 + 6 复杂适配器）
│   ├── aidaguard-storage/     # 加密审计存储 (SQLite + AES-256-GCM)
│   └── aidaguard-tauri/       # 桌面客户端 (Tauri 2.x + React 18)
├── rules/                     # 内置规则包（18 个 YAML 文件，按地区组织）
│   ├── core.yaml              # 全局基线规则
│   ├── global/                # 通用规则（凭证、金融、标识符）
│   ├── cn/                    # 中国 (PIPL)
│   ├── us/                    # 美国 (CCPA/HIPAA)
│   ├── eu/                    # 欧盟 (GDPR)
│   └── gb/                    # 英国 (DPA)
├── docs/                      # 文档
└── tests/                     # 集成测试
```

## 检测引擎

### 双引擎架构

Aidaguard 采用双引擎检测架构：

1. **YAML 规则引擎** — 从 `rules/` 目录加载，支持热重载
2. **模式识别器** — Rust 代码实现，带校验器（Luhn、mod-11、IBAN）和上下文词评分

两者由统一的 `AnalyzerEngine` 管线合并，经过置信度评分和重叠仲裁后输出最终结果。

### 规则格式

```yaml
rules:
  - id: credit_card
    name: "信用卡号"
    pattern: '(?-u:\b)(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13})(?-u:\b)'
    validator: luhn                      # Luhn 校验
    context_words: [credit, card, visa]  # 上下文词（提升置信度）
    base_confidence: 0.4                 # 基础置信度
    enabled: true
    strategy: placeholder                # placeholder | mask
    mode: filter                         # filter | detect
    priority: 100
    compliance: [PCI_DSS]                # 合规标签
```

### 替换策略

| 策略 | 效果 | 示例 |
|------|------|------|
| `placeholder` | 整体替换为占位符 | `13812345678` → `[[PHONE_CN@a1b2c3d4]]` |
| `mask` | 部分掩码 | `test@example.com` → `tes***com` |

## 开发

```bash
# 构建整个工作空间
cargo build

# 构建并启用 NLP 特性
cargo build --features nlp

# 运行所有测试
cargo test

# 运行指定 crate 的测试
cargo test -p aidaguard-detector

# 代码格式化
cargo fmt

# Lint
cargo clippy --all
```

详见 [开发指南](docs/DEVELOPMENT.md)。

## 文档

| 文档 | 说明 |
|------|------|
| [架构设计](docs/ARCHITECTURE.md) | 系统架构和模块说明 |
| [开发指南](docs/DEVELOPMENT.md) | 开发环境配置和工作流程 |
| [UI 设计](docs/UI_DESIGN.md) | 桌面客户端 UI 设计文档 |
| [开发记录](docs/WORKLOG.md) | 各阶段开发工作总结 |
| [LLM 提供商参考](docs/reference/llm-providers.md) | 国内外主流大模型提供商 |
| [工具适配器](docs/reference/tool-adapters.md) | AI 工具适配器分析 |
| [适配器架构](docs/reference/adapter-architecture.md) | 声明式适配器引擎设计 |

## 技术栈

**后端：** Rust · Tokio · Axum 0.7 · Reqwest 0.12 · Candle (可选 NLP/NER)

**前端：** Tauri 2.x · React 18 · TypeScript · shadcn/ui · Tailwind CSS · Zustand · Recharts

## 许可证

[MIT](LICENSE)
