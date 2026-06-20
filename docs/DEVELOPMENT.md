# AIDAGuard 开发指南

**版本：** 0.5.0

## 项目概述

AIDAGuard 是一个本地 LLM API 代理，部署在 AI 客户端与大模型 API 之间，自动检测请求中的敏感数据（手机号、身份证、银行卡等），替换为占位符后再转发给大模型，并在响应中将占位符还原为原始数据。

**技术栈：**
- 后端：Rust, Tokio, Axum 0.7, Reqwest 0.12, Serde, Regex, Notify
- 前端：Tauri 2.x, React 18, TypeScript, shadcn/ui, Tailwind CSS, Zustand

---

## 项目结构

```
aidaguard/
├── Cargo.toml                    # workspace 根，集中管理依赖版本
├── config.toml                   # 代理配置文件
├── rules/                        # YAML 规则文件目录（支持热加载）
│   ├── global/                   # 全局规则
│   ├── cn/                       # 中国地区规则
│   ├── eu/                       # 欧盟地区规则
│   └── ...
├── data/                         # [运行时生成] SQLite 数据库文件
├── docs/                         # 文档
└── crates/
    ├── aidaguard-core/           # 基础层：类型、配置、接口
    ├── aidaguard-detector/       # 检测引擎：正则 + NLP NER
    ├── aidaguard-proxy/          # HTTP 代理服务器
    ├── aidaguard-upstream/       # LLM 提供商管理
    ├── aidaguard-plugins/        # AI 工具适配器
    ├── aidaguard-storage/        # 加密审计存储
    └── aidaguard-tauri/          # Tauri 桌面应用
        ├── src-tauri/            # Rust 后端
        └── src/                  # React 前端
```

---

## 架构：请求/响应管线

```
┌─────────┐     HTTP      ┌──────────────────┐    HTTP + API Key    ┌──────────────┐
│  AI 客户端 │ ──────────> │  AIDAGuard 代理    │ ──────────────────> │  LLM API      │
│ (Cursor)  │ <────────── │  (127.0.0.1:19000)│ <────────────────── │  (DeepSeek)   │
└─────────┘              └──────────────────┘                      └──────────────┘
                                  │
                    ┌─────────────┴──────────────┐
                    │  detector (检测引擎)         │
                    │  replacer (占位符替换/还原)   │
                    │  proxy (HTTP 代理)          │
                    │  storage (加密审计存储)      │
                    └────────────────────────────┘
```

### 请求处理流程

1. **URL 拼接** — 将客户端请求的 path + query 追加到目标 URL
2. **Header 处理** — 过滤 hop-by-hop 头，移除客户端 Authorization，注入上游 API Key
3. **Body 读取** — 完整读取请求体至内存
4. **敏感数据检测与替换** — 若 Body 为 UTF-8 文本：
   - `detector.detect(text)` → 收集正则命中
   - `replacer.replace(text, &hits)` → 按策略替换为占位符或掩码
5. **转发** — 将处理后的请求发送至上游 API
6. **响应处理**：
   - **流式 (SSE)**：逐 chunk 处理，还原占位符
   - **非流式**：读取完整响应体，一次性还原后返回

---

## 核心模块

### 1. 检测引擎 (aidaguard-detector)

多策略检测引擎：正则模式识别 + 校验和验证 + 可选的 BERT NER。

**特性开关：**
- `nlp` — 启用 BERT NER（首次使用需下载约 400 MB 模型）

**检测管线流程：**
1. 传统 YAML 正则规则
2. 模式识别器（正则 + 校验和 + 上下文词）
3. NLP NER 识别器（BERT 推理）
4. 重叠项仲裁
5. 最低置信度过滤

### 2. 占位符引擎 (aidaguard-core/replacer)

**占位符格式：** `[[RULE_ID@UUID8]]`

- `RULE_ID` = 规则 ID 的大写形式
- `UUID8` = UUID v4 的前 8 位十六进制字符

**核心函数：**
- `replace(text, &[Match]) → (String, PlaceholderMap)`
- `restore(text, &PlaceholderMap) → String`

### 3. 代理服务器 (aidaguard-proxy)

基于 Axum 的反向代理。

**路由表：**

| 路由 | 方法 | 说明 |
|------|------|------|
| `/health` | GET | 健康检查 |
| `/` | ANY | 代理所有请求 |
| `/*path` | ANY | 代理带路径的请求 |

### 4. LLM 提供商管理 (aidaguard-upstream)

声明式 LLM 提供商定义，支持 OpenAI、Anthropic、DeepSeek、Qwen、Zhipu、Groq、Gemini 等。

**内置提供商：** `openai.yaml`, `anthropic.yaml`, `deepseek.yaml`, `qwen.yaml`, `zhipu.yaml`, `groq.yaml`, `gemini.yaml`

### 5. AI 工具适配器 (aidaguard-plugins)

插件系统，用于检测和配置 AI 编程工具（Cursor、Claude Code、Aider 等）。

**支持的工具：** Claude Code, Aider, Codex CLI, Gemini CLI, JetBrains AI 等

### 6. 加密审计存储 (aidaguard-storage)

基于 SQLite 的加密审计存储，使用 AES-256-GCM。通过 PBKDF2-HMAC-SHA256 (600,000 次迭代) 从用户密码派生加密密钥。

---

## 规则系统

### 规则文件格式 (YAML)

```yaml
version: "1.0"
name: "规则集名称"

rules:
  - id: phone_cn          # 规则唯一标识
    name: "手机号"         # 人类可读名称
    pattern: '1[3-9]\d{9}' # 正则表达式
    enabled: true          # 是否启用
    strategy: placeholder  # 替换策略 (placeholder | mask)
    priority: 100          # 优先级，越大越优先
```

### 替换策略

| 策略 | 效果 | 示例 |
|------|------|------|
| `placeholder` | 整体替换 | `13812345678` → `[[PHONE_CN@a1b2c3d4]]` |
| `mask` | 部分掩码 | `test@example.com` → `tes***com` |

### 检测模式

| 模式 | 行为 |
|------|------|
| `filter` | 在请求体中替换匹配内容，记录审计 |
| `detect` | 仅记录，不替换 |

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

[[upstreams]]
name = "DeepSeek"
url = "https://api.deepseek.com/v1"
api_key = "sk-xxx"
default = true
timeout_secs = 300
```

---

## 启动与测试

### 启动桌面应用

```bash
cd crates/aidaguard-tauri
cargo tauri dev
```

### 运行测试

```bash
cargo test                    # 全部测试
cargo test --features nlp     # 包含 NLP 测试
```

### 构建发布版

```bash
cd crates/aidaguard-tauri
cargo tauri build
```

---

## 开发路线图

### v0.4.0 ✅ 已完成
- UI 升级到 shadcn/ui + Tailwind CSS

### v0.5.0 进行中
- 检测引擎并发化
- 代理层流式优化
- 存储层 WAL + 批量写入

### v0.6.0 计划中
- 规则版本管理
- 错误处理增强
- NLP 分块推理

---

## 已知限制

1. **仅处理 UTF-8 文本请求体** — 二进制 body 跳过检测与还原
2. **非流式响应需完整加载** — 大响应可能占用较多内存
3. **流式还原仅处理 content 和 tool_calls** — reasoning_content 字段不做还原
