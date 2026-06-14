# EchoBird 工具配置系统参考

**来源：** [github.com/edison7009/EchoBird](https://github.com/edison7009/EchoBird)  
**文档版本：** v1.0  
**更新日期：** 2026-05-26  
**用途：** AIDAGuard 工具适配器架构参考

---

## 一、EchoBird 项目概述

EchoBird 是一个跨平台 AI 工具部署和管理桌面应用（Tauri + Rust），核心功能之一是 **App Manager** — 一键管理和配置各类 AI/Agent 工具。

与 AIDAGuard 关注"代理拦截敏感数据"不同，EchoBird 关注的是"统一模型配置，一处配置处处使用"。两者的工具适配器设计可以互相借鉴。

---

## 二、工具分类

EchoBird 将工具分为 4 大类：

### 2.1 CLI Tools（命令行工具）

| 工具 ID | 显示名称 | 配置格式 | API 协议 |
|---------|----------|----------|----------|
| `claudecode` | Claude Code | JSON (`~/.claude/settings.json`) | Anthropic |
| `codex` | Codex CLI | TOML (`~/.codex/config.toml`) | OpenAI |
| `aider` | Aider | YAML (`~/.aider.conf.yml`) | OpenAI, Anthropic |
| `openclaw` | OpenClaw | JSON (`~/.openclaw/openclaw.json`) | 通用 |
| `hermes` | Hermes Agent | JSON (`~/.echobird/hermes.json`) | 自定义 |
| `opencode` | Open Code | JSONC (`~/.config/opencode/opencode.jsonc`) | 通用 |
| `qwencode` | Qwen Code | JSON (`~/.qwen/settings.json`) | 自定义 |
| `coffeecli` | Coffee CLI | JSON (`~/.coffee-cli/config.json`) | 通用 |
| `grok` | Grok | TOML (`~/.grok/config.toml`) | 自定义 |
| `openfang` | Open Fang | TOML (`~/.openfang/config.toml`) | 通用 |
| `pi` | Pi | JSON (`~/.pi/agent/settings.json`) | 通用 |
| `picoclaw` | Pico Claw | JSON (`~/.picoclaw/config.json`) | 通用 |
| `nanobot` | Nano Bot | JSON (`~/.nanobot/config.json`) | 通用 |
| `zeroclaw` | Zero Claw | TOML (`~/.zeroclaw/config.toml`) | 通用 |

### 2.2 Desktop Apps（桌面应用）

| 工具 ID | 显示名称 | 配置格式 |
|---------|----------|----------|
| `claudedesktop` | Claude Desktop | JSON (`~/.claude/settings.json`) |
| `codexdesktop` | Codex Desktop | TOML (`~/.codex/config.toml`) |
| `geminidesktop` | Gemini Desktop | JSON (`~/.gemini/settings.json`) |

### 2.3 IDE / Editor（编辑器）

| 工具 ID | 显示名称 | 配置格式 |
|---------|----------|----------|
| `cursor` | Cursor | JSON (`~/.cursor/settings.json`) |
| `windsurf` | Windsurf | JSON (`~/.codeium/windsurf/settings.json`) |
| `vscode` | VS Code | JSON (`~/.vscode/settings.json`) |
| `trae` | Trae IDE | JSON (`~/.trae/settings.json`) |
| `traecn` | Trae CN | JSON (`~/.trae-cn/settings.json`) |

### 2.4 Embedded Tools（嵌入式工具）

| 工具 ID | 显示名称 | 运行时 |
|---------|----------|--------|
| `reversi` | AI Reversi | Webview 内嵌 |
| `ai-trader` | AI Trader | Webview 内嵌 |
| `fingpt` | FinGPT | Webview 内嵌 |
| `tradingagents` | Trading Agents | Webview 内嵌 |
| `translator` | Translator | Webview 内嵌 |

---

## 三、核心设计：声明式配置

EchoBird 的最大设计亮点是**声明式配置**——每个工具通过两个 JSON 文件描述，无需编写任何 Rust 代码。

### 3.1 文件结构

```
tools/
├── <tool-id>/
│   ├── config.json    # 配置读写映射
│   └── paths.json     # 安装路径和检测信息
```

### 3.2 config.json — 配置读写映射

定义如何**读取**和**写入**工具的模型配置（model、baseUrl、apiKey），使用 JSON 路径表示法。

**完整结构：**

```json
{
  "docs": "https://docs.example.com",
  "configFile": "~/.tool/config.json",
  "format": "json|toml|yaml|env",
  "read": {
    "model": ["path.to.model"],
    "baseUrl": ["path.to.baseUrl"],
    "apiKey": ["path.to.apiKey"]
  },
  "write": {
    "path.to.model": "model",
    "path.to.baseUrl": "baseUrl",
    "path.to.apiKey": "apiKey"
  }
}
```

**字段说明：**

| 字段 | 说明 |
|------|------|
| `docs` | 官方文档 URL |
| `configFile` | 工具配置文件路径（支持 `~` 展开） |
| `format` | 配置文件格式：`json`、`toml`、`yaml`、`env` |
| `read` | 读取映射：`{ 字段名: [候选 JSON 路径列表] }` |
| `write` | 写入映射：`{ JSON路径: 字段名或固定值 }` |

**read 映射**：按顺序尝试每个 JSON 路径，返回第一个找到的值。路径使用 `.` 分隔嵌套层级（如 `env.ANTHROPIC_BASE_URL`）。

**write 映射**：键为目标 JSON 路径，值为以下三种之一：
- `"model"` / `"baseUrl"` / `"apiKey"` — 写入对应字段的动态值
- `""`（空字符串）— 删除该键（用于清理不需要的字段）
- 其他字符串 — 写入固定值（如 `"3000000"` 表示固定超时值）

**实际示例 — Claude Code：**

```json
{
  "docs": "https://docs.anthropic.com/en/docs/claude-code/settings",
  "configFile": "~/.claude/settings.json",
  "format": "json",
  "read": {
    "model": ["env.ANTHROPIC_MODEL"],
    "baseUrl": ["env.ANTHROPIC_BASE_URL"],
    "apiKey": ["env.ANTHROPIC_AUTH_TOKEN", "env.ANTHROPIC_API_KEY"]
  },
  "write": {
    "env.ANTHROPIC_MODEL": "model",
    "env.ANTHROPIC_SMALL_FAST_MODEL": "model",
    "env.ANTHROPIC_DEFAULT_SONNET_MODEL": "model",
    "env.ANTHROPIC_DEFAULT_OPUS_MODEL": "model",
    "env.ANTHROPIC_DEFAULT_HAIKU_MODEL": "model",
    "env.ANTHROPIC_BASE_URL": "baseUrl",
    "env.ANTHROPIC_AUTH_TOKEN": "apiKey",
    "env.ANTHROPIC_API_KEY": "",
    "env.API_TIMEOUT_MS": "3000000",
    "env.CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1"
  }
}
```

> **关键设计**：当写入 `model` 字段时，同时更新 `ANTHROPIC_MODEL`、`ANTHROPIC_DEFAULT_SONNET_MODEL`、`ANTHROPIC_DEFAULT_OPUS_MODEL`、`ANTHROPIC_DEFAULT_HAIKU_MODEL` 四个键，确保所有模式使用统一模型。

**实际示例 — Aider：**

```json
{
  "configFile": "~/.aider.conf.yml",
  "format": "yaml",
  "custom": true
}
```

> `custom: true` 表示该工具使用自定义读写逻辑（需在 Rust 代码中单独实现），不通过声明式配置处理。

### 3.3 paths.json — 安装路径与检测

定义工具的安装路径和检测信息。

**完整结构：**

```json
{
  "name": "工具显示名称",
  "names": { "zh-Hans": "中文名" },
  "category": "CLI Code | Desktop | IDE | Embedded",
  "apiProtocol": ["openai", "anthropic"],
  "docs": "https://docs.example.com",
  "command": "cli命令名",
  "startCommand": "启动命令",
  "configDir": "~/.config-dir",
  "configFile": "~/.config-file",
  "requireConfigFile": false,
  "paths": {
    "win32": ["路径1", "路径2"],
    "darwin": ["路径1", "路径2"],
    "linux": ["路径1", "路径2"]
  }
}
```

**字段说明：**

| 字段 | 说明 |
|------|------|
| `name` | 显示名称 |
| `category` | 分类：`CLI Code`、`Desktop`、`IDE`、`Embedded` |
| `apiProtocol` | 支持的 API 协议列表 |
| `command` | CLI 命令名（用于 `which` 检测） |
| `configDir` | 配置目录路径 |
| `configFile` | 配置文件路径 |
| `paths` | 按平台的候选二进制路径列表 |

**路径变量支持：** `~` → 用户主目录，`%APPDATA%` → Windows AppData，`%LOCALAPPDATA%` → Windows LocalAppData

**实际示例 — Aider：**

```json
{
  "name": "Aider",
  "category": "CLI Code",
  "apiProtocol": ["openai", "anthropic"],
  "docs": "https://aider.chat",
  "command": "aider",
  "startCommand": "aider",
  "configDir": "~",
  "configFile": "~/.aider.conf.yml",
  "requireConfigFile": false,
  "paths": {
    "win32": [
      "%USERPROFILE%/.local/bin/aider.exe",
      "%APPDATA%/Python/Scripts/aider.exe"
    ],
    "darwin": [
      "/usr/local/bin/aider",
      "/opt/homebrew/bin/aider",
      "~/.local/bin/aider"
    ],
    "linux": [
      "/usr/local/bin/aider",
      "/usr/bin/aider",
      "~/.local/bin/aider"
    ]
  }
}
```

---

## 四、工具替换（配置应用）机制

### 4.1 整体流程

```
用户选择模型 → 读取 config.json → 解析 write 映射 → 读取目标配置文件
    → 按路径写入新值 → 序列化 → 写回配置文件 → 工具重启/生效
```

### 4.2 配置读取流程

```
1. 确定配置文件路径（configFile）
2. 根据 format 解析文件（JSON/TOML/YAML/ENV）
3. 按 read 映射提取 model / baseUrl / apiKey
4. 返回当前配置状态
```

### 4.3 配置写入流程

```
1. 读取目标配置文件（如不存在则创建）
2. 根据 format 解析
3. 遍历 write 映射：
   - 值为 "model"/"baseUrl"/"apiKey" → 写入用户选择的对应值
   - 值为 ""（空）→ 删除该键
   - 值为其他 → 写入固定值
4. 确保嵌套路径存在（如 env.ANTHROPIC_BASE_URL 需要先创建 env 对象）
5. 序列化并写回文件
```

### 4.4 特殊处理

- **Codex CLI**：内置协议转换代理（Responses API → Chat Completions API），运行时自动启动代理进程
- **API Key 加密**：写入配置文件前用 AES 加密，密钥存储在系统钥匙串
- **配置热重载**：部分工具支持配置变更后自动重载

---

## 五、与 AIDAGuard 的对比

| 维度 | EchoBird | AIDAGuard（当前） |
|------|----------|-------------------|
| **配置方式** | 声明式（JSON 文件） | 命令式（Rust trait 实现） |
| **新增工具** | 添加 2 个 JSON 文件 | 编写 ~100 行 Rust 代码 |
| **配置格式** | JSON/TOML/YAML/ENV 统一处理 | 每个适配器独立实现 |
| **检测方式** | 多路径候选 + which 命令 | 固定目录检查 |
| **读写路径** | JSON 路径（如 `env.ANTHROPIC_BASE_URL`） | 硬编码字段名 |
| **工具数量** | 25 个（4 大类别） | 17 个 |
| **API 协议** | 明确定义（anthropic/openai） | 隐式 |
| **备份还原** | 无（依赖配置管理系统） | 文件级备份/还原 |

### 关键差异

1. **EchoBird 的声明式方法**更适合工具数量快速增长——只需 JSON 配置，无需写 Rust 代码
2. **AIDAGuard 的命令式方法**更灵活——可以处理复杂的边缘情况（如 Cline 的双配置源）
3. **EchoBird 用 JSON 路径表示法**处理嵌套配置（如 `env.ANTHROPIC_BASE_URL`），AIDAGuard 硬编码每个字段

### AIDAGuard 可借鉴的设计

1. **声明式 config.json 格式**：为每个适配器添加标准化的读写路径声明，减少样板代码
2. **多路径候选检测**：支持按平台检测多个候选路径（而非单一固定路径）
3. **`custom` 标记**：区分"纯声明式"和"需自定义逻辑"的适配器
4. **write 映射中的固定值**：允许 configure 时同时设置辅助值（如超时、SSL 配置）

---

## 六、完整工具目录

### CLI Tools

| ID | 名称 | 配置文件 | 格式 | API 协议 | 自定义 |
|----|------|----------|------|----------|--------|
| `claudecode` | Claude Code | `~/.claude/settings.json` | JSON | Anthropic | 否 |
| `codex` | Codex CLI | `~/.codex/config.toml` | TOML | OpenAI | 是 |
| `aider` | Aider | `~/.aider.conf.yml` | YAML | OpenAI, Anthropic | 是 |
| `openclaw` | OpenClaw | `~/.openclaw/openclaw.json` | JSON | 通用 | 是 |
| `hermes` | Hermes Agent | `~/.echobird/hermes.json` | JSON | 自定义 | 是 |
| `opencode` | Open Code | `~/.config/opencode/opencode.jsonc` | JSONC | 通用 | 是 |
| `qwencode` | Qwen Code | `~/.qwen/settings.json` | JSON | 自定义 | 是 |
| `coffeecli` | Coffee CLI | `~/.coffee-cli/config.json` | JSON | 通用 | 否 |
| `grok` | Grok | `~/.grok/config.toml` | TOML | 自定义 | 是 |
| `openfang` | Open Fang | `~/.openfang/config.toml` | TOML | 通用 | 是 |
| `pi` | Pi | `~/.pi/agent/settings.json` | JSON | 通用 | 否 |
| `picoclaw` | Pico Claw | `~/.picoclaw/config.json` | JSON | 通用 | 是 |
| `nanobot` | Nano Bot | `~/.nanobot/config.json` | JSON | 通用 | 是 |
| `zeroclaw` | Zero Claw | `~/.zeroclaw/config.toml` | TOML | 通用 | 是 |

### Desktop Apps

| ID | 名称 | 配置文件 | 格式 |
|----|------|----------|------|
| `claudedesktop` | Claude Desktop | `~/.claude/settings.json` | JSON |
| `codexdesktop` | Codex Desktop | `~/.codex/config.toml` | TOML |
| `geminidesktop` | Gemini Desktop | `~/.gemini/settings.json` | JSON |

### IDE / Editor

| ID | 名称 | 配置文件 | 格式 |
|----|------|----------|------|
| `cursor` | Cursor | `~/.cursor/settings.json` | JSON |
| `windsurf` | Windsurf | `~/.codeium/windsurf/settings.json` | JSON |
| `vscode` | VS Code | `~/.vscode/settings.json` | JSON |
| `trae` | Trae IDE | `~/.trae/settings.json` | JSON |
| `traecn` | Trae CN | `~/.trae-cn/settings.json` | JSON |

### Embedded

| ID | 名称 | 运行时 |
|----|------|--------|
| `reversi` | AI Reversi | `public/tools/reversi.html` |
| `ai-trader` | AI Trader | 内嵌 |
| `fingpt` | FinGPT | 内嵌 |
| `tradingagents` | Trading Agents | 内嵌 |
| `translator` | Translator | `public/tools/translator.html` |

---

## 七、AIDAGuard 声明式适配器可行性

### 7.1 当前问题

AIDAGuard 每个工具适配器平均 ~100 行 Rust 代码，核心逻辑（detect、configure、restore、is_configured）高度重复：
- 所有适配器都是"找到配置文件 → 读写特定字段"的模式
- 仅字段路径和配置格式不同
- 添加新工具需编写完整的 Rust trait 实现

### 7.2 声明式方案

参考 EchoBird，AIDAGuard 可以用 `config.json` 描述每个工具的配置映射：

```json
{
  "id": "cline",
  "name": "Cline",
  "detect": { "dir": "~/.cline" },
  "configFile": "~/.cline/data/globalState.json",
  "format": "json",
  "endpoint": {
    "read": ["openAiBaseUrl", "anthropicBaseUrl"],
    "write": {
      "openAiBaseUrl": "proxyUrl",
      "anthropicBaseUrl": "proxyUrl"
    }
  },
  "model": {
    "read": ["actModeApiModelId", "planModeApiModelId"]
  },
  "secondaryConfigs": [
    {
      "configFile": "~/Library/Application Support/Code/User/settings.json",
      "endpoint": {
        "read": ["cline.openAiBaseUrl", "cline.anthropicBaseUrl"],
        "write": {
          "cline.openAiBaseUrl": "proxyUrl",
          "cline.anthropicBaseUrl": "proxyUrl"
        }
      }
    }
  ]
}
```

### 7.3 收益

- 新增工具：2 个 JSON 文件 vs ~100 行 Rust 代码
- 配置错误：JSON 语法错误 vs Rust 逻辑错误
- 代码量：1 个通用适配引擎 vs 17 个独立适配器
- 维护成本：集中修改引擎逻辑 vs 逐个适配器修改

### 7.4 限制

- Cline 双配置源等复杂场景仍需自定义逻辑（可用 `custom: true` 标记降级）
- Copilot 等无法直接修改 BaseURL 的工具仍需特殊处理
- 备份/还原策略因工具而异（单文件 vs 多文件 vs 共享文件）

---

## 八、参考资料

- EchoBird 仓库：https://github.com/edison7009/EchoBird
- EchoBird 官网：https://echobird.ai
- AIDAGuard 适配器文档：`AI_TOOLS_ADAPTERS.md`
- AIDAGuard 架构文档：`ARCHITECTURE.md`
