# Aidaguard

> **AI 时代的隐私守护者**
> 守护每一个 Token，保护每一次对话

Aidaguard 是一个开源桌面客户端，作为本地 API 网关代理，在敏感数据到达任何大模型 API 之前自动检测、替换和还原 —— 零配置，无需安装证书。

---

## 工作原理

```
你的 AI 客户端
      │
      │  HTTP  (localhost:7890)
      ▼
 Aidaguard 代理  ←── 检测并替换敏感数据
      │
      │  HTTPS (加密)
      ▼
 OpenAI / DeepSeek / 任何兼容 OpenAI 的 API
      │
      ▼
 Aidaguard 代理  ←── 还原响应中的占位符
      │
      ▼
你的 AI 客户端
```

## 功能特性

- 🛡️ **本地 API 网关** — 无需证书，只需将 API BaseURL 指向 `http://localhost:7890`
- 🔍 **智能检测** — 内置手机号、身份证、银行卡、邮箱、API Key 等规则
- 🔄 **无缝替换** — 发送前替换敏感数据为占位符，响应时自动还原
- 🌊 **流式支持** — 完整支持 SSE 流式响应，带滑动缓冲区还原
- 📁 **文件规则** — YAML 规则文件，支持热加载，可社区扩展
- 🔒 **加密存储** — 映射表本地存储，采用 AES-256-GCM 加密
- 🖥️ **跨平台** — 基于 Tauri，支持 macOS、Windows、Linux

## 支持的敏感数据类型

### 中国 (PIPL)
- 居民身份证、护照、军官证、港澳通行证
- 手机号、车牌号、银行卡号
- 统一社会信用代码

### 美国 (CCPA/CPRA)
- 社会安全号 (SSN)
- ITIN、EIN
- 各州驾照

### 欧盟 (GDPR)
- IBAN、SWIFT/BIC
- 各国增值税号 (VAT)
- 各国身份证

### 其他地区
- 🇸🇬 新加坡：NRIC、FIN、UEN
- 🇯🇵 日本：My Number（个人编号）
- 🇬🇧 英国：NINO、NHS 号码
- 🇰🇷 韩国：居民登记号

### 全球通用
- 信用卡（Visa、Mastercard、Amex 等），带 Luhn 校验
- 邮箱地址
- IP 地址（IPv4/IPv6）
- API Key（OpenAI、AWS、GitHub、Stripe 等）
- JWT Token、私钥

## 快速开始

> 前置条件：[Rust](https://rustup.rs) · [Node.js](https://nodejs.org) · [Tauri CLI](https://tauri.app)

```bash
git clone https://github.com/yourusername/aidaguard.git
cd aidaguard

# 构建并运行桌面应用
cd crates/aidaguard-tauri
cargo tauri dev
```

然后将你的 AI 客户端的 Base URL 设置为：
```
http://localhost:7890
```

## 项目结构

```
aidaguard/
├── crates/
│   ├── aidaguard-core/        # 核心类型、配置、检测器、替换器
│   ├── aidaguard-detector/    # 检测引擎（正则 + 校验 + NLP）
│   ├── aidaguard-proxy/       # HTTP 代理服务器 (Axum)
│   ├── aidaguard-upstream/    # LLM 提供商管理
│   ├── aidaguard-plugins/     # AI 工具适配器（Cursor、Claude Code 等）
│   ├── aidaguard-storage/     # 加密审计存储 (SQLite)
│   └── aidaguard-tauri/       # 桌面客户端 (Tauri 2.x + React)
├── rules/                     # 内置规则包（按地区组织）
└── docs/                      # 文档
```

## 文档

- [架构设计](docs/ARCHITECTURE.md) — 系统架构和模块说明
- [开发指南](docs/DEVELOPMENT.md) — 开发环境配置和工作流程
- [UI 设计](docs/UI_DESIGN.md) — 桌面客户端 UI 设计文档
- [规则优化计划](docs/RULES_OPTIMIZATION_PLAN.md) — 检测规则系统优化方案

## 支持的 AI 工具

Aidaguard 可自动配置以下工具的代理设置：

| 类别 | 工具 |
|------|------|
| 命令行 | Claude Code、Aider、Codex CLI、Gemini CLI、OpenCode |
| IDE | Cursor、Zed、Windsurf、JetBrains AI |
| VS Code 扩展 | Cline、Continue.dev、Codeium、Cody、Tabnine |

## 技术栈

**后端：**
- Rust、Tokio、Axum 0.7、Reqwest 0.12
- Candle（可选 NLP/NER，基于 BERT）

**前端：**
- Tauri 2.x、React 18、TypeScript
- shadcn/ui、Tailwind CSS、Zustand

## 许可证

MIT
