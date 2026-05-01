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
├── rules/                        # YAML 规则文件目录（支持热加载）
│   ├── general.yaml              #   通用规则：手机号、身份证、邮箱、银行卡、API Key
│   ├── finance.yaml              #   金融规则：银行卡（严格）、SWIFT 代码、金额
│   └── medical.yaml              #   医疗规则：病历号、诊断码、患者姓名
├── docs/                         # 文档
└── crates/
    └── aidaguard-core/
        ├── Cargo.toml
        └── src/
            ├── main.rs           # 入口：初始化 tracing，启动代理
            ├── lib.rs            # 模块声明：proxy, detector, replacer, storage
            ├── proxy/
            │   ├── mod.rs        # 模块声明
            │   ├── server.rs     # HTTP 反向代理服务器核心
            │   ├── stream.rs     # SSE 流式透传 + 占位符还原
            │   └── forwarder.rs  # [待实现] 通用请求转发器
            ├── detector/
            │   └── mod.rs        # 敏感数据检测引擎（正则 + 去重 + 热加载）
            ├── replacer/
            │   └── mod.rs        # 占位符替换与还原
            ├── storage/
            │   └── mod.rs        # [待实现] SQLite + AES-256-GCM 加密存储
            └── bin/
                └── mock_sse.rs   # 测试用 Mock SSE 服务器
```

---

## 架构：请求/响应管线

```
┌─────────┐     HTTP      ┌──────────────────┐    HTTP + API Key    ┌──────────────┐
│  AI 客户端 │ ──────────> │  Aidaguard 代理    │ ──────────────────> │  千帆 LLM API  │
│ (Roo Code)│ <────────── │  (127.0.0.1:19000)│ <────────────────── │  (qianfan)    │
└─────────┘              └──────────────────┘                      └──────────────┘
                                  │
                    ┌─────────────┴─────────────┐
                    │  detector (正则规则检测)      │
                    │  replacer (占位符替换/还原)   │
                    │  stream (SSE 流式处理)       │
                    └───────────────────────────┘
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

**文件：** `crates/aidaguard-core/src/proxy/server.rs` (283 行)

**`forward_headers()` 跳过列表（12 个 hop-by-hop 头）：**
`host`, `authorization`, `connection`, `content-encoding`, `content-length`, `keep-alive`, `proxy-authenticate`, `proxy-authorization`, `te`, `trailers`, `transfer-encoding`, `upgrade`

其中 `content-length` 和 `content-encoding` 的移除尤为关键——占位符替换后 body 长度会改变，若保留原始 Content-Length 会导致截断。

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
| `swift_code` | SWIFT 代码 | `\b[A-Z]{4}[A-Z]{2}(?=[A-Z0-9]*\d)[A-Z0-9]{2}(?:[A-Z0-9]{3})?\b` | placeholder | 90 | 启用 |
| `amount_cny` | 金额（人民币）| `(?:¥\|CNY\s?)\d+(?:\.\d{1,2})?` | mask | 70 | 禁用 |

**医疗规则 (medical.yaml):**

| ID | 名称 | 模式 | 策略 | 优先级 | 状态 |
|---|---|---|---|---|---|
| `medical_record_no` | 病历号 | `MR\d{6,10}` | placeholder | 100 | 启用 |
| `diagnosis_code` | ICD 诊断码 | `\b[A-Z]\d{2}(?:\.\d{1,4})?\b` | placeholder | 80 | 禁用 |
| `patient_name_hint` | 患者姓名提示 | `(?:患者\|病人\|就诊人)[：:]\s*[一-龥]{2,4}` | placeholder | 90 | 启用 |

---

## 环境变量

| 变量 | 必需 | 默认值 | 说明 |
|------|------|--------|------|
| `AIDAGUARD_API_KEY` | **是** | — | 上游 API 认证 Key，自动补充 `Bearer ` 前缀 |
| `AIDAGUARD_TARGET_URL` | 否 | `https://qianfan.baidubce.com/v2/coding` | 上游 API 基础 URL |
| `AIDAGUARD_RULES_DIR` | 否 | `./rules` | YAML 规则文件目录 |
| `RUST_LOG` | 否 | `info` | 日志级别 (tracing env-filter) |

---

## 启动与测试

### 启动代理

```bash
AIDAGUARD_API_KEY='你的API Key' cargo run --bin aidaguard
```

### 启动 Mock SSE 测试服务器

```bash
cargo run --bin mock-sse
# 监听 127.0.0.1:19999，模拟 SSE 流式响应
```

### 运行测试

```bash
cargo test                    # 全部测试
cargo test stream             # 仅流式模块测试 (5 tests)
cargo test detector           # 仅检测模块测试 (7 tests)
cargo test replacer           # 仅替换模块测试 (6 tests)
```

### 代理端口

- 代理监听：`127.0.0.1:19000`
- Mock SSE：`127.0.0.1:19999`

---

## 待实现模块

| 模块 | 文件 | 状态 |
|------|------|------|
| 通用请求转发器 | `proxy/forwarder.rs` | 仅占位注释 |
| 加密存储 | `storage/mod.rs` | 仅占位注释（依赖 `rusqlite` + `aes-gcm` 已就绪） |
| Tauri 桌面客户端 | `crates/aidaguard-tauri/` | 未创建 |

---

## 已知限制

1. **仅处理 UTF-8 文本请求体** — 二进制 body 跳过检测与还原
2. **非流式响应需完整加载** — 大响应可能占用较多内存
3. **swift_code 规则使用了 look-ahead** — Rust `regex` crate 不支持，该规则编译会失败并跳过（日志中有 warn）
4. **流式还原仅处理 content 和 tool_calls[0].function.arguments** — 其他文本字段（如 reasoning_content）不做还原
