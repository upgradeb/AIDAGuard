# Aidaguard 开发总结

## 项目概述

Aidaguard 是一个本地 API 网关代理，部署在 AI 客户端与大模型 API 之间，自动检测请求中的敏感数据（手机号、身份证、银行卡等），替换为占位符后再转发给大模型，并在响应中将占位符还原为原始数据。

**技术栈：** Rust, Tokio, Axum 0.7, Reqwest 0.12, Serde, Regex, Notify (热加载)

**默认上游：** 百度千帆 API (`https://qianfan.baidubce.com/v2/coding`)

---

## 项目结构

```
aidaguard/
├── Cargo.toml                    # workspace 根，集中管理依赖版本
├── config.toml                   # 代理配置文件（目标URL、端口、日志、存储等）
├── rules/                        # YAML 规则文件目录（支持热加载）
│   ├── general.yaml              #   通用规则：手机号、身份证、邮箱、银行卡、API Key
│   ├── finance.yaml              #   金融规则：银行卡（严格）、SWIFT 代码、金额
│   └── medical.yaml              #   医疗规则：病历号、诊断码、患者姓名
├── data/                         # [运行时生成] SQLite 数据库文件
├── docs/                         # 文档
└── crates/
    └── aidaguard-core/
        ├── Cargo.toml
        └── src/
            ├── main.rs           # 入口：加载配置、初始化 tracing、启动代理
            ├── lib.rs            # 模块声明：config, proxy, detector, replacer, storage
            ├── config.rs         # 配置文件加载（TOML + 默认值）
            ├── proxy/
            │   ├── mod.rs        # 模块声明
            │   ├── server.rs     # HTTP 反向代理服务器 + 健康检查端点
            │   ├── stream.rs     # SSE 流式透传 + 占位符还原
            │   └── forwarder.rs  # 通用请求转发器（TLS + 超时 + 自动注入）
            ├── detector/
            │   └── mod.rs        # 敏感数据检测引擎（正则 + 去重 + 热加载）
            ├── replacer/
            │   └── mod.rs        # 占位符替换与还原
            ├── storage/
            │   └── mod.rs        # SQLite + AES-256-GCM 加密审计存储
            └── examples/
                └── mock_sse.rs   # 测试用 Mock SSE 服务器
```

---

## 架构：请求/响应管线

```
┌─────────┐     HTTP      ┌──────────────────┐    HTTP + API Key    ┌──────────────┐
│  AI 客户端 │ ──────────> │  Aidaguard 代理    │ ──────────────────> │  千帆 LLM API  │
│  (Cline)  │ <────────── │  (127.0.0.1:19000)│ <────────────────── │  (qianfan)    │
└─────────┘              └──────────────────┘                      └──────────────┘
                                  │
                    ┌─────────────┴──────────────┐
                    │  config (TOML 配置)          │
                    │  detector (正则规则检测)       │
                    │  replacer (占位符替换/还原)    │
                    │  forwarder (请求转发)         │
                    │  stream (SSE 流式处理)        │
                    │  storage (加密审计存储)        │
                    │  /health (运行状态)           │
                    └────────────────────────────┘
```

### 请求处理流程（server.rs `handle()`）

1. **URL 拼接** — 将客户端请求的 path + query 追加到目标 URL
2. **Header 处理** — 过滤 hop-by-hop 头（`host`, `connection`, `transfer-encoding` 等 12 个），移除客户端 Authorization，注入 `AIDAGUARD_API_KEY`
3. **Body 读取** — 完整读取请求体至内存
4. **敏感数据检测与替换** — 若 Body 为 UTF-8 文本：
   - `detector.detect(text)` → 收集正则命中
   - `replacer.replace(text, &hits)` → 按策略替换为占位符或掩码
   - 生成 `PlaceholderMap`（占位符 → 原始值 映射表）
5. **转发** — 将处理后的请求发送至上游 API
6. **响应类型判断** — 检查 `Content-Type` 是否包含 `text/event-stream`
7. **响应处理**：
   - **流式 (SSE)**：调用 `stream::stream_response_with_restore()` 逐 chunk 处理
   - **非流式**：读取完整响应体，调用 `replacer::restore()` 一次性还原后返回

---

## 核心模块

### 1. 检测引擎 (detector)

**文件：** `crates/aidaguard-core/src/detector/mod.rs` (332 行)

```
YAML 规则 → RuleDef → CompiledRule (编译 regex) → Detector
                                                        │
                                              detect(text) → Vec<Match>
                                                   │
                                         排序 → 去重 → 去重叠
```

**关键设计：**
- 规则按 **优先级降序** 排列，高优先级先处理
- 去重叠算法：优先级高的命中保留，与已选命中范围重叠的后续命中丢弃
- `Match` 使用 **byte offset** (`start`/`end`) 定位，确保 UTF-8 安全

**规则热加载 (`watch_rules`):**
- 使用 `notify` crate 监听规则目录的文件变更
- 200ms 防抖合并连续事件
- 原子替换整个规则集（`detector.write().await.load_from_dir()`）

**替换策略：**

| 策略 | 效果 | 示例 |
|------|------|------|
| `placeholder` | 整体替换 | `13812345678` → `[[PHONE_CN@a1b2c3d4]]` |
| `mask` | 部分掩码 | `test@example.com` → `tes***com` |

### 2. 占位符引擎 (replacer)

**文件：** `crates/aidaguard-core/src/replacer/mod.rs` (185 行)

**占位符格式：** `[[RULE_ID@UUID8]]`，例如 `[[PHONE_CN@a1b2c3d4]]`

- `RULE_ID` = 规则 ID 的大写形式
- `UUID8` = UUID v4 的前 8 位十六进制字符

**核心函数：**

`replace(text, &[Match]) → (String, PlaceholderMap)`
- 匹配项**从后往前排序**，逐个 `replace_range()`，避免位置偏移
- 返回替换后的文本和占位符映射表

`restore(text, &PlaceholderMap) → String`
- 占位符按**长度降序**排列后逐一 `String::replace()`
- 防止短占位符匹配到长占位符的前缀

**`PlaceholderMap` 公开接口：**
```rust
pub fn insert(&mut self, original: &str, rule_id: &str) -> String
pub fn get(&self, placeholder: &str) -> Option<&str>
pub fn len(&self) -> usize
pub fn placeholders(&self) -> impl Iterator<Item = &String>
```

### 3. 流式还原 (stream) — 最复杂的模块

**文件：** `crates/aidaguard-core/src/proxy/stream.rs` (281 行)

#### 问题演进

开发过程中尝试了三种方案，前两种在测试中失败：

| 迭代 | 方案 | 失败原因 |
|------|------|---------|
| V1 | 固定 30 字节尾缓冲 | 从占位符 `[[PHONE_CN@4041d8aa]]` 中间切断 |
| V2 | `[[` 边界感知 + 500 字节搜索窗口 | ① reasoning_content 中的 `[[` 阻塞整个 buffer；② SSE JSON wrapper 使 `[[`...`]]` 距离超 500 字节 |
| **V3** | **解析 SSE JSON + 纯文本累积 + 已知前缀匹配** | ✅ 成功 |

#### 最终方案 (V3)

```
SSE chunk (raw bytes)
  │
  ├─ 按 \n\n 分割为多条 SSE 消息
  │
  └─ 每条消息：
       │
       ├─ [DONE] → 原样转发
       │
       └─ data: {json}
            │
            ├─ 解析 JSON (serde_json::Value)
            │
            ├─ 提取文本字段（优先级：content > tool_calls[0].function.arguments）
            │
            ├─ 追加至共享文本缓冲区 (Arc<Mutex<String>>)
            │
            ├─ find_safe_len(): 检查尾部是否为已知占位符前缀
            │   │
            │   ├─ 是 → 保留前缀部分，只还原和转发前面的安全文本
            │   └─ 否 → 全部安全，还原后转发
            │
            ├─ replacer::restore() 还原安全文本中的占位符
            │
            ├─ 修改 JSON 对应字段
            │
            └─ 重新序列化 → 转发
```

**关键设计决策：**

1. **解析 JSON 而非操作原始字节** — SSE JSON wrapper 会在 `[[PHONE_CN@` 和 `90190311]]` 之间插入约 55 字节的 JSON 结构（`"},"flag":0}]}\n\ndata: {"id":"...","choices":[{"delta":{"content":"`），使占位符在原始字节流中永远不连续。解析 JSON 后在纯文本上操作解决此问题。

2. **已知占位符前缀匹配，而非扫描通用 `[[`/`]]`** — 模型的 reasoning_content 中可能包含 `[[` 字符（如 "用户的消息是 [[PHONE_CN@...]]"），用通用扫描会误判。只对已知占位符做前缀匹配，`[[` 出现在文本中间而非尾部时不阻塞转发。

3. **同时处理 content 和 tool_calls 字段** — GLM-5 模型的最终输出在 `tool_calls[0].function.arguments` 中，而非 `content`。`extract_text_field()` 按优先级提取两个来源的文本。

4. **安全分割 (find_safe_len)** — 纯文本尾部若匹配已知占位符的前缀（如 `[[PHONE_CN@a3ed18`），保留该前缀等待后续 delta 补全；否则整块都是安全的。前缀匹配从长度 1 到 len-1（不含完整占位符本身，因为完整占位符不需要保留）。

### 4. 代理服务器 (server)

**文件：** `crates/aidaguard-core/src/proxy/server.rs` (343 行)

**启动流程：** `start(Config)` — 加载规则 → 初始化 Storage（可选）→ 创建 Forwarder → 绑定端口 → 构造 Router

**路由表：**
| 路由 | 方法 | Handler | 说明 |
|------|------|---------|------|
| `/health` | GET | `health_check` | 健康检查，返回 JSON（状态、版本、运行时长、规则数、存储状态） |
| `/` | ANY | `proxy_handler` | 代理所有请求 |
| `/*path` | ANY | `proxy_handler` | 代理带路径的请求 |

**健康检查响应示例：**
```json
{"status":"ok","version":"0.1.0","uptime_seconds":3600,"rules_count":9,"storage_enabled":true}
```

**`forward_headers()` 跳过列表（12 个 hop-by-hop 头）：**
`host`, `authorization`, `connection`, `content-encoding`, `content-length`, `keep-alive`, `proxy-authenticate`, `proxy-authorization`, `te`, `trailers`, `transfer-encoding`, `upgrade`

其中 `content-length` 和 `content-encoding` 的移除尤为关键——占位符替换后 body 长度会改变，若保留原始 Content-Length 会导致截断。

### 5. 配置模块 (config)

**文件：** `crates/aidaguard-core/src/config.rs` (92 行)

**配置来源：** 仅从 `config.toml` 文件加载，不依赖环境变量。

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `api_key` | String | `""` | **必填**，上游 LLM API 认证 Key |
| `port` | u16 | `19000` | 本地监听端口 |
| `target_url` | String | `https://qianfan.baidubce.com/v2/coding` | 上游 API 地址 |
| `rules_dir` | String | `"./rules"` | YAML 规则文件目录 |
| `log_level` | String | `"info"` | 日志级别 |
| `[storage]` | Table | — | 存储子表（可选） |
| `storage.enabled` | bool | `false` | 启用审计记录 |
| `storage.db_path` | String | `"./data/aidaguard.db"` | 数据库路径 |
| `storage.encryption_key` | Option | 内置默认值 | AES 加密密钥 |

### 6. 请求转发器 (forwarder)

**文件：** `crates/aidaguard-core/src/proxy/forwarder.rs` (45 行)

```rust
Forwarder::new() -> Result<Self>
Forwarder::forward(method, url, headers, body, api_key) -> Result<Response>
Forwarder::set_timeout(duration)
```

- 封装 reqwest HTTP 客户端（rustls-tls）
- 自动注入 `Bearer <api_key>` 到 Authorization 头
- 默认 300 秒超时
- 可扩展重试、故障转移、多后端负载均衡

### 7. 加密审计存储 (storage)

**文件：** `crates/aidaguard-core/src/storage/mod.rs` (271 行)

**依赖：** SQLite (rusqlite, bundled) + AES-256-GCM (aes-gcm) + SHA-256 (sha2)

**数据库表 `detections`：**

| 字段 | 类型 | 加密 | 说明 |
|------|------|------|------|
| `id` | TEXT PK | 否 | UUID v4 |
| `timestamp_ms` | INTEGER | 否 | Unix 毫秒时间戳 |
| `rule_id` | TEXT | 否 | 触发规则 ID |
| `strategy` | TEXT | 否 | 替换策略 |
| `placeholder` | TEXT | 否 | 生成的占位符 |
| `original_encrypted` | BLOB | **AES-256-GCM** | 原始敏感值密文 |
| `context_encrypted` | BLOB | **AES-256-GCM** | 匹配位置前后各 80 字节上下文密文 |
| `request_path` | TEXT | 否 | 请求路径 |
| `sanitized_body` | TEXT | 否 | 替换后的完整请求体 |
| `response_status` | INTEGER | 否 | 上游响应 HTTP 状态码 |

**公开 API：**
```rust
Storage::open(db_path, encryption_key) -> Result<Self>
Storage::record(rule_id, strategy, placeholder, original, context, request_path, sanitized_body, response_status) -> Result<()>
Storage::list(limit, offset) -> Result<Vec<DetectionRecord>>  // 解密 original + context
Storage::count() -> Result<usize>
```

**密钥派生：** SHA-256(encryption_key) → 32 字节 AES-256 密钥。`encryption_key` 未设置时使用内置默认密钥。

**加密格式：** [nonce (12 bytes) | AES-GCM ciphertext]，nonce 每次记录随机生成（OsRng）。

---

## 规则系统

### 规则文件格式 (YAML)

```yaml
version: "1.0"
name: "规则集名称"
description: "规则集描述"

rules:
  - id: phone_cn          # 规则唯一标识
    name: "手机号"         # 人类可读名称
    pattern: '1[3-9]\d{9}' # 正则表达式
    enabled: true          # 是否启用 (默认 true)
    strategy: placeholder  # 替换策略 (placeholder | mask, 默认 placeholder)
    priority: 100          # 优先级，越大越优先 (默认 100)
```

### 当前规则汇总

**通用规则 (general.yaml):**

| ID | 名称 | 模式 | 策略 | 优先级 | 状态 |
|---|---|---|---|---|---|
| `phone_cn` | 手机号 | `1[3-9]\d{9}` | placeholder | 100 | 启用 |
| `id_card_cn` | 身份证 | `\d{17}[\dXx]` | placeholder | 100 | 启用 |
| `email` | 邮箱 | `[\w.+-]+@[\w-]+\.\w+` | mask | 90 | 启用 |
| `bank_card` | 银行卡 | `\b\d{16,19}\b` | placeholder | 100 | 启用 |
| `ipv4` | IPv4 | `\b(?:\d{1,3}\.){3}\d{1,3}\b` | placeholder | 80 | 禁用 |
| `api_key` | API Key | `(?:sk\|pk)-[A-Za-z0-9]{32,}` | placeholder | 100 | 启用 |

**金融规则 (finance.yaml):**

| ID | 名称 | 模式 | 策略 | 优先级 | 状态 |
|---|---|---|---|---|---|
| `bank_card_strict` | 银行卡（严格）| `\b(?:62\|60\|64)\d{14,17}\b` | placeholder | 100 | 启用 |
| `swift_code` | SWIFT 代码 | `\b[A-Z]{6}[A-Z0-9]{2}([A-Z0-9]{3})?\b` | placeholder | 90 | 启用 |
| `amount_cny` | 金额（人民币）| `(?:¥\|CNY\s?)\d+(?:\.\d{1,2})?` | mask | 70 | 禁用 |

**医疗规则 (medical.yaml):**

| ID | 名称 | 模式 | 策略 | 优先级 | 状态 |
|---|---|---|---|---|---|
| `medical_record_no` | 病历号 | `MR\d{6,10}` | placeholder | 100 | 启用 |
| `diagnosis_code` | ICD 诊断码 | `\b[A-Z]\d{2}(?:\.\d{1,4})?\b` | placeholder | 80 | 禁用 |
| `patient_name_hint` | 患者姓名提示 | `(?:患者\|病人\|就诊人)[：:]\s*[一-龥]{2,4}` | placeholder | 90 | 启用 |

---

## 配置文件

所有配置集中在 `config.toml` 中，不依赖环境变量。详见上方配置模块文档。

### 最小可运行配置

```toml
api_key = "your-api-key"
[storage]
enabled = true
```

---

## 启动与测试

### 启动代理

```bash
# 在 config.toml 中配置 api_key 后直接启动
cargo run --bin aidaguard
```

### 启动 Mock SSE 测试服务器

```bash
cargo run --bin mock-sse
# 监听 127.0.0.1:19999，模拟 SSE 流式响应
```

### 运行测试

```bash
cargo test                    # 全部测试 (24 tests)
cargo test stream             # 流式模块测试 (5 tests)
cargo test detector           # 检测模块测试 (7 tests)
cargo test replacer           # 替换模块测试 (6 tests)
cargo test storage            # 存储模块测试 (5 tests)
```

### 代理端口

- 代理监听：`127.0.0.1:19000`
- Mock SSE：`127.0.0.1:19999`

---

## 待实现模块

| 模块 | 文件 | 状态 |
|------|------|------|
| reasoning_content 流式还原 | `proxy/stream.rs` | 探索中 |
| 请求体大小限制 | `proxy/server.rs` | 探索中 |
| 规则热重载 API | `proxy/server.rs` | 探索中 |
| Tauri 桌面客户端 | `crates/aidaguard-tauri/` | 未创建 |

---

## 已知限制

1. **仅处理 UTF-8 文本请求体** — 二进制 body 跳过检测与还原
2. **非流式响应需完整加载** — 大响应可能占用较多内存
3. **流式还原仅处理 content 和 tool_calls[0].function.arguments** — reasoning_content 字段不做还原
4. **无请求体大小限制** — 超大 body 可能导致内存不足
