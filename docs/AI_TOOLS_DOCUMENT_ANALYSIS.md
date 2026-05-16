# AI 工具文档发送行为分析

## 一、分析目标

分析各 AI 编程工具是否会直接将二进制文档文件发送到大模型：
- Claude Code
- Codex
- OpenClaw
- Cursor
- Cline
- Windsurf
- Aider
- Zed

---

## 二、工具分类

### 2.1 CLI 工具类

| 工具 | 类型 | 文档发送方式 |
|------|------|--------------|
| Claude Code | CLI | ❌ 不支持文件上传 |
| Codex | CLI | ❌ 不支持文件上传 |
| Aider | CLI | ⚠️ 仅支持图片 |
| OpenClaw | CLI | ❌ 不支持文件上传 |

### 2.2 IDE/编辑器类

| 工具 | 类型 | 文档发送方式 |
|------|------|--------------|
| Cursor | IDE | ⚠️ 粘贴文本，不支持二进制文件 |
| Windsurf | IDE | ⚠️ 粘贴文本，不支持二进制文件 |
| Zed | IDE | ❌ 不支持文件上传 |
| VS Code + Cline | 插件 | ⚠️ 读取编辑器文本内容 |

### 2.3 Web AI 类（对照）

| 工具 | 类型 | 文档发送方式 |
|------|------|--------------|
| ChatGPT | Web | ✅ 支持文件上传 |
| Claude Web | Web | ✅ 支持文件上传 |
| Gemini Web | Web | ✅ 支持文件上传 |

---

## 三、各工具详细分析

### 3.1 Claude Code

**工具类型：** Anthropic 官方 CLI 工具

**文件处理方式：**
```bash
# Claude Code 的典型使用方式
claude-code "请帮我分析这段代码"

# 使用方式 1：直接输入文本
claude-code "我的身份证号是 310101199001011234，帮我检查格式"

# 使用方式 2：管道输入文本
cat document.txt | claude-code "分析这个文件"

# 使用方式 3：指定本地文件路径（读取文本文件）
claude-code --file README.md "解释这个文件"
```

**关键发现：**
- Claude Code **不支持直接上传二进制文档**
- 所有输入都是 **纯文本格式**
- `--file` 参数只能读取 **文本文件**（代码、Markdown、JSON 等）
- 不支持 Word、Excel、PDF 等二进制格式

**API 请求格式：**
```json
{
  "model": "claude-3-opus",
  "max_tokens": 4096,
  "messages": [
    {
      "role": "user",
      "content": "分析这个文本内容：\n\n合同编号：HT-2024-001\n甲方：张三..."
    }
  ]
}
```

**AIDAGuard 拦截：** ✅ 已支持（纯文本检测）

---

### 3.2 Codex (OpenAI CLI)

**工具类型：** OpenAI 官方 CLI 工具

**文件处理方式：**
```bash
# Codex 的典型使用方式
codex "帮我写一个 Python 脚本"

# 输入文本
codex "请分析这段代码：def foo(): pass"

# 不支持文件上传
# codex --upload document.docx ← 不存在此功能
```

**关键发现：**
- Codex **不支持文件上传**
- 仅处理纯文本输入
- 无二进制文档支持

**API 请求格式：**
```json
{
  "model": "gpt-4",
  "messages": [
    {
      "role": "user",
      "content": "文本内容..."
    }
  ]
}
```

**AIDAGuard 拦截：** ✅ 已支持

---

### 3.3 OpenClaw

**工具类型：** 开源 AI Agent 框架

**配置文件分析：**
```json
// ~/.openclaw/openclaw.json
{
  "models": {
    "providers": {
      "openai": {
        "baseUrl": "https://api.openai.com/v1"
      }
    }
  },
  "agents": {
    "defaults": {
      "model": {
        "primary": "gpt-4"
      }
    }
  }
}
```

**文件处理方式：**
```bash
# OpenClaw 使用方式
openclaw run "帮我分析项目结构"

# 读取文本文件
openclaw run --context README.md "解释项目"

# 不支持二进制文档上传
```

**关键发现：**
- OpenClaw 是 Agent 框架，主要处理 **代码和文本**
- `--context` 参数读取文本文件内容
- **不支持** Word/Excel/PDF 等二进制文档
- API 调用格式与 OpenAI 兼容（纯文本 JSON）

**AIDAGuard 拦截：** ✅ 已支持

---

### 3.4 Cursor

**工具类型：** AI-first 代码编辑器（VS Code 分支）

**文件处理方式：**

```
用户操作 → Cursor 内部处理 → API 调用
```

**场景分析：**

| 用户操作 | Cursor 处理 | 发送到 API | AIDAGuard 拦截 |
|----------|-------------|------------|----------------|
| 打开代码文件 | 读取编辑器内容 | 纯文本 | ✅ 已支持 |
| 粘贴文本内容 | 直接使用 | 纯文本 | ✅ 已支持 |
| Ctrl+K 内联编辑 | 读取选中文本 | 纯文本 | ✅ 已支持 |
| Ctrl+L 聊天 | 聊天内容 | 纯文本 | ✅ 已支持 |
| 打开 Word 文档 | **不支持** | - | ❌ 无需拦截 |
| 打开 PDF 文件 | **不支持** | - | ❌ 无需拦截 |

**关键发现：**
- Cursor **不直接支持** 打开 Word/Excel/PPT/PDF
- 用户打开 Word 文档时，Cursor 会提示 "无法打开此文件类型"
- 用户需要：
  1. 在 Word 中打开文档
  2. 复制文本内容
  3. 粘贴到 Cursor 中
- 粘贴的内容已经是 **纯文本**，AIDAGuard 可以检测

**实际 API 请求：**
```json
{
  "model": "gpt-4",
  "messages": [
    {
      "role": "user",
      "content": "用户粘贴的文本内容：\n\n合同编号：HT-2024-001\n甲方：张三（身份证 310101...）\n乙方：李四公司\n..."
    }
  ]
}
```

**AIDAGuard 拦截：** ✅ 完全支持

---

### 3.5 Cline (VS Code 插件)

**工具类型：** VS Code AI 编程插件

**文件处理方式：**
```
VS Code 编辑器 → Cline 读取内容 → API 调用
```

**场景分析：**

| 文件类型 | VS Code 支持 | Cline 处理 | 发送到 API |
|----------|--------------|------------|------------|
| .txt / .md | ✅ | 读取内容 | 纯文本 |
| .js / .py / .rs | ✅ | 读取代码 | 纯文本 |
| .json / .yaml | ✅ | 读取内容 | 纯文本 |
| .docx | ❌ VS Code 无法打开 | - | 无 |
| .xlsx | ❌ VS Code 无法打开 | - | 无 |
| .pdf | ❌ VS Code 无法打开 | - | 无 |

**关键发现：**
- Cline 依赖 VS Code 的文件打开能力
- VS Code **不原生支持** Word/Excel/PDF
- 需要额外插件才能预览这些格式
- 即使安装插件，Cline 也只会读取 **文本内容**

**AIDAGuard 拦截：** ✅ 已支持

---

### 3.6 Windsurf

**工具类型：** AI IDE（类似 Cursor）

**分析结论：** 与 Cursor 相同
- 不支持直接打开 Word/Excel/PDF
- 用户需要粘贴文本内容
- API 请求为纯文本格式

**AIDAGuard 拦截：** ✅ 已支持

---

### 3.7 Aider

**工具类型：** CLI AI 编程助手

**特殊能力：** 支持 **图片** 输入

```bash
# Aider 支持图片
aider --image screenshot.png "解释这个截图"

# 但不支持 Word/Excel/PDF
```

**关键发现：**
- Aider 支持 **图片文件**（PNG/JPEG）
- 图片以 Base64 编码发送到 API
- 这是 **唯一需要特殊处理** 的场景

**API 请求格式（图片）：**
```json
{
  "model": "gpt-4-vision",
  "messages": [
    {
      "role": "user",
      "content": [
        {
          "type": "text",
          "text": "解释这个截图"
        },
        {
          "type": "image_url",
          "image_url": {
            "url": "data:image/png;base64,iVBORw0KGgoAAAANS..."
          }
        }
      ]
    }
  ]
}
```

**AIDAGuard 需处理：**
- 图片 Base64 解码
- OCR 提取文字（可选）
- 或：跳过图片内容，仅检测文本部分

---

### 3.8 Zed

**工具类型：** 高性能编辑器 + AI 功能

**分析结论：**
- Zed 是代码编辑器
- 主要处理代码和文本文件
- **不支持** Word/Excel/PDF

**AIDAGuard 拦截：** ✅ 已支持

---

## 四、总结对比

### 4.1 文档发送方式汇总

| 工具 | Word | Excel | PDF | 图片 | 纯文本 |
|------|------|-------|-----|------|--------|
| Claude Code | ❌ | ❌ | ❌ | ❌ | ✅ |
| Codex | ❌ | ❌ | ❌ | ❌ | ✅ |
| OpenClaw | ❌ | ❌ | ❌ | ❌ | ✅ |
| Cursor | ❌ | ❌ | ❌ | ❌ | ✅ |
| Cline | ❌ | ❌ | ❌ | ❌ | ✅ |
| Windsurf | ❌ | ❌ | ❌ | ❌ | ✅ |
| Aider | ❌ | ❌ | ❌ | ✅ | ✅ |
| Zed | ❌ | ❌ | ❌ | ❌ | ✅ |
| ChatGPT Web | ✅ | ✅ | ✅ | ✅ | ✅ |
| Claude Web | ✅ | ❌ | ✅ | ✅ | ✅ |

### 4.2 关键结论

**结论 1：AI 编程工具不直接发送二进制文档**

- 所有 AI 编程工具（CLI + IDE）**不支持** Word/Excel/PPT/PDF 文件上传
- 用户只能：
  1. 复制粘贴文本内容
  2. 或使用工具读取文本文件
- **文档解析发生在客户端**，而非 API 请求

**结论 2：文档内容以纯文本形式发送**

```
用户流程：
打开 Word 文档 → 复制内容 → 粘贴到 AI 工具 → 文本发送给 API

AIDAGuard 拦截点：
                              ┌────────────────┐
                              │ API 请求体     │
                              │ {              │
                              │   "content":   │
                              │   "文本内容"   │ ← AIDAGuard 检测 ✅
                              │ }              │
                              └────────────────┘
```

**结论 3：Aider 的图片支持是唯一例外**

- 图片以 Base64 编码发送
- 需要 OCR 或视觉模型处理
- 建议：跳过图片检测，或添加 OCR 功能

---

## 五、对 AIDAGuard 的建议

### 5.1 当前支持已足够

| AI 工具类型 | 文档处理位置 | AIDAGuard 拦截 | 覆盖率 |
|-------------|--------------|----------------|--------|
| CLI 工具 | 客户端（纯文本） | ✅ 已支持 | 100% |
| IDE 工具 | 客户端（纯文本） | ✅ 已支持 | 100% |
| Aider | 图片 Base64 | ⚠️ 可选支持 | - |

### 5.2 Phase 3 文档过滤的真实场景

**Phase 3 规划的文档过滤适用于：**

| 场景 | 使用者 | 发送方式 | 优先级 |
|------|--------|----------|--------|
| ChatGPT Web 文件上传 | Web 用户 | Base64/URL | P1 |
| Claude Web 文件上传 | Web 用户 | Base64 | P1 |
| 企业自研 AI 应用 | 企业用户 | 可能 Base64 | P2 |
| Aider 图片输入 | CLI 用户 | Base64 图片 | P3 |

**不适用于：**
- Cursor、Claude Code、Cline 等编程工具
- 这些工具的文档内容已经是纯文本，AIDAGuard 已能检测

### 5.3 实施优先级调整建议

| 原规划 | 调整后 | 原因 |
|--------|--------|------|
| P1: Base64 PDF | → P1 | Web AI 用户场景 |
| P2: Base64 Word/Excel | → P2 | Web AI 用户场景 |
| P3: 文件 URL | → P2 | 企业部署场景 |
| P4: OCR 图片 | → P3 | Aider 场景，用户量小 |

---

## 六、最终结论

**核心发现：**

> AI 编程工具（Claude Code、Codex、OpenClaw、Cursor、Cline 等）
> **不会** 直接将 Word/Excel/PPT/PDF 文件发送到大模型。
> 
> 用户必须先复制文本内容，再粘贴到工具中。
> 
> AIDAGuard 对纯文本的检测 **已 100% 覆盖** 这些场景。

**Phase 3 文档过滤的真正价值：**

> 面向 Web AI（ChatGPT/Claude Web）用户，
> 他们可能通过网页直接上传文档文件。
> 
> 这是企业部署场景的重要需求。

**建议：**

> 保持 Phase 3 规划，但明确其适用场景为 Web AI 用户，
> 而非 AI 编程工具（后者已完全覆盖）。