# LLM 文档处理流程分析

## 一、主流 LLM API 文档处理方式

### 1.1 OpenAI GPT-4 Vision / GPT-4o

**支持的文档格式：**
- 图片：PNG, JPEG, GIF, WebP
- PDF（通过 GPT-4o 的多模态能力）
- **不直接支持 Word/Excel/PPT**

**处理流程：**
```
用户上传文件 → OpenAI 服务器 → 文件解析 → 多模态编码 → 模型输入
```

**PDF 处理方式：**
1. 每页 PDF 渲染为图片
2. 使用 Vision 模型识别图片中的文字
3. 将识别结果作为上下文输入模型

**限制：**
- 单文件最大 512MB
- 文本通过 OCR 提取，可能丢失格式
- 复杂表格识别率有限

---

### 1.2 Anthropic Claude

**支持的文档格式：**
- PDF, TXT, CSV, JSON, MD
- 图片：PNG, JPEG, GIF, WebP
- **不直接支持 Word/Excel/PPT**

**处理流程：**
```
用户上传文件 → Claude API → 文档解析器 → 文本/图片提取 → 模型输入
```

**PDF 处理方式：**
```python
# Claude API 示例
import anthropic

client = anthropic.Anthropic()

with open("document.pdf", "rb") as f:
    doc_data = f.read()

message = client.messages.create(
    model="claude-3-opus-20240229",
    max_tokens=1024,
    messages=[
        {
            "role": "user",
            "content": [
                {
                    "type": "document",
                    "source": {
                        "type": "base64",
                        "media_type": "application/pdf",
                        "data": base64.b64encode(doc_data).decode()
                    }
                },
                {
                    "type": "text",
                    "text": "请分析这份文档"
                }
            ]
        }
    ]
)
```

**Claude 文档解析特点：**
- 内部 PDF 解析器提取文本
- 保留部分结构信息（标题、段落）
- 表格转为近似 Markdown 格式
- 图片嵌入会作为图片内容处理

---

### 1.3 Google Gemini

**支持的文档格式：**
- PDF, TXT, CSV, JSON, MD
- 图片、视频、音频
- **部分支持 Word/Excel**（通过 Google Docs 转换）

**处理流程：**
```
用户上传文件 → Gemini API → Google 文档服务转换 → 多模态编码 → 模型输入
```

**Word/Excel 处理方式：**
- 利用 Google Docs/Sheets 后端转换
- 转为结构化文本后输入模型
- 保留部分格式信息

---

### 1.4 国产大模型（通义千问、文心一言、智谱 GLM）

**通义千问 (Qwen)：**
- 支持 PDF、图片
- Word/Excel 需转为 PDF 或文本

**文心一言：**
- 支持 PDF、图片
- 通过百度文档服务处理 Office 格式

**智谱 GLM：**
- 支持 PDF、图片
- Office 格式需要预处理

---

## 二、文档发送到 LLM 的实际解析流程

### 2.1 客户端预处理模式（主流）

```
┌─────────────────────────────────────────────────────────────────────┐
│                          客户端应用                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────┐     ┌──────────────┐     ┌──────────────┐         │
│  │ 用户上传     │ ──▶ │ 文档解析库   │ ──▶ │ 文本/图片    │         │
│  │ Word/Excel  │     │ (本地处理)   │     │ 提取         │         │
│  └─────────────┘     └──────────────┘     └──────────────┘         │
│                                                  │                  │
│                                                  ▼                  │
│  ┌─────────────┐     ┌──────────────┐     ┌──────────────┐         │
│  │ API 请求    │ ◀── │ JSON 序列化  │ ◀── │ Base64 编码  │         │
│  │ 发送        │     │              │     │              │         │
│  └─────────────┘     └──────────────┘     └──────────────┘         │
│         │                                                          │
└─────────│──────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────────────┐
│                          LLM API 服务器                              │
│                                                                     │
│  ┌─────────────┐     ┌──────────────┐     ┌──────────────┐         │
│  │ 接收请求    │ ──▶ │ Token 编码   │ ──▶ │ 模型推理     │         │
│  │             │     │              │     │              │         │
│  └─────────────┘     └──────────────┘     └──────────────┘         │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 服务端处理模式

```
┌─────────────────────────────────────────────────────────────────────┐
│                          客户端应用                                  │
│                                                                     │
│  ┌─────────────┐     ┌──────────────┐                              │
│  │ 用户上传     │ ──▶ │ 直接上传     │ ──▶ API 请求                 │
│  │ 文档文件    │     │ 二进制/URL   │                              │
│  └─────────────┘     └──────────────┘                              │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                          LLM API 服务器                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────┐     ┌──────────────┐     ┌──────────────┐         │
│  │ 接收文件    │ ──▶ │ 文档解析器   │ ──▶ │ 内容编码     │         │
│  │            │     │ (服务端处理) │     │              │         │
│  └─────────────┘     └──────────────┘     └──────────────┘         │
│                                                  │                  │
│                                                  ▼                  │
│                     ┌──────────────┐     ┌──────────────┐          │
│                     │ 模型推理     │ ◀── │ Token 编码   │          │
│                     │              │     │              │          │
│                     └──────────────┘     └──────────────┘          │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 三、AI 编程工具的文档处理方式

### 3.1 Cursor

**文档处理流程：**

```
用户打开/粘贴文档 → Cursor 提取文本 → 构建 prompt → 发送给 LLM
```

**Word 文档处理：**
- 方式 1：用户复制粘贴文本 → 直接作为 prompt
- 方式 2：文件引用 → 读取纯文本内容

**实际发送给 LLM 的格式：**
```json
{
  "messages": [
    {
      "role": "user",
      "content": "分析以下文档内容：\n\n【文档标题】\n合同编号：HT-2024-001\n...\n\n请帮我找出其中的敏感信息"
    }
  ]
}
```

**关键点：**
- Cursor **不直接处理二进制文档格式**
- 用户需要先打开文档，复制内容
- 或使用 Cursor 的文件读取功能（仅限文本文件）

---

### 3.2 Claude Code CLI

**文档处理：**

```bash
# 用户粘贴文档内容
claude-code "分析这份合同：
---
合同编号：HT-2024-001
甲方：张三 (身份证号：310101199001011234)
乙方：李四公司
...
---"
```

**文件引用：**
```bash
# 读取本地文本文件
claude-code --file contract.txt "分析这份合同"
```

**不支持：**
- 直接上传 Word/Excel/PDF 文件
- 二进制格式解析

---

### 3.3 Cline (VS Code 插件)

**文档处理流程：**

```
用户在 VS Code 打开文档 → 读取编辑器内容 → 发送给 LLM
```

**支持格式：**
- 纯文本文件（.txt, .md, .json 等）
- 代码文件（.js, .py, .rs 等）
- **不支持 Word/Excel/PDF**

---

### 3.4 ChatGPT / Claude Web

**文档上传处理：**

```
用户上传文件 → 服务端解析 → 提取文本/图片 → 多模态编码 → 模型处理
```

**服务端解析器实现推测：**

```python
# PDF 解析（推测）
import fitz  # PyMuPDF

def parse_pdf(file_bytes):
    doc = fitz.open(stream=file_bytes, filetype="pdf")
    text = ""
    for page in doc:
        text += page.get_text()
    return text

# Word 解析（推测）
from docx import Document

def parse_docx(file_bytes):
    doc = Document(io.BytesIO(file_bytes))
    text = ""
    for para in doc.paragraphs:
        text += para.text + "\n"
    return text
```

---

## 四、AIDAGuard 代理层的拦截点分析

### 4.1 当前代理流程

```
AI 工具 → HTTP 请求 → AIDAGuard 代理 → 上游 LLM API
             │
             ▼
        请求体 (JSON)
        {
          "model": "gpt-4",
          "messages": [
            {"role": "user", "content": "文本内容..."}
          ]
        }
```

**关键发现：**
- 大多数 AI 编程工具 **在客户端已经完成文档解析**
- 发送到 API 的请求体已经是 **纯文本格式**
- AIDAGuard 检测的是 **已提取的文本内容**

### 4.2 文档内容在 API 请求中的位置

**OpenAI 格式：**
```json
{
  "model": "gpt-4o",
  "messages": [
    {
      "role": "user",
      "content": "请分析以下合同内容：\n\n合同编号：HT-2024-001\n甲方：张三，身份证号 310101199001011234\n..."
    }
  ]
}
```

**多模态格式（图片/PDF）：**
```json
{
  "model": "gpt-4o",
  "messages": [
    {
      "role": "user",
      "content": [
        {
          "type": "text",
          "text": "请分析这份文档"
        },
        {
          "type": "image_url",
          "image_url": {
            "url": "data:application/pdf;base64,JVBERi0xLjQK..."
          }
        }
      ]
    }
  ]
}
```

---

## 五、AIDAGuard 文档过滤的实现策略

### 5.1 场景分类

| 场景 | 文档处理位置 | AIDAGuard 拦截点 | 难度 |
|------|--------------|------------------|------|
| **场景 A** | 客户端解析 | 文本内容已在 JSON 中 | ✅ 已支持 |
| **场景 B** | 服务端解析 | 文档以 Base64 发送 | ⚠️ 需新增 |
| **场景 C** | 文件上传 | 文件 URL 或 multipart | ⚠️ 需新增 |

### 5.2 场景 A：客户端解析（当前已支持）

```
用户复制粘贴 → AI 工具解析 → 文本内容 → AIDAGuard 检测 ✅
```

**示例：**
```json
// AIDAGuard 已能检测
{
  "messages": [
    {
      "role": "user", 
      "content": "我的身份证号是 310101199001011234，帮我检查下"
    }
  ]
}
```

### 5.3 场景 B：Base64 编码文档

```
用户上传文件 → AI 工具 Base64 编码 → 发送到 API → AIDAGuard 需解码检测
```

**示例请求：**
```json
{
  "messages": [
    {
      "role": "user",
      "content": [
        {
          "type": "text",
          "text": "分析这份合同"
        },
        {
          "type": "image_url",
          "image_url": {
            "url": "data:application/pdf;base64,JVBERi0xLjQKJeLjz9MKMSAwIG9iago8PC..."
          }
        }
      ]
    }
  ]
}
```

**AIDAGuard 需要做的：**
```rust
// 1. 识别 Base64 编码的文档
if content.starts_with("data:application/pdf;base64,") {
    let pdf_bytes = base64_decode(&content[28..]);
    
    // 2. 提取文本
    let text = extract_pdf_text(&pdf_bytes)?;
    
    // 3. 检测敏感数据
    let matches = detector.scan(&text);
    
    // 4. 脱敏处理
    if !matches.is_empty() {
        let sanitized = sanitize_pdf(&pdf_bytes, &matches)?;
        let new_base64 = base64_encode(&sanitized);
        // 替换请求体中的 Base64
    }
}
```

### 5.4 场景 C：文件上传（URL / multipart）

```
用户上传文件 → 存储服务 → 返回 URL → URL 发送给 LLM
```

**示例：**
```json
{
  "messages": [
    {
      "role": "user",
      "content": [
        {
          "type": "file",
          "file": {
            "url": "https://files.openai.com/abc123.pdf"
          }
        }
      ]
    }
  ]
}
```

**处理难度：**
- 需要下载文件
- 需要处理 URL 有效期
- 需要存储脱敏后的文件

---

## 六、推荐实现方案

### 6.1 短期方案：场景 A 增强

**目标：** 确保客户端解析的文本内容 100% 覆盖

**现状：** ✅ 已支持

**增强点：**
- 提示用户复制粘贴文档内容
- 在 AI 工具中显示检测到的敏感数据

### 6.2 中期方案：场景 B 支持

**目标：** 支持 Base64 编码的 PDF/图片

**实现步骤：**

```rust
// crates/aidaguard-proxy/src/document_interceptor.rs

use base64::{Engine as _, engine::general_purpose};

/// 拦截并处理 Base64 编码的文档
pub fn intercept_base64_document(
    content: &str,
    detector: &AnalyzerEngine,
) -> Result<Option<String>, ProxyError> {
    // 检测 PDF
    if content.starts_with("data:application/pdf;base64,") {
        return process_pdf_base64(content, detector);
    }
    
    // 检测图片中的文字（OCR）
    if content.starts_with("data:image/") {
        return process_image_base64(content, detector);
    }
    
    // 检测 Word
    if content.starts_with("data:application/vnd.openxmlformats-officedocument.wordprocessingml.document;base64,") {
        return process_word_base64(content, detector);
    }
    
    Ok(None)
}

fn process_pdf_base64(content: &str, detector: &AnalyzerEngine) -> Result<Option<String>, ProxyError> {
    let base64_data = &content[28..]; // 去掉 "data:application/pdf;base64,"
    let pdf_bytes = general_purpose::STANDARD.decode(base64_data)?;
    
    // 提取文本
    let text = pdf_extract::extract_text_from_mem(&pdf_bytes)?;
    
    // 检测
    let matches = detector.scan(&text);
    
    if matches.is_empty() {
        return Ok(None);
    }
    
    // 警告用户（不修改 PDF，因为难以精确脱敏）
    tracing::warn!(
        "PDF 中检测到 {} 处敏感数据，建议用户检查后再发送",
        matches.len()
    );
    
    // 可选：返回脱敏后的文本提示
    Ok(Some(format!(
        "⚠️ 检测到文档包含 {} 处敏感数据，已记录。建议检查后再发送。",
        matches.len()
    )))
}
```

### 6.3 长期方案：完整文档处理

**目标：** 支持 Word/Excel/PPT 的完整脱敏

**架构：**

```
┌─────────────────────────────────────────────────────────────────────┐
│                        AIDAGuard Proxy                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    请求拦截层                                │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │   │
│  │  │ JSON 文本   │  │ Base64 文档 │  │ 文件 URL   │          │   │
│  │  │ 检测        │  │ 解码+检测   │  │ 下载+检测  │          │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘         │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              │                                      │
│                              ▼                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    文档处理层                                │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │   │
│  │  │ PDF 处理器  │  │ Word 处理器 │  │ Excel 处理器│         │   │
│  │  │ pdf-extract │  │ docx-rs     │  │ calamine    │         │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘         │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              │                                      │
│                              ▼                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    脱敏处理层                                │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │   │
│  │  │ 文本替换    │  │ 占位符生成  │  │ 文档重建    │         │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘         │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 七、结论

### 关键发现

1. **大多数 AI 编程工具在客户端完成文档解析**
   - Cursor、Cline、Claude Code 等不直接上传二进制文档
   - 用户复制粘贴的内容已经是纯文本
   - AIDAGuard 当前已能检测这些内容

2. **Web 端 AI（ChatGPT、Claude Web）支持文件上传**
   - 文档在服务端解析
   - 可能以 Base64 或 URL 形式发送
   - 这是 AIDAGuard 需要拦截的新场景

3. **PDF 是最常见的文档格式**
   - 技术最成熟
   - 检测可行，脱敏有挑战
   - 建议优先支持

### 实施建议

| 优先级 | 场景 | 工作量 | 建议 |
|--------|------|--------|------|
| **P0** | 客户端解析的文本 | ✅ 已完成 | 持续优化 |
| **P1** | Base64 PDF | 3-5 天 | 优先实现 |
| **P2** | Base64 Word/Excel | 5-7 天 | 中期实现 |
| **P3** | 文件 URL | 7-10 天 | 长期规划 |

### 与 Phase 3 文档过滤的关联

Phase 3 规划的文档过滤功能与 LLM 实际处理流程高度匹配：

1. **文本提取** → 对应客户端已解析的文本检测
2. **Base64 解码** → 对应服务端处理模式
3. **位置感知脱敏** → 对应精确脱敏需求

建议按 Phase 3 规划实施，优先完成 PDF 检测。
