# 主流大模型提供商完整参考

**文档版本：** v1.0
**更新日期：** 2026-06-14
**用途：** AIDAGuard Phase 4.1 技术参考
**数据来源：** 各提供商官方文档

---

## 目录

- [一、提供商总览](#一提供商总览)
- [二、国内提供商](#二国内提供商)
  - [2.1 百度千帆 (Qianfan/文心一言)](#21-百度千帆-qianfan文心一言)
  - [2.2 DeepSeek (深度求索)](#22-deepseek-深度求索)
  - [2.3 阿里云百炼 (Qwen/通义千问)](#23-阿里云百炼-qwen通义千问)
  - [2.4 智谱 AI (GLM/ChatGLM)](#24-智谱-aiglmchatglm)
  - [2.5 Moonshot AI (Kimi/月之暗面)](#25-moonshot-aikimi月之暗面)
  - [2.6 腾讯混元 (Hunyuan)](#26-腾讯混元-hunyuan)
  - [2.7 火山引擎 (字节跳动/Doubao/豆包)](#27-火山引擎-字节跳动doubao豆包)
  - [2.8 SiliconFlow](#28-siliconflow)
  - [2.9 Minimax](#29-minimax)
  - [2.10 零一万物 (Yi)](#210-零一万物-yi)
- [三、国际提供商](#三国际提供商)
  - [3.1 OpenAI](#31-openai)
  - [3.2 Anthropic (Claude)](#32-anthropic-claude)
  - [3.3 Google (Gemini)](#33-google-gemini)
  - [3.4 Groq](#34-groq)
  - [3.5 Mistral AI](#35-mistral-ai)
- [四、Token 统计字段对照表](#四token-统计字段对照表)
- [五、API 格式兼容性总结](#五api-格式兼容性总结)
- [六、认证方式总结](#六认证方式总结)
- [七、实现参考](#七实现参考)
  - [7.1 统一 Token 提取](#71-统一-token-提取)
  - [7.2 价格计算](#72-价格计算)
  - [7.3 流式响应处理](#73-流式响应处理)
- [八、Coding Plan 与套餐对比](#八coding-plan-与套餐对比)
  - [8.1 阿里云百炼 - Coding Plan](#81-阿里云百炼--coding-plan)
  - [8.2 阿里云百炼 - Token Plan (团队版)](#82-阿里云百炼--token-plan-团队版)
  - [8.3 火山引擎 - Agent Plan / Coding Plan](#83-火山引擎--agent-plan--coding-plan)
  - [8.4 智谱 AI - Coding Plan](#84-智谱-ai--coding-plan)
  - [8.5 腾讯混元 - 预付费资源包](#85-腾讯混元--预付费资源包)
  - [8.6 DeepSeek](#86-deepseek)
  - [8.7 Coding Plan 对比总结](#87-coding-plan-对比总结)
- [九、选择建议](#九选择建议)
- [十、注意事项](#十注意事项)
- [十一、参考资料](#十一参考资料)

---

## 一、提供商总览

| 提供商 | 公司/组织 | API 协议 | 认证方式 | 计费方式 | 官方文档 |
|--------|-----------|----------|----------|----------|----------|
| Qianfan | 百度智能云 | OpenAI Compatible | Bearer Token | 按量计费 | https://cloud.baidu.com/doc/WENXINWORKSHOP |
| DeepSeek | 深度求索 | OpenAI / Anthropic Compatible | Bearer Token | 按量计费 | https://platform.deepseek.com/docs |
| Qwen | 阿里云 | OpenAI Compatible | Bearer Token | 按量计费 | https://help.aliyun.com/zh/dashscope |
| Zhipu | 智谱AI | OpenAI Compatible | Bearer Token | 按量计费 | https://open.bigmodel.cn/dev/api |
| Moonshot | 月之暗面 | OpenAI Compatible | Bearer Token | 按量计费 | https://platform.moonshot.cn/docs |
| Hunyuan | 腾讯云 | OpenAI Compatible | 云平台 API Key | 按量计费 | https://cloud.tencent.com/document/product/1729 |
| Doubao | 字节跳动 | OpenAI Compatible | Bearer Token | 按量计费 | https://www.volcengine.com/docs/82379 |
| SiliconFlow | SiliconFlow | OpenAI Compatible | Bearer Token | 按量计费 | https://docs.siliconflow.cn |
| Minimax | Minimax | OpenAI Compatible | Bearer Token | 按量计费 | https://www.minimaxi.com/document |
| Yi | 零一万物 | OpenAI Compatible | Bearer Token | 按量计费 | https://platform.lingyiwanwu.com/docs |
| OpenAI | OpenAI | OpenAI Compatible | Bearer Token | 按量计费 | https://platform.openai.com/docs |
| Anthropic | Anthropic | Anthropic Compatible | x-api-key Header | 按量计费 | https://docs.anthropic.com |
| Gemini | Google | Gemini SDK | API Key Param | 按量计费 | https://ai.google.dev/docs |
| Groq | Groq | OpenAI Compatible | Bearer Token | Free/Paid | https://console.groq.com/docs |
| Mistral | Mistral AI | OpenAI Compatible | Bearer Token | 按量计费 | https://console.mistral.ai |

---

## 二、国内提供商

### 2.1 百度千帆 (Qianfan/文心一言)

#### 基础信息
- **公司**: 百度
- **产品名称**: 文心大模型
- **官网**: https://yiyan.baidu.com
- **API 平台**: https://console.bce.baidu.com/qianfan/

#### API 接口
| 参数 | 值 |
|------|-----|
| Base URL (v2) | `https://qianfan.baidubce.com/v2` |
| Base URL (旧版) | `https://aip.baidubce.com/rpc/2.0/ai_custom/v1/wenxinworkshop/` |
| API 格式 | v2 兼容 OpenAI 格式；旧版为百度自定义格式 |

#### 认证方式
- **v2 (OpenAI 兼容)**: `Authorization: Bearer xxx`
- **旧版**: Access Token，通过 API Key + Secret Key 获取
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

#### 定价

| 模型 | 输入价格 | 输出价格 | 单位 |
|------|----------|----------|------|
| ERNIE-4.0-8K | ¥0.12 | ¥0.12 | /1K tokens |
| ERNIE-3.5-8K | ¥0.04 | ¥0.08 | /1K tokens |
| ERNIE-Speed | 免费 | 免费 | - |

---

### 2.2 DeepSeek (深度求索)

#### 基础信息
- **公司**: DeepSeek AI (深度求索)
- **官网**: https://www.deepseek.com
- **API 平台**: https://platform.deepseek.com
- **文档**: https://api-docs.deepseek.com

#### API 接口
| 参数 | 值 |
|------|-----|
| Base URL (OpenAI 格式) | `https://api.deepseek.com` 或 `https://api.deepseek.com/v1` |
| Base URL (Anthropic 格式) | `https://api.deepseek.com/anthropic` |
| API 格式 | OpenAI / Anthropic 兼容 |

#### 认证方式
- **API Key 认证**: `Authorization: Bearer ${DEEPSEEK_API_KEY}`
- **获取方式**: https://platform.deepseek.com/api_keys

#### 主要接口
完全兼容 OpenAI 格式

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

**旧版价格参考**:

| 模型 | 输入价格 | 输出价格 | 单位 |
|------|----------|----------|------|
| deepseek-chat | $0.27 | $1.10 | /1M tokens |
| deepseek-coder | $0.27 | $1.10 | /1M tokens |
| deepseek-reasoner | $0.55 | $2.19 | /1M tokens |

**特色**: 支持深度思考模式（deepseek-reasoner），性价比高，1M tokens 长上下文

---

### 2.3 阿里云百炼 (Qwen/通义千问)

#### 基础信息
- **公司**: 阿里云
- **产品名称**: 大模型服务平台百炼 (Model Studio)
- **官网**: https://www.aliyun.com/product/tongyi
- **文档**: https://help.aliyun.com/zh/model-studio/

#### API 接口
| 参数 | 值 |
|------|-----|
| Base URL | `https://dashscope.aliyuncs.com/compatible-mode/v1` |
| API 格式 | OpenAI 兼容格式 |
| 支持地域 | cn-beijing, cn-shanghai, cn-shenzhen, cn-hangzhou |

#### 认证方式
- **API Key 认证**: `Authorization: Bearer sk-xxx`
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

#### 定价

| 模型 | 输入价格 | 输出价格 | 单位 |
|------|----------|----------|------|
| qwen-max | ¥2.00 | ¥8.00 | /1M tokens |
| qwen-plus | ¥0.40 | ¥2.00 | /1M tokens |
| qwen-turbo | ¥0.30 | ¥0.60 | /1M tokens |
| qwen-long | ¥0.50 | ¥2.00 | /1M tokens |

**特色**: 长上下文模型（qwen-long）支持 1M tokens

#### 开发工具集成
支持多种 AI 编码工具：
- OpenClaw, Claude Code, Cursor, Cline
- Qwen Code, Cherry Studio, Chatbox
- Dify, Postman 等

---

### 2.4 智谱 AI (GLM/ChatGLM)

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
- **API Key 认证**: `Authorization: Bearer xxx.xxx`
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

**旧版价格参考**:

| 模型 | 输入价格 | 输出价格 | 单位 |
|------|----------|----------|------|
| glm-4 | ¥0.10 | ¥0.10 | /1K tokens |
| glm-4-air | ¥0.001 | ¥0.001 | /1K tokens |
| glm-4-flash | 免费 | 免费 | - |

**搜索工具服务**

| 工具 | 价格 | 说明 |
|------|------|------|
| Search-Std | 0.01 元/次 | 基础版，速度快 |
| Search-Pro | 0.03 元/次 | Pro 版，召回率高 |
| Search-Pro-Sogou | 0.05 元/次 | 搜狗搜索 |
| Search-Pro-Quark | 0.05 元/次 | 夸克搜索 |

**特色**: GLM-4-flash 免费使用，适合测试

---

### 2.5 Moonshot AI (Kimi/月之暗面)

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

**旧版价格参考**:

| 模型 | 输入价格 | 输出价格 | 单位 |
|------|----------|----------|------|
| moonshot-v1-8k | ¥0.012 | ¥0.012 | /1K tokens |
| moonshot-v1-32k | ¥0.024 | ¥0.024 | /1K tokens |
| moonshot-v1-128k | ¥0.06 | ¥0.06 | /1K tokens |

**计费说明**:
- Token 换算: 1 token 约等于 1.5-2 个汉字
- 文件接口限时免费

**特色**: 支持长上下文（128K/256K）

---

### 2.6 腾讯混元 (Hunyuan)

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

**平台迁移**: 混元大模型正在迁移至 TokenHub 平台，新购需前往 https://console.cloud.tencent.com/tokenhub

---

### 2.7 火山引擎 (字节跳动/Doubao/豆包)

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

#### 定价

| 模型 | 输入价格 | 输出价格 | 单位 |
|------|----------|----------|------|
| doubao-pro-4k | ¥0.0008 | ¥0.002 | /1K tokens |
| doubao-pro-32k | ¥0.005 | ¥0.009 | /1K tokens |
| doubao-pro-128k | ¥0.03 | ¥0.05 | /1K tokens |

**特色**: 超低价格，性价比极高

#### SDK 支持
- Python SDK
- Go SDK
- Java SDK
- OpenAI SDK 兼容

---

### 2.8 SiliconFlow

#### API 接口
- **Base URL**: `https://api.siliconflow.cn/v1`
- **API 格式**: OpenAI 兼容格式

#### 认证方式
```http
Authorization: Bearer sk-xxx
```

#### 主要接口
兼容 OpenAI 格式

**特色**: 聚合平台，提供多种开源模型 API

**支持模型**: Qwen、DeepSeek、Yi、GLM、Llama 等

---

### 2.9 Minimax

#### API 接口
- **Base URL**: `https://api.minimax.chat/v1`
- **API 格式**: OpenAI 兼容格式

#### 认证方式
```http
Authorization: Bearer xxx
GroupId: xxx
```

#### 主要接口
兼容 OpenAI 格式

**特色**: 支持 MoE 模型，中文能力强

---

### 2.10 零一万物 (Yi)

#### 基础信息
- **公司**: 零一万物
- **API 平台**: https://platform.lingyiwanwu.com
- **文档**: https://platform.lingyiwanwu.com/docs

#### API 接口
- **Base URL**: `https://api.lingyiwanwu.com/v1`
- **API 格式**: OpenAI 兼容格式

#### 认证方式
```http
Authorization: Bearer xxx
```

#### 主要接口
兼容 OpenAI 格式

#### 定价

| 模型 | 输入价格 | 输出价格 | 单位 |
|------|----------|----------|------|
| yi-lightning | ¥0.99 | ¥0.99 | /1M tokens |
| yi-large | ¥20.00 | ¥20.00 | /1M tokens |

---

## 三、国际提供商

### 3.1 OpenAI

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
```http
Authorization: Bearer ${OPENAI_API_KEY}
```
- **获取方式**: https://platform.openai.com/api-keys

#### 主要接口
```
POST /chat/completions    # 对话补全
POST /completions         # 文本补全（旧版）
POST /embeddings          # 向量化
POST /models              # 模型列表
```

#### 请求示例
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

#### 响应格式
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

#### 定价

| 模型 | 输入价格 | 输出价格 | 单位 |
|------|----------|----------|------|
| gpt-4o | $2.50 | $10.00 | /1M tokens |
| gpt-4o-mini | $0.15 | $0.60 | /1M tokens |
| gpt-4-turbo | $10.00 | $30.00 | /1M tokens |
| gpt-3.5-turbo | $0.50 | $1.50 | /1M tokens |

**Token 统计**: 响应中 `usage` 字段包含完整统计

---

### 3.2 Anthropic (Claude)

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
```http
x-api-key: ${ANTHROPIC_API_KEY}
anthropic-version: 2023-06-01
```
- **获取方式**: https://console.anthropic.com/

#### 主要接口
```
POST /messages           # 对话补全
POST /messages/count_tokens  # Token 计数
```

#### 请求示例
```json
{
  "model": "claude-3-5-sonnet-20241022",
  "max_tokens": 1024,
  "messages": [
    {"role": "user", "content": "Hello"}
  ]
}
```

#### 响应格式
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

#### 主要模型

| 模型 | 上下文 | 说明 |
|------|--------|------|
| claude-sonnet-4-20250514 | 200K | 最新旗舰 |
| claude-3-5-sonnet | 200K | 高性能 |
| claude-3-opus | 200K | 最强能力 |
| claude-3-haiku | 200K | 快速响应 |

#### 定价

| 模型 | 输入价格 | 输出价格 | 单位 |
|------|----------|----------|------|
| claude-3.5-sonnet | $3.00 | $15.00 | /1M tokens |
| claude-3.5-haiku | $0.80 | $4.00 | /1M tokens |
| claude-3-opus | $15.00 | $75.00 | /1M tokens |

**Token 统计**: 响应中 `usage` 字段（注意：字段名不同于 OpenAI）

---

### 3.3 Google (Gemini)

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
```
URL 参数: ?key=${GOOGLE_API_KEY}
```
- **获取方式**: https://aistudio.google.com/apikey

#### 主要接口
```
POST /models/{model}:generateContent
POST /models/{model}:streamGenerateContent
GET /models                    # 模型列表
```

#### 请求示例
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

#### 响应格式
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

#### 主要模型

| 模型 | 上下文 | 说明 |
|------|--------|------|
| gemini-2.5-pro | 1M | 最新旗舰 |
| gemini-2.5-flash | 1M | 快速模型 |
| gemini-1.5-pro | 2M | 长上下文 |
| gemini-1.5-flash | 1M | 轻量模型 |

#### 定价

| 模型 | 输入价格 | 输出价格 | 单位 |
|------|----------|----------|------|
| gemini-2.0-flash | $0.10 | $0.40 | /1M tokens |
| gemini-1.5-pro | $1.25 | $5.00 | /1M tokens |
| gemini-1.5-flash | $0.075 | $0.30 | /1M tokens |

**Token 统计**: 响应中 `usageMetadata` 字段（注意字段名不同）

---

### 3.4 Groq

#### API 接口
- **Base URL**: `https://api.groq.com/openai/v1`
- **API 格式**: 完全兼容 OpenAI 格式

#### 认证方式
```http
Authorization: Bearer gsk_xxx
```

#### 主要接口
完全兼容 OpenAI 格式

#### 定价

| 模型 | 输入价格 | 输出价格 | 单位 |
|------|----------|----------|------|
| llama-3.3-70b-versatile | 免费 | 免费 | Free tier |
| llama-3.1-8b-instant | 免费 | 免费 | Free tier |
| mixtral-8x7b-32768 | 免费 | 免费 | Free tier |

**特色**: 超快推理速度，免费额度充足

---

### 3.5 Mistral AI

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

## 四、Token 统计字段对照表

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

## 五、API 格式兼容性总结

| 提供商 | OpenAI 兼容 | Anthropic 兼容 | 原生格式 |
|--------|-------------|----------------|----------|
| DeepSeek | ✅ | ✅ | - |
| 阿里云百炼 | ✅ | - | - |
| 智谱 AI | ✅ | - | - |
| 腾讯混元 | ✅ | - | - |
| Moonshot | ✅ | - | - |
| 火山引擎 | ✅ | - | ✅ (Responses) |
| 百度文心 | ✅ (v2) | - | ✅ (旧版) |
| SiliconFlow | ✅ | - | - |
| Minimax | ✅ | - | - |
| 零一万物 | ✅ | - | - |
| OpenAI | ✅ | - | - |
| Anthropic | - | ✅ | - |
| Google | - | - | ✅ |
| Groq | ✅ | - | - |
| Mistral | ✅ | - | - |

---

## 六、认证方式总结

| 认证方式 | 使用提供商 |
|----------|-----------|
| Bearer Token (Header) | DeepSeek, 智谱, Moonshot, Qwen, OpenAI, Mistral, Qianfan, Doubao, SiliconFlow, Minimax, Yi, Groq |
| x-api-key (Header) | Anthropic |
| URL Key 参数 | Google |
| Access Token | 百度 (旧版) |
| 云平台 API Key | 阿里云, 腾讯云, 火山引擎 |

---

## 七、实现参考

### 7.1 统一 Token 提取

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

### 7.2 价格计算

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

### 7.3 流式响应处理

流式响应（SSE）中，部分提供商在最后一个 chunk 中包含 usage：

- **OpenAI**: 需要设置 `stream_options: { include_usage: true }`
- **Anthropic**: 最后一个 message_start 事件包含 usage
- **其他 OpenAI 兼容**: 通常在最后 chunk 包含 usage

---

## 八、Coding Plan 与套餐对比

### 8.1 阿里云百炼 - Coding Plan

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
**严禁 API 调用**: 仅限在编程工具中使用，禁止以下场景:
- 自动化脚本
- 自定义应用程序后端
- 任何非交互式批量调用场景

违规使用可能导致订阅被暂停或 API Key 被封禁。

#### 购买页面
- 订阅地址: https://common-buy.aliyun.com/coding-plan
- 管理控制台: https://bailian.console.aliyun.com/cn-beijing/?tab=model#/efm/coding_plan

---

### 8.2 阿里云百炼 - Token Plan (团队版)

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

### 8.3 火山引擎 - Agent Plan / Coding Plan

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

### 8.4 智谱 AI - Coding Plan

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

### 8.5 腾讯混元 - 预付费资源包

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
资源包点数用量 = 资源用量 x 抵扣系数 (点/千 tokens)

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

---

### 8.6 DeepSeek

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

### 8.7 Coding Plan 对比总结

| 提供商 | Coding Plan | Token Plan | 预付费包 | 免费模型 |
|--------|-------------|------------|----------|----------|
| **阿里云百炼** | ✅ Pro ¥200/月 | ✅ 团队版 | ✅ | - |
| **火山引擎** | ✅ Agent Plan | - | ✅ | - |
| **智谱 AI** | ✅ (需登录) | - | ✅ | GLM-4.7-Flash |
| **腾讯混元** | ❌ | ❌ | ✅ | Hunyuan-lite |
| **DeepSeek** | ❌ | ❌ | ❌ | - |
| **Moonshot** | ❌ | ❌ | ❌ | - |

#### 推荐选择

1. **AI 编程工具用户**: 阿里云 Coding Plan (¥200/月, 90,000 次/月)
2. **免费体验**: 智谱 GLM-4.7-Flash 或 腾讯 Hunyuan-lite
3. **高性价比 API**: DeepSeek v4-flash ($0.14/M 输入)
4. **团队协作**: 阿里云 Token Plan 团队版

---

## 九、选择建议

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
5. **快速推理**: Groq (免费，超低延迟)

---

## 十、注意事项

1. **价格变动**: 以上价格为 2026 年 5 月参考价格，实际价格请以官方为准
2. **免费额度**: 部分提供商有免费额度或免费模型，适合测试
3. **地区限制**: 部分服务有地区限制，可能需要代理访问
4. **API 版本**: 各提供商 API 版本可能不同，注意兼容性
5. **流式计费**: 流式响应的 token 统计可能有延迟
6. **Coding Plan 限制**: Coding Plan 套餐仅限编程工具内使用，禁止自动化脚本和批量调用
7. **模型废弃**: DeepSeek 的 deepseek-chat 和 deepseek-reasoner 将于 2026/07/24 废弃
8. **平台迁移**: 腾讯混元正在迁移至 TokenHub 平台

---

## 十一、参考资料

### 国内提供商
- 百度智能云: https://console.bce.baidu.com/qianfan
- DeepSeek Platform: https://platform.deepseek.com
- 阿里云 DashScope: https://dashscope.console.aliyun.com
- 智谱开放平台: https://open.bigmodel.cn
- 月之暗面: https://platform.moonshot.cn
- 腾讯混元: https://hunyuan.cloud.tencent.com
- 火山引擎: https://console.volcengine.com/ark

### 国际提供商
- OpenAI Platform: https://platform.openai.com
- Anthropic Console: https://console.anthropic.com
- Google AI Studio: https://aistudio.google.com
- Groq Console: https://console.groq.com
- Mistral Console: https://console.mistral.ai

### Coding Plan 相关
- 阿里云 Coding Plan 订阅: https://common-buy.aliyun.com/coding-plan
- 阿里云百炼控制台: https://bailian.console.aliyun.com
- 智谱 Coding Plan: https://bigmodel.cn/coding-plan/overview
- 火山引擎 Agent Plan: https://www.volcengine.com/docs/82379/2289964
