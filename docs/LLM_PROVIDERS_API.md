# 主流大模型提供商接口分析

**文档版本：** v1.0  
**更新日期：** 2026-05-16  
**用途：** AIDAGuard Phase 4.1 技术参考

---

## 一、提供商总览

| 提供商 | 公司/组织 | API 协议 | 认证方式 | 计费方式 | 官方文档 |
|--------|-----------|----------|----------|----------|----------|
| OpenAI | OpenAI | OpenAI Compatible | Bearer Token | 按量计费 | https://platform.openai.com/docs |
| Anthropic | Anthropic | Anthropic Compatible | x-api-key Header | 按量计费 | https://docs.anthropic.com |
| DeepSeek | 深度求索 | OpenAI Compatible | Bearer Token | 按量计费 | https://platform.deepseek.com/docs |
| Qwen | 阿里云 | OpenAI Compatible | Bearer Token | 按量计费 | https://help.aliyun.com/zh/dashscope |
| Zhipu | 智谱AI | OpenAI Compatible | Bearer Token | 按量计费 | https://open.bigmodel.cn/dev/api |
| Gemini | Google | Gemini SDK | API Key Param | 按量计费 | https://ai.google.dev/docs |
| Groq | Groq | OpenAI Compatible | Bearer Token | Free/Paid | https://console.groq.com/docs |
| Moonshot | 月之暗面 | OpenAI Compatible | Bearer Token | 按量计费 | https://platform.moonshot.cn/docs |
| Qianfan | 百度智能云 | OpenAI Compatible | Bearer Token | 按量计费 | https://cloud.baidu.com/doc/WENXINWORKSHOP |
| Doubao | 字节跳动 | OpenAI Compatible | Bearer Token | 按量计费 | https://www.volcengine.com/docs/82379 |
| SiliconFlow | SiliconFlow | OpenAI Compatible | Bearer Token | 按量计费 | https://docs.siliconflow.cn |
| Minimax | Minimax | OpenAI Compatible | Bearer Token | 按量计费 | https://www.minimaxi.com/document |
| Yi | 零一万物 | OpenAI Compatible | Bearer Token | 按量计费 | https://platform.lingyiwanwu.com/docs |

---

## 二、API 接口详情

### 2.1 OpenAI

**端点：** `https://api.openai.com/v1`

**认证方式：**
```http
Authorization: Bearer sk-xxx
```

**主要接口：**
```
POST /chat/completions    # 对话补全
POST /completions         # 文本补全（旧版）
POST /embeddings          # 向量化
POST /models              # 模型列表
```

**请求示例：**
```json
{
  "model": "gpt-4o",
  "messages": [
    {"role": "user", "content": "Hello"}
  ],
  "max_tokens": 1024,
  "temperature": 0.7
}
```

**响应格式：**
```json
{
  "id": "chatcmpl-xxx",
  "object": "chat.completion",
  "created": 1234567890,
  "model": "gpt-4o",
  "choices": [...],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 20,
    "total_tokens": 30
  }
}
```

**价格模型：**
| 模型 | 输入价格 | 输出价格 | 单位 |
|------|----------|----------|------|
| gpt-4o | $2.50 | $10.00 | /1M tokens |
| gpt-4o-mini | $0.15 | $0.60 | /1M tokens |
| gpt-4-turbo | $10.00 | $30.00 | /1M tokens |
| gpt-3.5-turbo | $0.50 | $1.50 | /1M tokens |

**Token 统计：** 响应中 `usage` 字段包含完整统计

---

### 2.2 Anthropic

**端点：** `https://api.anthropic.com/v1`

**认证方式：**
```http
x-api-key: sk-ant-xxx
anthropic-version: 2023-06-01
```

**主要接口：**
```
POST /messages           # 对话补全
POST /messages/count_tokens  # Token 计数
```

**请求示例：**
```json
{
  "model": "claude-3-5-sonnet-20241022",
  "max_tokens": 1024,
  "messages": [
    {"role": "user", "content": "Hello"}
  ]
}
```

**响应格式：**
```json
{
  "id": "msg_xxx",
  "type": "message",
  "role": "assistant",
  "content": [...],
  "model": "claude-3-5-sonnet-20241022",
  "usage": {
    "input_tokens": 10,
    "output_tokens": 20
  }
}
```

**价格模型：**
| 模型 | 输入价格 | 输出价格 | 单位 |
|------|----------|----------|------|
| claude-3.5-sonnet | $3.00 | $15.00 | /1M tokens |
| claude-3.5-haiku | $0.80 | $4.00 | /1M tokens |
| claude-3-opus | $15.00 | $75.00 | /1M tokens |

**Token 统计：** 响应中 `usage` 字段（注意：字段名不同于 OpenAI）

---

### 2.3 DeepSeek

**端点：** `https://api.deepseek.com/v1` 或 `https://api.deepseek.com/chat/completions`

**认证方式：**
```http
Authorization: Bearer sk-xxx
```

**主要接口：** 完全兼容 OpenAI 格式

**价格模型：**
| 模型 | 输入价格 | 输出价格 | 单位 |
|------|----------|----------|------|
| deepseek-chat | $0.27 | $1.10 | /1M tokens |
| deepseek-coder | $0.27 | $1.10 | /1M tokens |
| deepseek-reasoner | $0.55 | $2.19 | /1M tokens |

**特色：** 支持深度思考模式（deepseek-reasoner），性价比高

---

### 2.4 Qwen (阿里云 DashScope)

**端点：** `https://dashscope.aliyuncs.com/compatible-mode/v1`

**认证方式：**
```http
Authorization: Bearer sk-xxx
```

**主要接口：** 兼容 OpenAI 格式

**价格模型：**
| 模型 | 输入价格 | 输出价格 | 单位 |
|------|----------|----------|------|
| qwen-max | ¥2.00 | ¥8.00 | /1M tokens |
| qwen-plus | ¥0.40 | ¥2.00 | /1M tokens |
| qwen-turbo | ¥0.30 | ¥0.60 | /1M tokens |
| qwen-long | ¥0.50 | ¥2.00 | /1M tokens |

**特色：** 长上下文模型（qwen-long）支持 1M tokens

---

### 2.5 Zhipu (智谱 AI)

**端点：** `https://open.bigmodel.cn/api/paas/v4`

**认证方式：**
```http
Authorization: Bearer xxx.xxx
```

**主要接口：** 兼容 OpenAI 格式

**价格模型：**
| 模型 | 输入价格 | 输出价格 | 单位 |
|------|----------|----------|------|
| glm-4 | ¥0.10 | ¥0.10 | /1K tokens |
| glm-4-air | ¥0.001 | ¥0.001 | /1K tokens |
| glm-4-flash | 免费 | 免费 | - |

**特色：** GLM-4-flash 免费使用，适合测试

---

### 2.6 Gemini

**端点：** `https://generativelanguage.googleapis.com/v1beta`

**认证方式：**
```
URL 参数: ?key=xxx
```

**主要接口：**
```
POST /models/{model}:generateContent
POST /models/{model}:streamGenerateContent
GET /models                    # 模型列表
```

**请求示例：**
```json
{
  "contents": [
    {
      "parts": [{"text": "Hello"}]
    }
  ],
  "generationConfig": {
    "maxOutputTokens": 1024,
    "temperature": 0.7
  }
}
```

**响应格式：**
```json
{
  "candidates": [...],
  "usageMetadata": {
    "promptTokenCount": 10,
    "candidatesTokenCount": 20,
    "totalTokenCount": 30
  }
}
```

**价格模型：**
| 模型 | 输入价格 | 输出价格 | 单位 |
|------|----------|----------|------|
| gemini-2.0-flash | $0.10 | $0.40 | /1M tokens |
| gemini-1.5-pro | $1.25 | $5.00 | /1M tokens |
| gemini-1.5-flash | $0.075 | $0.30 | /1M tokens |

**Token 统计：** 响应中 `usageMetadata` 字段（注意字段名不同）

---

### 2.7 Groq

**端点：** `https://api.groq.com/openai/v1`

**认证方式：**
```http
Authorization: Bearer gsk_xxx
```

**主要接口：** 完全兼容 OpenAI 格式

**价格模型：**
| 模型 | 输入价格 | 输出价格 | 单位 |
|------|----------|----------|------|
| llama-3.3-70b-versatile | 免费 | 免费 | Free tier |
| llama-3.1-8b-instant | 免费 | 免费 | Free tier |
| mixtral-8x7b-32768 | 免费 | 免费 | Free tier |

**特色：** 超快推理速度，免费额度充足

---

### 2.8 Moonshot (月之暗面)

**端点：** `https://api.moonshot.cn/v1`

**认证方式：**
```http
Authorization: Bearer sk-xxx
```

**主要接口：** 兼容 OpenAI 格式

**价格模型：**
| 模型 | 输入价格 | 输出价格 | 单位 |
|------|----------|----------|------|
| moonshot-v1-8k | ¥0.012 | ¥0.012 | /1K tokens |
| moonshot-v1-32k | ¥0.024 | ¥0.024 | /1K tokens |
| moonshot-v1-128k | ¥0.06 | ¥0.06 | /1K tokens |

**特色：** 支持长上下文（128K）

---

### 2.9 百度千帆 (Qianfan)

**端点：** `https://qianfan.baidubce.com/v2`

**认证方式：**
```http
Authorization: Bearer xxx
```

**主要接口：** 兼容 OpenAI 格式

**价格模型：**
| 模型 | 输入价格 | 输出价格 | 单位 |
|------|----------|----------|------|
| ERNIE-4.0-8K | ¥0.12 | ¥0.12 | /1K tokens |
| ERNIE-3.5-8K | ¥0.04 | ¥0.08 | /1K tokens |
| ERNIE-Speed | 免费 | 免费 | - |

---

### 2.10 字节豆包 (Doubao)

**端点：** `https://ark.cn-beijing.volces.com/api/v3`

**认证方式：**
```http
Authorization: Bearer xxx
```

**主要接口：** 兼容 OpenAI 格式

**价格模型：**
| 模型 | 输入价格 | 输出价格 | 单位 |
|------|----------|----------|------|
| doubao-pro-4k | ¥0.0008 | ¥0.002 | /1K tokens |
| doubao-pro-32k | ¥0.005 | ¥0.009 | /1K tokens |
| doubao-pro-128k | ¥0.03 | ¥0.05 | /1K tokens |

**特色：** 超低价格，性价比极高

---

### 2.11 SiliconFlow

**端点：** `https://api.siliconflow.cn/v1`

**认证方式：**
```http
Authorization: Bearer sk-xxx
```

**主要接口：** 兼容 OpenAI 格式

**特色：** 聚合平台，提供多种开源模型 API

**支持模型：** Qwen、DeepSeek、Yi、GLM、Llama 等

---

### 2.12 Minimax

**端点：** `https://api.minimax.chat/v1`

**认证方式：**
```http
Authorization: Bearer xxx
GroupId: xxx
```

**主要接口：** 兼容 OpenAI 格式

**特色：** 支持 MoE 模型，中文能力强

---

### 2.13 零一万物 (Yi)

**端点：** `https://api.lingyiwanwu.com/v1`

**认证方式：**
```http
Authorization: Bearer xxx
```

**主要接口：** 兼容 OpenAI 格式

**价格模型：**
| 模型 | 输入价格 | 输出价格 | 单位 |
|------|----------|----------|------|
| yi-lightning | ¥0.99 | ¥0.99 | /1M tokens |
| yi-large | ¥20.00 | ¥20.00 | /1M tokens |

---

## 三、Token 统计字段对照表

| 提供商 | 协议 | Prompt 字段 | Completion 字段 | Total 字段 |
|--------|------|-------------|-----------------|------------|
| OpenAI | OpenAI | `usage.prompt_tokens` | `usage.completion_tokens` | `usage.total_tokens` |
| Anthropic | Anthropic | `usage.input_tokens` | `usage.output_tokens` | 需计算 |
| DeepSeek | OpenAI | `usage.prompt_tokens` | `usage.completion_tokens` | `usage.total_tokens` |
| Qwen | OpenAI | `usage.prompt_tokens` | `usage.completion_tokens` | `usage.total_tokens` |
| Zhipu | OpenAI | `usage.prompt_tokens` | `usage.completion_tokens` | `usage.total_tokens` |
| Gemini | Gemini | `usageMetadata.promptTokenCount` | `usageMetadata.candidatesTokenCount` | `usageMetadata.totalTokenCount` |
| Groq | OpenAI | `usage.prompt_tokens` | `usage.completion_tokens` | `usage.total_tokens` |
| Moonshot | OpenAI | `usage.prompt_tokens` | `usage.completion_tokens` | `usage.total_tokens` |
| Qianfan | OpenAI | `usage.prompt_tokens` | `usage.completion_tokens` | `usage.total_tokens` |
| Doubao | OpenAI | `usage.prompt_tokens` | `usage.completion_tokens` | `usage.total_tokens` |

---

## 四、实现建议

### 4.1 统一 Token 提取

```rust
/// 从响应中提取 token 使用量
pub fn extract_usage(response: &Value, protocol: &ProtocolType) -> Option<TokenUsage> {
    match protocol {
        ProtocolType::OpenAiCompatible => {
            let usage = response.get("usage")?;
            Some(TokenUsage {
                prompt_tokens: usage.get("prompt_tokens")?.as_u64()?,
                completion_tokens: usage.get("completion_tokens")?.as_u64()?,
                total_tokens: usage.get("total_tokens")?.as_u64()?,
            })
        }
        ProtocolType::AnthropicCompatible => {
            let usage = response.get("usage")?;
            let prompt = usage.get("input_tokens")?.as_u64()?;
            let completion = usage.get("output_tokens")?.as_u64()?;
            Some(TokenUsage {
                prompt_tokens: prompt,
                completion_tokens: completion,
                total_tokens: prompt + completion,
            })
        }
    }
}
```

### 4.2 价格计算

```rust
/// 估算成本（美元）
pub fn estimate_cost(
    model: &str,
    prompt_tokens: u64,
    completion_tokens: u64,
) -> f64 {
    let pricing = PRICING_TABLE.get(model).unwrap_or(&DEFAULT_PRICING);
    
    let prompt_cost = (prompt_tokens as f64 / 1_000_000.0) * pricing.input_price_usd;
    let completion_cost = (completion_tokens as f64 / 1_000_000.0) * pricing.output_price_usd;
    
    prompt_cost + completion_cost
}
```

### 4.3 流式响应处理

流式响应（SSE）中，部分提供商在最后一个 chunk 中包含 usage：

- **OpenAI：** 需要设置 `stream_options: { include_usage: true }`
- **Anthropic：** 最后一个 message_start 事件包含 usage
- **其他 OpenAI 兼容：** 通常在最后 chunk 包含 usage

---

## 五、注意事项

1. **价格变动：** 以上价格为 2026 年 5 月参考价格，实际价格请以官方为准
2. **免费额度：** 部分提供商有免费额度或免费模型，适合测试
3. **地区限制：** 部分服务有地区限制，可能需要代理访问
4. **API 版本：** 各提供商 API 版本可能不同，注意兼容性
5. **流式计费：** 流式响应的 token 统计可能有延迟

---

## 六、参考资料

- OpenAI Platform: https://platform.openai.com
- Anthropic Console: https://console.anthropic.com
- DeepSeek Platform: https://platform.deepseek.com
- 阿里云 DashScope: https://dashscope.console.aliyun.com
- 智谱开放平台: https://open.bigmodel.cn
- Google AI Studio: https://aistudio.google.com
- Groq Console: https://console.groq.com
- 月之暗面: https://platform.moonshot.cn
- 百度智能云: https://console.bce.baidu.com/qianfan
- 火山引擎: https://console.volcengine.com/ark
