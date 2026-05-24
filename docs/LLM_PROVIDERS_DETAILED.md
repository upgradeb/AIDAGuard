# 大模型提供商详细信息

本文档详细记录国内外主要大模型提供商的基础信息、API接口、认证方式、服务类型和模型列表。

---

## 一、国内大模型提供商

### 1. DeepSeek (深度求索)

#### 基础信息
- **公司**: DeepSeek AI (深度求索)
- **官网**: https://www.deepseek.com
- **API 平台**: https://platform.deepseek.com
- **文档**: https://api-docs.deepseek.com

#### API 接口
| 参数 | 值 |
|------|-----|
| Base URL (OpenAI 格式) | `https://api.deepseek.com` |
| Base URL (Anthropic 格式) | `https://api.deepseek.com/anthropic` |
| API 格式 | OpenAI / Anthropic 兼容 |

#### 认证方式
- **API Key 认证**: `Authorization: Bearer ${DEEPSEEK_API_KEY}`
- **获取方式**: https://platform.deepseek.com/api_keys

#### 提供的服务
- 文本生成 (Chat Completion)
- 流式输出 (Streaming)
- 深度思考模式 (Thinking Mode)
- 工具调用 (Tool Calls)
- JSON 输出 (JSON Mode)
- 上下文缓存 (Context Caching)
- FIM 补全 (Fill-In-the-Middle)
- Chat Prefix Completion

#### 模型列表

| 模型名称 | 版本 | 上下文长度 | 最大输出 | 特性 |
|----------|------|-----------|----------|------|
| deepseek-v4-flash | DeepSeek-V4-Flash | 1M tokens | 384K | 思考/非思考模式，高性价比 |
| deepseek-v4-pro | DeepSeek-V4-Pro | 1M tokens | 384K | 思考/非思考模式，旗舰模型 |
| deepseek-chat | (即将废弃) | - | - | 2026/07/24 废弃，对应 v4-flash 非思考模式 |
| deepseek-reasoner | (即将废弃) | - | - | 2026/07/24 废弃，对应 v4-flash 思考模式 |

#### 定价 (每百万 tokens)

| 模型 | 输入 (缓存命中) | 输入 (缓存未命中) | 输出 |
|------|----------------|------------------|------|
| deepseek-v4-flash | $0.0028 | $0.14 | $0.28 |
| deepseek-v4-pro | $0.003625 (75% off) | $0.435 (75% off) | $0.87 (75% off) |

**注**: v4-pro 当前 75% 折扣，有效期至 2026/05/31 15:59 UTC

---

### 2. 阿里云百炼 (Qwen/通义千问)

#### 基础信息
- **公司**: 阿里云
- **产品名称**: 大模型服务平台百炼 (Model Studio)
- **官网**: https://www.aliyun.com/product/tongyi
- **文档**: https://help.aliyun.com/zh/model-studio/

#### API 接口
- **Base URL**: 通过控制台获取 Endpoint
- **API 格式**: OpenAI 兼容格式
- **支持地域**: cn-beijing, cn-shanghai, cn-shenzhen, cn-hangzhou

#### 认证方式
- **API Key 认证**: 通过阿里云控制台创建 API Key
- **获取方式**: https://bailian.console.aliyun.com/

#### 提供的服务
- 文本生成 (Qwen 系列)
- 视觉理解 (Qwen-VL 系列)
- 图片生成与编辑
- 视频生成与编辑
- 语音合成 (TTS)
- 语音识别 (ASR)
- 向量与重排序 (Embedding/Rerank)
- 模型微调
- 模型部署
- Agent 工具集成 (OpenClaw, Cursor, Claude Code 等)

#### 主要模型

| 模型系列 | 类型 | 说明 |
|----------|------|------|
| Qwen-Max | 文本生成 | 旗舰模型 |
| Qwen-Plus | 文本生成 | 高性价比 |
| Qwen-Turbo | 文本生成 | 快速响应 |
| Qwen-VL-Max | 视觉理解 | 多模态旗舰 |
| Qwen-VL-Plus | 视觉理解 | 多模态标准 |
| Qwen-Embedding | 向量模型 | 文本向量化 |
| Qwen-Audio | 音频理解 | 语音处理 |

#### 开发工具集成
支持多种 AI 编码工具：
- OpenClaw, Claude Code, Cursor, Cline
- Qwen Code, Cherry Studio, Chatbox
- Dify, Postman 等

---

### 3. 智谱 AI (GLM/ChatGLM)

#### 基础信息
- **公司**: 智谱 AI (Zhipu AI)
- **产品名称**: 智谱大模型开放平台
- **官网**: https://bigmodel.cn
- **文档**: https://open.bigmodel.cn

#### API 接口
| 参数 | 值 |
|------|-----|
| Base URL | `https://open.bigmodel.cn/api/paas/v4/` |
| API 格式 | OpenAI 兼容格式 |

#### 认证方式
- **API Key 认证**: `Authorization: Bearer ${ZHIPU_API_KEY}`
- **获取方式**: https://bigmodel.cn/usercenter/proj-mgmt/apikeys

#### 提供的服务
- 文本生成
- 视觉理解
- 图像生成
- 视频生成
- 音视频处理
- 向量模型
- 角色模型
- 深度思考 (Thinking)
- 工具调用
- 上下文缓存
- 结构化输出
- 联网搜索
- 模型微调
- 知识库服务

#### 模型列表与定价

**旗舰模型 (GLM-5 系列)**

| 模型 | 上下文 | 输入价格 | 输出价格 | 说明 |
|------|--------|----------|----------|------|
| GLM-5.1 | 32K+ | 6-8 元 | 24-28 元 | 面向长程任务，可独立工作 8 小时 |
| GLM-5-Turbo | 32K+ | 5-7 元 | 22-26 元 | 高性能旗舰 |
| GLM-5 | 32K+ | 4-6 元 | 18-22 元 | 标准旗舰 |
| GLM-4.7 | 32K+ | 2-4 元 | 8-16 元 | 高性价比 |
| GLM-4.5-Air | 32K+ | 0.8-1.2 元 | 2-8 元 | 轻量级模型 |
| GLM-4.7-FlashX | 200K | 0.5 元 | 3 元 | 高速低价 |
| GLM-4.7-Flash | 200K | 免费 | 免费 | 免费模型 |

**推理模型 (GLM-4 系列)**

| 模型 | 上下文 | 价格 | 说明 |
|------|--------|------|------|
| GLM-4-Plus | 128K | 5 元/M | 高智能旗舰 |
| GLM-4-Air | 128K | 0.5 元/M | 高性价比 |
| GLM-4-AirX | 8K | 10 元/M | 极速推理 |
| GLM-4-Long | 1M | 1 元/M | 超长输入 |
| GLM-4-Flash | 128K | 免费 | 免费模型 |
| GLM-Z1-Air | 128K | 0.5 元/M | 推理模型 |
| GLM-Z1-Flash | 128K | 免费 | 推理模型免费 |

**搜索工具服务**

| 工具 | 价格 | 说明 |
|------|------|------|
| Search-Std | 0.01 元/次 | 基础版，速度快 |
| Search-Pro | 0.03 元/次 | Pro 版，召回率高 |
| Search-Pro-Sogou | 0.05 元/次 | 搜狗搜索 |
| Search-Pro-Quark | 0.05 元/次 | 夸克搜索 |

---

### 4. 腾讯混元 (Hunyuan)

#### 基础信息
- **公司**: 腾讯云
- **产品名称**: 腾讯混元大模型
- **官网**: https://cloud.tencent.com/product/hunyuan
- **文档**: https://cloud.tencent.com/document/product/1729

#### API 接口
- **Base URL**: 通过腾讯云控制台获取
- **API 格式**: OpenAI 兼容格式
- **迁移说明**: 正在迁移至 TokenHub 平台

#### 认证方式
- 需要腾讯云实名认证 (企业或个人)
- API Key 通过控制台获取
- 控制台: https://hunyuan.cloud.tencent.com/

#### 提供的服务
- 文本生成
- 图像生成
- 多模态理解
- 向量模型
- 翻译服务

#### 模型列表与定价

| 模型 | 输入价格 | 输出价格 | 说明 |
|------|----------|----------|------|
| Tencent HY 2.0 Think | 3.975-5.3 元 | 15.9-21.2 元 | 深度思考模型 |
| Tencent HY 2.0 Instruct | 3.18-4.5 元 | 7.95-11.13 元 | 指令模型 |
| Hunyuan-T1 | 1 元 | 4 元 | 标准模型 |
| Hunyuan-TurboS | 0.8 元 | 2 元 | 快速模型 |
| Hunyuan-a13b | 0.5 元 | 2 元 | 轻量模型 |
| Hunyuan-lite | 免费 | 免费 | 免费模型 |
| Hunyuan-translation | 1.2 元 | 3.6 元 | 翻译模型 |
| Tencent HY Vision 1.5 | 3 元 | 9 元 | 视觉模型 |
| Hunyuan-embedding | - | - | 向量模型 (100万 tokens 免费) |

**免费额度**: 首次开通获得 100 万 tokens 免费额度，有效期 1 年

---

### 5. Moonshot AI (Kimi/月之暗面)

#### 基础信息
- **公司**: Moonshot AI (月之暗面)
- **产品名称**: Kimi API 开放平台
- **官网**: https://kimi.moonshot.cn
- **API 平台**: https://platform.kimi.com
- **文档**: https://platform.kimi.com/docs

#### API 接口
| 参数 | 值 |
|------|-----|
| Base URL | `https://api.moonshot.cn/v1` |
| API 格式 | OpenAI 兼容格式 |

#### 认证方式
- **API Key 认证**: `Authorization: Bearer ${MOONSHOT_API_KEY}`
- **获取方式**: https://platform.kimi.com/console/account

#### 提供的服务
- 文本生成
- 多模态理解 (视觉+文本)
- 长上下文处理 (256K)
- 文档解析
- 联网搜索
- 工具调用
- JSON Mode
- 批量推理

#### 模型列表与定价

| 模型 | 上下文 | 输入 (缓存命中) | 输入 (缓存未命中) | 输出 | 说明 |
|------|--------|----------------|------------------|------|------|
| kimi-k2.6 | 256K | ¥1.10 | ¥6.50 | ¥27.00 | 最新多模态模型，强代码能力 |
| kimi-k2.5 | 256K | - | - | - | 多模态模型 |
| kimi-k2 | 128K | - | - | - | MoE 模型，强 Agent 能力 |
| moonshot-v1-8k | 8K | - | - | - | 经典模型 |
| moonshot-v1-32k | 32K | - | - | - | 长上下文 |
| moonshot-v1-128k | 128K | - | - | - | 超长上下文 |

**计费说明**:
- Token 换算: 1 token ≈ 1.5-2 个汉字
- 文件接口限时免费

---

### 6. 火山引擎 (字节跳动/Doubao/豆包)

#### 基础信息
- **公司**: 字节跳动
- **产品名称**: 火山方舟大模型服务平台
- **官网**: https://www.volcengine.com/product/ark
- **文档**: https://www.volcengine.com/docs/82379

#### API 接口
| 参数 | 值 |
|------|-----|
| Base URL | `https://ark.cn-beijing.volces.com/api/v3` |
| API 格式 | OpenAI 兼容格式 / Responses API |

#### 认证方式
- **API Key 认证**: `Authorization: Bearer ${ARK_API_KEY}`
- **获取方式**: https://console.volcengine.com/ark/region:ark+cn-beijing/apikey

#### 提供的服务
- 文本生成
- 多模态理解 (图片/视频/文档)
- 视频生成 (Seedance)
- 图片生成 (Seedream)
- 3D 生成
- 深度思考
- 联网搜索
- 工具调用
- 上下文缓存
- 批量推理
- 模型微调
- MCP 工具集成

#### 主要模型

| 模型系列 | 类型 | 说明 |
|----------|------|------|
| Doubao Seed 2.0 | 文本生成 | 旗舰级 Agent 通用模型 |
| Doubao Seedance 2.0 | 视频生成 | 最强视频生成模型 |
| Doubao Seedream 5.0 | 图片生成 | 最强图片生成模型 |
| doubao-seed-2-0-lite | 文本生成 | 轻量级模型 |

#### SDK 支持
- Python SDK
- Go SDK
- Java SDK
- OpenAI SDK 兼容

---

### 7. 百度文心一言 (ERNIE)

#### 基础信息
- **公司**: 百度
- **产品名称**: 文心大模型
- **官网**: https://yiyan.baidu.com
- **API 平台**: https://console.bce.baidu.com/qianfan/

#### API 接口
- **Base URL**: `https://aip.baidubce.com/rpc/2.0/ai_custom/v1/wenxinworkshop/`
- **API 格式**: 百度自定义格式 (非 OpenAI 兼容)

#### 认证方式
- **Access Token**: 通过 API Key + Secret Key 获取
- **获取方式**: 百度智能云控制台

#### 提供的服务
- 文本生成
- 图像生成
- 向量模型
- 模型微调

#### 主要模型

| 模型 | 说明 |
|------|------|
| ERNIE-4.0 | 旗舰模型 |
| ERNIE-3.5 | 标准模型 |
| ERNIE-Speed | 快速模型 |
| ERNIE-Lite | 轻量模型 |
| ERNIE-Tiny | 轻量级免费 |

---

## 二、国际大模型提供商

### 1. OpenAI

#### 基础信息
- **公司**: OpenAI
- **官网**: https://openai.com
- **API 平台**: https://platform.openai.com

#### API 接口
| 参数 | 值 |
|------|-----|
| Base URL | `https://api.openai.com/v1` |
| API 格式 | OpenAI 原生格式 |

#### 认证方式
- **API Key 认证**: `Authorization: Bearer ${OPENAI_API_KEY}`
- **获取方式**: https://platform.openai.com/api-keys

#### 主要模型

| 模型 | 上下文 | 说明 |
|------|--------|------|
| gpt-4o | 128K | 多模态旗舰 |
| gpt-4o-mini | 128K | 轻量多模态 |
| gpt-4-turbo | 128K | 高性能 |
| gpt-3.5-turbo | 16K | 经济型 |
| o1 | 200K | 推理模型 |
| o1-mini | 128K | 轻量推理 |
| o3-mini | 200K | 最新推理模型 |

---

### 2. Anthropic (Claude)

#### 基础信息
- **公司**: Anthropic
- **官网**: https://www.anthropic.com
- **API 平台**: https://console.anthropic.com

#### API 接口
| 参数 | 值 |
|------|-----|
| Base URL | `https://api.anthropic.com/v1` |
| API 格式 | Anthropic 原生格式 |

#### 认证方式
- **API Key 认证**: `x-api-key: ${ANTHROPIC_API_KEY}`
- **获取方式**: https://console.anthropic.com/

#### 主要模型

| 模型 | 上下文 | 说明 |
|------|--------|------|
| claude-sonnet-4-20250514 | 200K | 最新旗舰 |
| claude-3-5-sonnet | 200K | 高性能 |
| claude-3-opus | 200K | 最强能力 |
| claude-3-haiku | 200K | 快速响应 |

---

### 3. Google (Gemini)

#### 基础信息
- **公司**: Google
- **产品名称**: Google AI Studio / Vertex AI
- **官网**: https://ai.google.dev

#### API 接口
| 参数 | 值 |
|------|-----|
| Base URL | `https://generativelanguage.googleapis.com/v1beta` |
| API 格式 | Google 原生格式 |

#### 认证方式
- **API Key 认证**: URL 参数 `?key=${GOOGLE_API_KEY}`
- **获取方式**: https://aistudio.google.com/apikey

#### 主要模型

| 模型 | 上下文 | 说明 |
|------|--------|------|
| gemini-2.5-pro | 1M | 最新旗舰 |
| gemini-2.5-flash | 1M | 快速模型 |
| gemini-1.5-pro | 2M | 长上下文 |
| gemini-1.5-flash | 1M | 轻量模型 |

---

### 4. Mistral AI

#### 基础信息
- **公司**: Mistral AI
- **官网**: https://mistral.ai
- **API 平台**: https://console.mistral.ai

#### API 接口
| 参数 | 值 |
|------|-----|
| Base URL | `https://api.mistral.ai/v1` |
| API 格式 | OpenAI 兼容格式 |

#### 主要模型

| 模型 | 说明 |
|------|------|
| mistral-large-2411 | 旗舰模型 |
| mistral-small-2409 | 轻量模型 |
| codestral-2405 | 代码模型 |
| pixtral-12b | 多模态模型 |

---

## 三、API 格式兼容性总结

| 提供商 | OpenAI 兼容 | Anthropic 兼容 | 原生格式 |
|--------|-------------|----------------|----------|
| DeepSeek | ✅ | ✅ | - |
| 阿里云百炼 | ✅ | - | - |
| 智谱 AI | ✅ | - | - |
| 腾讯混元 | ✅ | - | - |
| Moonshot | ✅ | - | - |
| 火山引擎 | ✅ | - | ✅ (Responses) |
| 百度文心 | - | - | ✅ |
| OpenAI | ✅ | - | - |
| Anthropic | - | ✅ | - |
| Google | - | - | ✅ |
| Mistral | ✅ | - | - |

---

## 四、认证方式总结

| 认证方式 | 使用提供商 |
|----------|-----------|
| Bearer Token (Header) | DeepSeek, 智谱, Moonshot, OpenAI, Mistral |
| x-api-key (Header) | Anthropic |
| URL Key 参数 | Google |
| Access Token | 百度 |
| 云平台 API Key | 阿里云, 腾讯云, 火山引擎 |

---

## 五、选择建议

### 国内场景
1. **高性价比**: DeepSeek v4-flash (输入 $0.14/M, 输出 $0.28/M)
2. **免费使用**: 智谱 GLM-4.7-Flash, 腾讯 Hunyuan-lite
3. **长上下文**: DeepSeek (1M), 智谱 GLM-4-Long (1M), Kimi (256K)
4. **Agent 能力**: 火山引擎 Doubao Seed 2.0, 智谱 GLM-5.1
5. **多模态**: Kimi K2.6, 火山引擎 Doubao 系列

### 国际场景
1. **综合能力**: OpenAI GPT-4o, Anthropic Claude Sonnet 4
2. **推理能力**: OpenAI o1/o3, Anthropic Claude
3. **长上下文**: Google Gemini 1.5 Pro (2M)
4. **代码能力**: OpenAI GPT-4o, Mistral Codestral

---

---

## 六、Coding Plan 与 Token Plan 套餐

### 1. 阿里云百炼 - Coding Plan

#### 套餐概述
阿里云百炼提供的 **Coding Plan** 是面向 AI 编程工具的专属套餐，支持多种主流编程工具 (Claude Code, OpenClaw, Cursor 等)，折算成本远低于常规 API 调用，通过固定月费模式有效防范欠费风险。

#### 套餐详情

| 套餐 | 价格 | 用量限制 | 支持模型 |
|------|------|----------|----------|
| **Pro 高级套餐** | ¥200/月 | 每 5 小时 6,000 次<br>每周 45,000 次<br>每月 90,000 次 | qwen3.6-plus, kimi-k2.5, glm-5, MiniMax-M2.5, qwen3.5-plus, qwen3-max, qwen3-coder-next, qwen3-coder-plus, glm-4.7 |
| ~~Lite 基础套餐~~ | ~~已停止新购~~ | - | - |

**注**: Lite 套餐已于 2026 年 3 月 20 日停止新购，4 月 13 日停止续费。

#### 专属接口

| 参数 | 值 |
|------|-----|
| **Base URL (OpenAI 兼容)** | `https://coding.dashscope.aliyuncs.com/v1` |
| **Base URL (Anthropic 兼容)** | `https://coding.dashscope.aliyuncs.com/apps/anthropic` |
| **API Key 格式** | 以 `sk-sp-` 开头 |

**重要说明**:
- Coding Plan 专属 API Key 与百炼按量计费的 API Key (`sk-xxxxx`) 不互通
- Coding Plan Base URL 与百炼通用 Base URL (`https://dashscope.aliyuncs.com/xxxxxx`) 不互通
- 请勿混用，否则会导致额外扣费

#### 额度恢复规则
- **每 5 小时额度**: 滚动恢复，每分钟自动释放 5 小时前的额度
- **每周额度**: 每周一 00:00:00 (UTC+08:00) 重置
- **每月额度**: 在下一个月订阅日的 00:00:00 (UTC+08:00) 重置

#### 消耗说明
- 单次提问按实际「模型调用次数」扣除额度
- 简单任务约消耗 5-10 次
- 复杂任务约消耗 10-30+ 次
- 实际消耗受任务难度、上下文及工具使用影响

#### 使用限制
⚠️ **严禁 API 调用**: 仅限在编程工具中使用，禁止以下场景:
- 自动化脚本
- 自定义应用程序后端
- 任何非交互式批量调用场景

违规使用可能导致订阅被暂停或 API Key 被封禁。

#### 购买页面
- 订阅地址: https://common-buy.aliyun.com/coding-plan
- 管理控制台: https://bailian.console.aliyun.com/cn-beijing/?tab=model#/efm/coding_plan

---

### 2. 阿里云百炼 - Token Plan (团队版)

#### 套餐概述
Token Plan (团队版) 是阿里云百炼提供的团队协作方案，支持多成员共享 Token 配额、统一计费管理。

#### 功能特性
- 团队成员管理
- 共享 Token 配额
- 统一账单与成本管理
- 权限控制

#### 文档链接
- 概述: https://help.aliyun.com/zh/model-studio/token-plan-overview
- 快速开始: https://help.aliyun.com/zh/model-studio/token-plan-quickstart
- 团队管理: https://help.aliyun.com/zh/model-studio/token-plan-team

---

### 3. 火山引擎 - Agent Plan / Coding Plan

#### 套餐概述
火山方舟提供 Agent Plan 和 Coding Plan 套餐，面向 AI 编程和 Agent 场景优化。

#### 功能特性
- 支持 AI 编程工具集成
- 联网搜索 MCP
- 视觉理解能力扩展
- 方舟文档 MCP

#### 支持的编程工具
- OpenClaw
- Claude Code
- Cursor
- Qwen Code
- Cherry Studio
- Cline
- Codex
- 更多工具...

#### 文档链接
- Agent Plan 文档: https://www.volcengine.com/docs/82379/2289964
- 模型价格: https://www.volcengine.com/docs/82379/1544106

---

### 4. 智谱 AI - Coding Plan

#### 套餐概述
智谱 AI 提供 Coding Plan 套餐，面向 AI 编程工具用户。

#### 获取方式
需要登录智谱开放平台后访问 Coding Plan 页面:
- 地址: https://bigmodel.cn/coding-plan/overview

**注**: 需要登录后才能查看详细套餐信息和价格。

#### 免费额度
智谱提供丰富的免费额度:
- GLM-4.7-Flash: 免费
- GLM-Z1-Flash: 免费
- 新用户: 2000 万 tokens 体验包

---

### 5. 腾讯混元 - 预付费资源包

#### 套餐概述
腾讯混元 **暂无专门的 Coding Plan 套餐**，但提供预付费资源包，性价比高于按量付费。

#### 预付费资源包规格

| 产品名 | 价格 | 备注 |
|--------|------|------|
| 混元大模型-预付费包-1万点 | ¥100 | - |
| 混元大模型-预付费包-10万点 | ¥950 | 相当于 ¥0.95/点 |
| 混元大模型-预付费包-100万点 | ¥9,000 | 相当于 ¥0.9/点 |
| 混元大模型-预付费包-1000万点 | ¥85,000 | 相当于 ¥0.85/点 |
| 混元大模型-预付费包-1亿点 | ¥800,000 | 相当于 ¥0.8/点 |

#### 抵扣系数
资源包点数用量 = 资源用量 × 抵扣系数 (点/千 tokens)

| 模型 | 输入系数 | 输出系数 |
|------|----------|----------|
| Hunyuan-T1 | 0.1 | 0.4 |
| Hunyuan-TurboS | 0.08 | 0.2 |
| Hunyuan-a13b | 0.05 | 0.2 |
| Hunyuan-lite | 免费 | 免费 |

#### 免费额度
- 首次开通获得 **100 万 tokens** 免费额度
- Hunyuan-lite: 永久免费
- Hunyuan-embedding: 100 万 tokens 免费

#### 平台迁移
⚠️ 混元大模型正在迁移至 **TokenHub** 平台:
- 新购需前往: https://console.cloud.tencent.com/tokenhub
- 原平台不再新增模型能力

---

### 6. DeepSeek

#### 套餐说明
DeepSeek **暂无专门的 Coding Plan 套餐**，仅提供按量付费模式。

#### 价格优势
DeepSeek 的按量付费价格本身已经非常低:

| 模型 | 输入 (缓存未命中) | 输出 | 折扣 |
|------|------------------|------|------|
| deepseek-v4-flash | $0.14/M | $0.28/M | - |
| deepseek-v4-pro | $0.435/M | $0.87/M | 75% off |

**缓存命中价格更低**:
- v4-flash: $0.0028/M (输入缓存命中)
- v4-pro: $0.003625/M (输入缓存命中)

#### 适合场景
- 对价格敏感的用户
- 需要大上下文 (1M tokens)
- 需要深度思考模式

---

## 七、Coding Plan 对比总结

| 提供商 | Coding Plan | Token Plan | 预付费包 | 免费模型 |
|--------|-------------|------------|----------|----------|
| **阿里云百炼** | ✅ Pro ¥200/月 | ✅ 团队版 | ✅ | - |
| **火山引擎** | ✅ Agent Plan | - | ✅ | - |
| **智谱 AI** | ✅ (需登录) | - | ✅ | GLM-4.7-Flash |
| **腾讯混元** | ❌ | ❌ | ✅ | Hunyuan-lite |
| **DeepSeek** | ❌ | ❌ | ❌ | - |
| **Moonshot** | ❌ | ❌ | ❌ | - |

### 推荐选择

1. **AI 编程工具用户**: 阿里云 Coding Plan (¥200/月, 90,000 次/月)
2. **免费体验**: 智谱 GLM-4.7-Flash 或 腾讯 Hunyuan-lite
3. **高性价比 API**: DeepSeek v4-flash ($0.14/M 输入)
4. **团队协作**: 阿里云 Token Plan 团队版

---

**文档更新时间**: 2026-05-17
**数据来源**: 各提供商官方文档 (通过浏览器自动化采集)
**Coding Plan 数据**: 阿里云百炼、火山引擎、智谱 AI、腾讯混元官方文档
