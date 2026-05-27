# AI 智能体工具适配器分析

**文档版本：** v1.0  
**更新日期：** 2026-05-16  
**用途：** AIDAGuard Phase 4.2 技术参考

**相关文档：**
- [适配器架构 V2 设计文档](./ADAPTER_ARCHITECTURE_V2.md) — 声明式适配器引擎设计方案（Phase 5）
- [EchoBird 工具配置系统参考](./ECHOBIRD_TOOLS_REFERENCE.md) — EchoBird 的声明式工具配置方法

---

## 一、工具分类

### 1.1 IDE 集成类

| 工具 | 平台 | 类型 | 当前支持 |
|------|------|------|----------|
| Cursor | 独立 IDE | AI-First Editor | ✅ |
| Windsurf | 独立 IDE | AI-First Editor | ✅ |
| Zed | 独立 IDE | 高性能编辑器 | ✅ |
| VS Code + Cline | VS Code | 扩展 | ✅ |
| VS Code + Continue.dev | VS Code | 扩展 | ✅ |
| VS Code + Codeium | VS Code | 扩展 | ✅ |
| VS Code + Cody | VS Code | 扩展 | ✅ |
| VS Code + Tabnine | VS Code | 扩展 | ✅ |
| VS Code + CodeWhisperer | VS Code | 扩展 | ✅ |
| JetBrains AI | JetBrains IDE | 插件 | ✅ |
| VS Code (built-in) | VS Code | 内置 | ✅ |
| Trae IDE | 独立 IDE | AI-First Editor | ✅ |
| Trae CN | 独立 IDE | AI-First Editor | ✅ |

### 1.2 CLI 工具类

| 工具 | 类型 | 当前支持 |
|------|------|----------|
| Claude Code | Anthropic CLI | ✅ |
| Aider | Git 集成 CLI | ✅ |
| OpenAI Codex | OpenAI CLI | ✅ |
| Gemini CLI | Google CLI | ✅ |
| OpenClaw | 通用 CLI | ✅ |
| OpenCode | 通用 CLI | ✅ |
| Qwen Code | 通义千问 CLI | ✅ |
| Coffee CLI | 通用 CLI | ✅ |
| Grok | xAI CLI | ✅ |
| Open Fang | 通用 CLI | ✅ |
| Pi | Inflection CLI | ✅ |
| Pico Claw | 通用 CLI | ✅ |
| Nano Bot | 通用 CLI | ✅ |
| Zero Claw | 通用 CLI | ✅ |

### 1.3 Agent 平台类

| 工具 | 类型 | 当前支持 |
|------|------|----------|
| Hermes Agent | 自定义 Agent | ✅ |

### 1.4 桌面应用类

| 工具 | 类型 | 当前支持 |
|------|------|----------|
| Claude Desktop | Anthropic 桌面 | ✅ |
| Codex Desktop | OpenAI 桌面 | ✅ |
| Gemini Desktop | Google 桌面 | ✅ |

---

## 二、已支持工具详情

### 2.1 Cursor

**配置文件路径：**
- macOS: `~/Library/Application Support/Cursor/User/settings.json`
- Linux: `~/.config/Cursor/User/settings.json`
- Windows: `%APPDATA%\Cursor\User\settings.json`

**API 配置方式：**
```json
{
  "cursor.aiProvider": "openai",
  "cursor.openaiApiKey": "sk-xxx",
  "cursor.openaiBaseUrl": "https://api.openai.com/v1",
  "cursor.model": "gpt-4o"
}
```

**代理配置：**
```json
{
  "http.proxy": "http://127.0.0.1:19000",
  "http.proxyStrictSSL": false
}
```

**检测方式：**
- 检查配置目录是否存在
- macOS: `~/Library/Application Support/Cursor/`

**备份文件：**
- `settings.json`
- `keybindings.json`
- `snippets/`

---

### 2.2 Windsurf

**配置文件路径：**
- macOS: `~/Library/Application Support/Windsurf/User/settings.json`
- Linux: `~/.config/Windsurf/User/settings.json`

**API 配置方式：**
```json
{
  "windsurf.aiProvider": "openai-compatible",
  "windsurf.apiKey": "sk-xxx",
  "windsurf.baseUrl": "https://api.openai.com/v1",
  "windsurf.model": "gpt-4o"
}
```

**代理配置：** 同 Cursor

---

### 2.3 Cline

**扩展 ID：** `saoudrizwan.claude-dev` (VS Code) / `cline/cline` (CLI)

Cline 从**两个来源**读取 BaseURL 配置，**VS Code settings 优先级更高**：

| 优先级 | 配置来源 | 字段 |
|--------|----------|------|
| 高 | VS Code `settings.json` | `cline.openAiBaseUrl`, `cline.anthropicBaseUrl` |
| 低 | `~/.cline/data/globalState.json` | `openAiBaseUrl`, `anthropicBaseUrl` |

**配置文件路径：**
- 全局设置: `~/.cline/data/globalState.json` — 包含 API 提供商、模型选择、base URL 等配置
- VS Code 设置: `<VS Code>/User/settings.json` — `cline.openAiBaseUrl`, `cline.anthropicBaseUrl`
- MCP 设置: `~/.cline/data/settings/cline_mcp_settings.json`
- 任务历史: `~/.cline/data/sessions/`
- 工作区状态: `.cline/`（每个项目独立）

**API 配置字段（在 globalState.json 中）：**
```json
{
  "apiProvider": "openai",
  "actModeApiModelId": "gpt-4o",
  "planModeApiModelId": "claude-sonnet-4-6",
  "openAiBaseUrl": "https://api.openai.com/v1",
  "anthropicBaseUrl": "https://api.anthropic.com"
}
```

**VS Code settings.json 中的配置：**
```json
{
  "cline.openAiBaseUrl": "http://127.0.0.1:19000",
  "cline.anthropicBaseUrl": "http://127.0.0.1:19000"
}
```

**API Provider 支持：** `openai`, `anthropic`, `openrouter`, `ollama`, `gemini`, `litellm`, `bedrock`, `vertex` 等 45+ 个提供商。

**代理配置方式：**
AIDAGuard **同时修改两个配置源**，确保 Cline 无论从哪个来源读取都能生效：
```json
// globalState.json
{
  "openAiBaseUrl": "http://127.0.0.1:19000",
  "anthropicBaseUrl": "http://127.0.0.1:19000"
}

// VS Code settings.json
{
  "cline.openAiBaseUrl": "http://127.0.0.1:19000",
  "cline.anthropicBaseUrl": "http://127.0.0.1:19000"
}
```

**检测方式：**
- 检查 `~/.cline/` 目录是否存在

**备份策略：**
- 备份 `~/.cline/data/globalState.json`
- 备份 VS Code `settings.json`（因为 cline.* 字段也写入此文件）

---

### 2.4 Claude Code

**配置文件路径：**
- `~/.claude/settings.json` — Claude Code 全局设置文件

**API 配置方式：**
Claude Code 通过 `settings.json` 中的 `env` 对象设置环境变量：
```json
{
  "env": {
    "ANTHROPIC_AUTH_TOKEN": "sk-ant-xxx",
    "ANTHROPIC_BASE_URL": "https://api.anthropic.com",
    "ANTHROPIC_MODEL": "claude-sonnet-4-6"
  }
}
```

或使用 shell 环境变量：
```bash
export ANTHROPIC_API_KEY="sk-ant-xxx"
export ANTHROPIC_BASE_URL="https://api.anthropic.com"
```

**代理配置方式：**
AIDAGuard 修改 `settings.json` 中 `env.ANTHROPIC_BASE_URL`，将 API 请求路由到本地代理：
```json
{
  "env": {
    "ANTHROPIC_BASE_URL": "http://127.0.0.1:19000"
  }
}
```

**检测方式：**
- 检查 `~/.claude/` 目录是否存在

**备份策略：**
- 备份 `~/.claude/settings.json`

---

### 2.5 Aider

**配置文件路径：**
- `~/.aider.conf.yml`
- 或环境变量

**API 配置方式：**
```yaml
# ~/.aider.conf.yml
api-key: sk-xxx
api-base: https://api.openai.com/v1
model: gpt-4o
```

或环境变量：
```bash
export OPENAI_API_KEY="sk-xxx"
export OPENAI_API_BASE="https://api.openai.com/v1"
```

**代理配置：**
```bash
export HTTP_PROXY="http://127.0.0.1:19000"
```

**检测方式：**
- `which aider`

---

### 2.6 Continue.dev

**配置文件路径：**
- `~/.continue/config.json`

**API 配置方式：**
```json
{
  "models": [
    {
      "title": "GPT-4o",
      "provider": "openai",
      "model": "gpt-4o",
      "apiKey": "sk-xxx",
      "apiBase": "https://api.openai.com/v1"
    }
  ]
}
```

**代理配置：**
- 通过系统代理或环境变量

**检测方式：**
- 检查 VS Code 扩展: `~/.vscode/extensions/continue.continue-*`

---

### 2.7 Zed

**配置文件路径：**
- `~/.zed/settings.json`

**API 配置方式：**
```json
{
  "language_models": {
    "openai": {
      "api_key": "sk-xxx",
      "base_url": "https://api.openai.com/v1"
    }
  }
}
```

**代理配置：**
```json
{
  "http_proxy": "http://127.0.0.1:19000"
}
```

**检测方式：**
- 检查 `~/.zed/` 目录

---

## 三、待开发工具详情

### 3.1 Codeium

**配置文件路径：**
- `~/.codeium/config.json`
- VS Code 扩展: `~/.vscode/extensions/codeium.codeium-*`

**API 配置方式：**
```json
{
  "apiKey": "xxx",
  "baseUrl": "https://server.codeium.com"
}
```

**代理配置：**
```json
{
  "httpProxy": "http://127.0.0.1:19000"
}
```

**检测方式：**
- `~/.codeium/` 目录
- VS Code 扩展目录

---

### 3.2 Sourcegraph Cody

**配置文件路径：**
- `~/.cody/config.json`
- VS Code 扩展: `~/.vscode/extensions/sourcegraph.cody-*`

**API 配置方式：**
```json
{
  "provider": "openai",
  "apiKey": "sk-xxx",
  "baseUrl": "https://api.openai.com/v1"
}
```

**代理配置：**
- VS Code 全局代理设置

**检测方式：**
- `~/.cody/` 目录
- VS Code 扩展目录

---

### 3.3 Tabnine

**配置文件路径：**
- `~/.tabnine/config.json`
- VS Code 扩展: `~/.vscode/extensions/tabnine.tabnine-*`

**API 配置方式：**
```json
{
  "api_key": "xxx",
  "server_url": "https://api.tabnine.com"
}
```

**代理配置：**
- 环境变量或配置文件

**检测方式：**
- `~/.tabnine/` 目录

---

### 3.4 Amazon CodeWhisperer

**配置文件路径：**
- AWS credentials: `~/.aws/credentials`
- VS Code 扩展: `~/.vscode/extensions/amazonwebservices.aws-toolkit-vscode-*`

**认证方式：**
- AWS IAM 凭证
- AWS SSO

**配置示例：**
```ini
# ~/.aws/credentials
[default]
aws_access_key_id = AKIAXXX
aws_secret_access_key = xxx
region = us-east-1
```

**代理配置：**
```bash
export HTTP_PROXY="http://127.0.0.1:19000"
export HTTPS_PROXY="http://127.0.0.1:19000"
```

**检测方式：**
- `~/.aws/credentials` 文件
- VS Code 扩展目录

**特殊说明：**
- 需要安装 AWS Toolkit 扩展
- 使用 AWS 凭证而非 API Key

---

### 3.5 JetBrains AI

**配置文件路径：**
- `~/.config/JetBrains/*/settings.xml`
- 或 IDE 内配置

**认证方式：**
- JetBrains Account
- OpenAI API Key

**代理配置：**
- IDE 设置 → Appearance & Behavior → System Settings → HTTP Proxy

**检测方式：**
- JetBrains 配置目录: `~/.config/JetBrains/`
- 或 `~/Library/Application Support/JetBrains/` (macOS)

---

## 四、工具适配器开发模板

### 4.1 标准模板

```rust
// crates/aidaguard-plugins/src/adapters/xxx.rs

use crate::{ToolAdapter, PluginManifest, backup::BackupManager};
use std::path::PathBuf;

pub struct XxxAdapter;

impl XxxAdapter {
    pub const ID: &'static str = "xxx";
    pub const NAME: &'static str = "Xxx Tool";
}

impl ToolAdapter for XxxAdapter {
    fn id(&self) -> &str { Self::ID }
    fn name(&self) -> &str { Self::NAME }
    
    fn config_path(&self) -> &str {
        // 配置文件路径
        "~/.xxx/config.json"
    }
    
    fn detect(&self) -> bool {
        // 检测工具是否安装
        Self::config_path().exists() || 
        PathBuf::from("~/.vscode/extensions/xxx-*").exists()
    }
    
    fn current_endpoint(&self) -> Option<String> {
        // 读取当前配置的端点
        let config = Self::read_config()?;
        config.get("baseUrl")?.as_str().map(|s| s.to_string())
    }
    
    fn current_model(&self) -> Option<String> {
        // 读取当前模型
        let config = Self::read_config()?;
        config.get("model")?.as_str().map(|s| s.to_string())
    }
    
    fn configure(&self, proxy_url: &str) -> Result<(), String> {
        // 配置代理
        let mut config = Self::read_config().unwrap_or(json!({}));
        config["httpProxy"] = json!(proxy_url);
        Self::write_config(&config)?;
        Ok(())
    }
    
    fn restore(&self) -> Result<(), String> {
        // 恢复原始配置
        BackupManager::new().restore(Self::ID)
    }
    
    fn is_configured(&self) -> bool {
        // 检查是否已配置代理
        let config = Self::read_config().unwrap_or(json!({}));
        config.get("httpProxy").is_some()
    }
    
    fn backup(&self, backup_dir: &Path) -> Result<(), String> {
        // 备份配置
        BackupManager::new().backup(Self::ID, &[
            Self::config_path(),
        ], backup_dir)
    }
}

impl Plugin for XxxAdapter {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            id: Self::ID.to_string(),
            name: Self::NAME.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Description of the tool".to_string(),
            author: "Author Name".to_string(),
            config_path_template: Self::config_path().to_string(),
            categories: vec!["ide".to_string()],
        }
    }
}
```

### 4.2 环境变量工具模板

```rust
// 适用于 CLI 工具（Claude Code, Aider 等）

impl ToolAdapter for CliToolAdapter {
    fn config_path(&self) -> &str {
        ""  // 无配置文件
    }
    
    fn detect(&self) -> bool {
        // 检查命令是否存在
        which::which("cli-tool").is_ok()
    }
    
    fn current_endpoint(&self) -> Option<String> {
        std::env::var("TOOL_API_BASE").ok()
    }
    
    fn configure(&self, proxy_url: &str) -> Result<(), String> {
        // 设置环境变量（需要写入 shell 配置文件）
        let shell_config = Self::get_shell_config()?;
        let export_line = format!(r#"export HTTP_PROXY="{}""#, proxy_url);
        // 追加到配置文件...
        Ok(())
    }
    
    fn restore(&self) -> Result<(), String> {
        // 从 shell 配置中移除代理设置
        Ok(())
    }
}
```

---

## 五、配置文件格式汇总

| 工具 | 配置格式 | 主要字段 |
|------|----------|----------|
| Cursor | JSON | `cursor.aiProvider`, `cursor.openaiApiKey`, `cursor.openaiBaseUrl` |
| Windsurf | JSON | `windsurf.apiProvider`, `windsurf.apiKey`, `windsurf.baseUrl` |
| Cline | JSON (`~/.cline/data/globalState.json` + VS Code `settings.json`) | `openAiBaseUrl`, `anthropicBaseUrl`, `cline.openAiBaseUrl`, `cline.anthropicBaseUrl` |
| Claude Code | JSON (`~/.claude/settings.json`) | `env.ANTHROPIC_BASE_URL` |
| Aider | YAML/环境变量 | `api-key`, `api-base`, `model` |
| Continue.dev | JSON | `models[].provider`, `models[].apiKey`, `models[].apiBase` |
| Zed | JSON | `language_models.openai.api_key`, `language_models.openai.base_url` |
| Codeium | JSON | `apiKey`, `baseUrl` |
| Cody | JSON | `provider`, `apiKey`, `baseUrl` |
| Tabnine | JSON | `api_key`, `server_url` |
| CodeWhisperer | INI (AWS) | `aws_access_key_id`, `aws_secret_access_key` |
| OpenCode | JSON | 多 provider 配置 |
| OpenClaw | JSON | `providers.*.baseURL` |
| Claude Desktop | JSON | `env.ANTHROPIC_BASE_URL` |
| VS Code | JSON | `http.proxy` |
| JetBrains AI | XML | IDE options |
| Gemini CLI | .env 文件 | `GOOGLE_GEMINI_BASE_URL` |

> 完整清单详见 [EchoBird 工具配置系统参考](./ECHOBIRD_TOOLS_REFERENCE.md)。

---

## 六、代理配置方式汇总

### 6.1 HTTP 代理设置

**方式一：配置文件**
```json
{
  "http.proxy": "http://127.0.0.1:19000",
  "http.proxyStrictSSL": false
}
```

**方式二：环境变量**
```bash
export HTTP_PROXY="http://127.0.0.1:19000"
export HTTPS_PROXY="http://127.0.0.1:19000"
export NO_PROXY="localhost,127.0.0.1"
```

**方式三：系统代理**
- macOS: 系统偏好设置 → 网络 → 高级 → 代理
- Windows: 设置 → 网络和 Internet → 代理

### 6.2 注意事项

1. **HTTPS 代理：** 大多数工具需要同时设置 HTTP_PROXY 和 HTTPS_PROXY
2. **SSL 验证：** 本地代理通常需要禁用 SSL 验证
3. **代理绕过：** 某些请求可能需要绕过代理（如本地服务）

---

## 七、开发状态

所有 31 个工具适配器已实现：

- **25 个声明式适配器**：通过 JSON 清单（`manifests/*.json`）编译时嵌入，由通用的 `DeclarativeAdapter` 引擎驱动。新工具的添加只需创建清单 JSON 文件，无需编写 Rust 代码。
- **6 个复杂适配器**：Aider（YAML）、Codex（多格式 JSON/YAML/TOML）、Hermes Agent（YAML）、Gemini CLI（.env 文件）、CodeWhisperer（AWS 凭证）、JetBrains AI（XML 配置）。这些工具使用非 JSON 格式或需要特殊逻辑，故保留手写 Rust 实现。

详见 [适配器架构 V2 设计文档](./ADAPTER_ARCHITECTURE_V2.md)。

---

## 八、测试清单

### 8.1 功能测试

- [ ] 工具检测正确
- [ ] 配置读取正确
- [ ] 代理配置生效
- [ ] 配置恢复正常
- [ ] 备份功能正常

### 8.2 兼容性测试

- [ ] macOS 测试通过
- [ ] Linux 测试通过
- [ ] Windows 测试通过

### 8.3 边界情况

- [ ] 工具未安装时的处理
- [ ] 配置文件不存在的处理
- [ ] 配置文件格式错误时的处理
- [ ] 权限不足时的处理
