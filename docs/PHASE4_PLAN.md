# Phase 4: 大模型提供商与AI智能体工具整合

**版本：** v0.5.0 → v0.6.0  
**优先级：** P1  
**预估工作量：** 15-20 天

---

## 一、目标

1. **大模型提供商整合** - 统计计费、coding plan 计数、token plan 统计
2. **AI智能体工具插件化** - 全面支持主流工具，逐个分析配置方式
3. **自定义配置选项** - 支持不在列表中的智能体工具

---

## 二、当前状态

### 2.1 已有大模型提供商支持

```
aidaguard-upstream/
├── types.rs          → ProviderConfig, UpstreamConfig, ModelInfo
├── provider.rs      → ProviderRegistry (内置提供商)
├── manager.rs       → UpstreamManager
└── client.rs        → UpstreamClient

内置提供商:
- OpenAI (openai.yaml)
- Anthropic (anthropic.yaml)
- DeepSeek (deepseek.yaml)
- Qwen (qwen.yaml)
- Zhipu (zhipu.yaml)
- Groq (groq.yaml)
- Gemini (gemini.yaml)
```

### 2.2 已有智能体工具支持

```
aidaguard-plugins/src/adapters/
├── cursor.rs        → Cursor IDE
├── cline.rs         → Cline (VS Code)
├── claude_code.rs   → Claude Code CLI
├── codex.rs         → OpenAI Codex
├── continue_dev.rs  → Continue.dev
├── windsurf.rs      → Windsurf IDE
├── gemini.rs        → Google Gemini
├── zed.rs           → Zed Editor
├── aider.rs         → Aider CLI
├── roo_code.rs      → Roo Code
├── opencode.rs      → OpenCode
├── openclaw.rs      → OpenClaw
└── hermes_agent.rs  → Hermes Agent

共 13 个适配器
```

### 2.3 缺失功能

| 功能 | 状态 | 说明 |
|------|------|------|
| 计费统计 | ❌ 缺失 | 无 token 使用量追踪 |
| coding plan 计数 | ❌ 缺失 | 无 coding 计划统计 |
| token plan 统计 | ❌ 缺失 | 无 token 配额管理 |
| 主流智能体工具 | ⚠️ 部分 | 缺少部分主流工具 |
| 自定义智能体 | ❌ 缺失 | 无法添加自定义工具 |

---

## 三、Phase 4.1: 大模型提供商计费统计

### 3.1 主流大模型提供商接口分析

| 提供商 | API 协议 | 认证方式 | Token 统计 | 价格模型 |
|--------|----------|----------|------------|----------|
| **OpenAI** | OpenAI Compatible | Bearer Token | usage 字段 | GPT-4o: $2.5/$10 per 1M tokens |
| **Anthropic** | Anthropic Compatible | x-api-key header | usage 字段 | Claude-3.5: $3/$15 per 1M tokens |
| **DeepSeek** | OpenAI Compatible | Bearer Token | usage 字段 | DeepSeek-V3: $0.27/$1.1 per 1M tokens |
| **Qwen** | OpenAI Compatible | Bearer Token | usage 字段 | Qwen-Max: ¥2/¥8 per 1M tokens |
| **Zhipu** | OpenAI Compatible | Bearer Token | usage 字段 | GLM-4: ¥0.1/¥0.1 per 1K tokens |
| **Gemini** | Gemini SDK | API Key param | usageMetadata | Flash: $0.075/$0.3 per 1M tokens |
| **Groq** | OpenAI Compatible | Bearer Token | usage 字段 | Llama: Free tier available |
| **Moonshot** | OpenAI Compatible | Bearer Token | usage 字段 | Moonshot-V1: ¥0.012/¥0.012 per 1K tokens |
| **Baidu Qianfan** | OpenAI Compatible | Bearer Token | usage 字段 | ERNIE-4: ¥0.12/¥0.12 per 1K tokens |
| **ByteDance Doubao** | OpenAI Compatible | Bearer Token | usage 字段 | Doubao-Pro: ¥0.0008/¥0.002 per 1K tokens |

### 3.2 Token 统计设计方案

```rust
// aidaguard-upstream/src/usage.rs

/// Token 使用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub id: String,              // UUID
    pub timestamp_ms: i64,
    pub upstream_id: String,     // 提供商 ID
    pub model: String,           // 模型 ID
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub request_type: RequestType,
    pub estimated_cost_usd: f64, // 估算成本（美元）
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestType {
    Chat,        // 普通对话
    Coding,      // 代码生成
    Embedding,    // 向量化
    Other,
}

/// Token 配额计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPlan {
    pub id: String,
    pub name: String,
    pub upstream_id: String,
    pub daily_limit: Option<u64>,     // 每日 token 限制
    pub monthly_limit: Option<u64>,   // 每月 token 限制
    pub budget_usd: Option<f64>,      // 预算限制（美元）
    pub used_today: u64,
    pub used_month: u64,
    pub spent_usd: f64,
}

/// Coding Plan 计数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodingPlan {
    pub id: String,
    pub upstream_id: String,
    pub total_requests: u64,    // 总请求次数
    pub coding_requests: u64,  // 编码请求次数
    pub success_count: u64,    // 成功次数
    pub error_count: u64,      // 失败次数
    pub avg_latency_ms: f64,   // 平均延迟
}
```

### 3.3 计费统计存储

```rust
// aidaguard-storage/src/usage.rs

impl Storage {
    /// 记录 token 使用
    pub fn record_usage(&self, usage: &TokenUsage) -> Result<(), StorageError>;
    
    /// 查询使用记录
    pub fn list_usage(&self, filter: UsageFilter) -> Result<Vec<TokenUsage>, StorageError>;
    
    /// 按提供商统计
    pub fn usage_by_upstream(&self, from_ms: i64, to_ms: i64) -> Result<Vec<UpstreamUsage>, StorageError>;
    
    /// Token 配额管理
    pub fn get_token_plan(&self, upstream_id: &str) -> Result<Option<TokenPlan>, StorageError>;
    pub fn update_token_plan(&self, plan: &TokenPlan) -> Result<(), StorageError>;
    
    /// 每日重置
    pub fn reset_daily_usage(&self) -> Result<(), StorageError>;
}
```

### 3.4 文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `aidaguard-upstream/src/usage.rs` | 新增 | TokenUsage, TokenPlan, CodingPlan |
| `aidaguard-upstream/src/pricing.rs` | 新增 | 价格模型定义 |
| `aidaguard-storage/src/usage.rs` | 新增 | 使用记录存储 |
| `aidaguard-proxy/src/server.rs` | 修改 | 提取 usage 字段并记录 |
| `aidaguard-tauri/.../usage.rs` | 新增 | 前端命令接口 |

---

## 四、Phase 4.2: AI智能体工具插件化

### 4.1 主流 AI 智能体工具分析

| 工具 | 类型 | 配置文件 | API 配置方式 | 当前支持 |
|------|------|----------|--------------|----------|
| **Cursor** | IDE | `~/.cursor/settings.json` | OpenAI Compatible | ✅ |
| **Windsurf** | IDE | `~/.windsurf/settings.json` | OpenAI Compatible | ✅ |
| **Cline** | VS Code Ext | VS Code settings | OpenAI Compatible | ✅ |
| **Roo Code** | VS Code Ext | VS Code settings | OpenAI Compatible | ✅ |
| **Continue.dev** | VS Code Ext | `~/.continue/config.json` | 多协议支持 | ✅ |
| **Claude Code** | CLI | 环境变量 | Anthropic SDK | ✅ |
| **Aider** | CLI | 环境变量 | OpenAI Compatible | ✅ |
| **Zed** | Editor | `~/.zed/settings.json` | OpenAI Compatible | ✅ |
| **OpenAI Codex** | CLI | 环境变量 | OpenAI SDK | ✅ |
| **Gemini CLI** | CLI | 环境变量 | Gemini SDK | ✅ |
| **OpenClaw** | CLI | 配置文件 | OpenAI Compatible | ✅ |
| **Hermes Agent** | Agent | 配置文件 | OpenAI Compatible | ✅ |
| **OpenCode** | CLI | 配置文件 | OpenAI Compatible | ✅ |

### 4.2 需要新增的工具

| 工具 | 类型 | 配置文件 | API 协议 | 优先级 |
|------|------|----------|----------|--------|
| **Copilot** | VS Code Ext | GitHub Copilot settings | GitHub Copilot API | P1 |
| **Codeium** | VS Code Ext | `~/.codeium/config.json` | Codeium API | P1 |
| **Tabnine** | VS Code Ext | `~/.tabnine/config.json` | Tabnine API | P2 |
| **Amazon CodeWhisperer** | VS Code Ext | AWS credentials | AWS SDK | P2 |
| **Replit AI** | Web IDE | Replit settings | Replit API | P3 |
| **Sourcegraph Cody** | VS Code Ext | `~/.cody/config.json` | Sourcegraph API | P2 |
| **JetBrains AI** | IDE Plugin | JetBrains settings | JetBrains AI API | P2 |

### 4.3 工具适配器模板

每个工具适配器需要分析：

```rust
// adapters/template.rs

/// 工具适配器分析模板
pub struct ToolAdapterTemplate {
    // 1. 基本信息
    pub id: &'static str,           // 唯一标识
    pub name: &'static str,         // 显示名称
    
    // 2. 配置文件路径
    pub config_path_template: &'static str,  // 配置文件路径模板
    
    // 3. API 配置字段
    pub endpoint_field: &'static str,    // 端点配置字段名
    pub api_key_field: &'static str,    // API Key 字段名
    pub model_field: Option<&'static str>, // 模型字段名（可选）
    
    // 4. 检测方式
    pub detect_paths: &'static [&'static str],  // 工具安装路径
    pub detect_env_vars: &'static [&'static str], // 环境变量检测
    
    // 5. API 协议
    pub protocol: ProtocolType,      // API 协议类型
    pub auth_type: AuthType,         // 认证方式
    
    // 6. 备份策略
    pub backup_files: &'static [&'static str], // 需要备份的文件
    
    // 7. 特殊配置
    pub special_config: Option<SpecialConfig>, // 特殊配置需求
}
```

### 4.4 新增适配器示例：GitHub Copilot

```rust
// adapters/copilot.rs

use crate::{ToolAdapter, PluginManifest};
use crate::backup::BackupManager;

pub struct Copilot;

impl Copilot {
    const ID: &'static str = "copilot";
    const NAME: &'static str = "GitHub Copilot";
    
    // GitHub Copilot 使用 OAuth 认证，需要特殊处理
    const AUTH_URL: &'static str = "https://github.com/login/device";
    const TOKEN_URL: &'static str = "https://github.com/login/oauth/access_token";
}

impl ToolAdapter for Copilot {
    fn id(&self) -> &str { Self::ID }
    fn name(&self) -> &str { Self::NAME }
    
    fn config_path(&self) -> &str {
        // VS Code 扩展配置存储在全局 storage
        "~/.vscode/extensions/github.copilot-*/"
    }
    
    fn detect(&self) -> bool {
        // 检测 VS Code 扩展目录
        std::path::Path::new("~/.vscode/extensions")
            .join("github.copilot-*")
            .exists()
    }
    
    fn current_endpoint(&self) -> Option<String> {
        // Copilot 使用固定端点
        Some("https://api.githubcopilot.com".to_string())
    }
    
    fn current_model(&self) -> Option<String> {
        // Copilot 默认模型
        Some("gpt-4o-copilot".to_string())
    }
    
    fn configure(&self, proxy_url: &str) -> Result<(), String> {
        // GitHub Copilot 需要修改 VS Code settings.json
        let settings_path = dirs::config_dir()
            .ok_or("Cannot find config dir")?
            .join("Code/User/settings.json");
        
        // 读取现有配置
        let mut settings: serde_json::Value = if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)
                .map_err(|e| e.to_string())?;
            serde_json::from_str(&content).unwrap_or(json!({}))
        } else {
            json!({})
        };
        
        // 设置代理
        settings["http.proxy"] = json!(proxy_url);
        settings["http.proxyStrictSSL"] = json!(false);
        
        // 写回配置
        std::fs::write(&settings_path, serde_json::to_string_pretty(&settings).unwrap())
            .map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    fn restore(&self) -> Result<(), String> {
        // 恢复原始配置
        let backup = BackupManager::new();
        backup.restore(Self::ID)
    }
    
    fn is_configured(&self) -> bool {
        // 检查是否配置了代理
        let settings_path = dirs::config_dir()
            .and_then(|p| Some(p.join("Code/User/settings.json")));
        
        if let Some(path) = settings_path {
            if path.exists() {
                let content = std::fs::read_to_string(&path).ok()?;
                let settings: serde_json::Value = serde_json::from_str(&content).ok()?;
                return settings.get("http.proxy").is_some();
            }
        }
        false
    }
    
    fn backup(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        let backup = BackupManager::new();
        backup.backup(Self::ID, &[
            "~/.vscode/extensions/github.copilot-*",
            "~/Library/Application Support/Code/User/settings.json",
        ], backup_dir)
    }
}

impl Plugin for Copilot {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            id: Self::ID.to_string(),
            name: Self::NAME.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "GitHub Copilot - AI pair programmer".to_string(),
            author: "GitHub".to_string(),
            config_path_template: "~/.vscode/extensions/github.copilot-*/".to_string(),
            categories: vec!["ide".to_string(), "vscode-extension".to_string()],
        }
    }
}
```

### 4.5 文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `aidaguard-plugins/src/adapters/copilot.rs` | 新增 | GitHub Copilot 适配器 |
| `aidaguard-plugins/src/adapters/codeium.rs` | 新增 | Codeium 适配器 |
| `aidaguard-plugins/src/adapters/cody.rs` | 新增 | Sourcegraph Cody 适配器 |
| `aidaguard-plugins/src/adapters/tabnine.rs` | 新增 | Tabnine 适配器 |
| `aidaguard-plugins/src/adapters/codewhisperer.rs` | 新增 | Amazon CodeWhisperer 适配器 |
| `aidaguard-plugins/src/adapters/jetbrains_ai.rs` | 新增 | JetBrains AI 适配器 |
| `aidaguard-plugins/src/adapters/mod.rs` | 修改 | 导出新模块 |

---

## 五、Phase 4.3: 自定义 AI 智能体配置

### 5.1 自定义配置 UI 设计

```typescript
// 前端组件: CustomAgentConfig.tsx

interface CustomAgentConfig {
  id: string;
  name: string;
  description?: string;
  
  // API 配置
  endpoint: string;
  protocol: 'openai' | 'anthropic' | 'custom';
  authType: 'bearer' | 'api_key_header' | 'custom';
  authHeader?: string;  // 自定义 header 名称
  
  // 模型配置
  defaultModel?: string;
  models?: CustomModel[];
  
  // 请求配置
  timeout?: number;
  maxRetries?: number;
  
  // 自定义请求模板（高级）
  requestTemplate?: RequestTemplate;
}

interface CustomModel {
  id: string;
  name: string;
  contextWindow?: number;
  maxOutput?: number;
  capabilities?: string[];
}

interface RequestTemplate {
  // 自定义请求体格式
  bodyTemplate?: string;  // JSON 模板，支持变量替换
  headerTemplate?: Record<string, string>;
  responseParser?: string;  // JSONPath 表达式
}
```

### 5.2 自定义配置存储

```rust
// aidaguard-plugins/src/custom.rs

/// 自定义智能体配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomAgentConfig {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    
    // API 配置
    pub endpoint: String,
    pub protocol: ProtocolType,
    pub auth_type: AuthType,
    pub auth_header: Option<String>,
    
    // 模型配置
    pub default_model: Option<String>,
    pub models: Vec<CustomModel>,
    
    // 请求配置
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    pub max_retries: Option<u32>,
    
    // 自定义请求模板
    pub request_template: Option<RequestTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomModel {
    pub id: String,
    pub name: String,
    pub context_window: Option<usize>,
    pub max_output: Option<usize>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestTemplate {
    pub body_template: Option<String>,
    pub header_template: Option<HashMap<String, String>>,
    pub response_parser: Option<String>,
}

/// 自定义智能体注册表
pub struct CustomAgentRegistry {
    agents: Vec<CustomAgentConfig>,
    config_path: PathBuf,
}

impl CustomAgentRegistry {
    pub fn new(config_dir: PathBuf) -> Self {
        let config_path = config_dir.join("custom_agents.json");
        let mut registry = Self {
            agents: Vec::new(),
            config_path,
        };
        registry.load();
        registry
    }
    
    pub fn add(&mut self, agent: CustomAgentConfig) -> Result<(), String> {
        if self.agents.iter().any(|a| a.id == agent.id) {
            return Err(format!("Agent with id '{}' already exists", agent.id));
        }
        self.agents.push(agent);
        self.save()
    }
    
    pub fn update(&mut self, agent: CustomAgentConfig) -> Result<(), String> {
        let idx = self.agents.iter().position(|a| a.id == agent.id)
            .ok_or_else(|| format!("Agent '{}' not found", agent.id))?;
        self.agents[idx] = agent;
        self.save()
    }
    
    pub fn delete(&mut self, id: &str) -> Result<(), String> {
        self.agents.retain(|a| a.id != id);
        self.save()
    }
    
    pub fn list(&self) -> &[CustomAgentConfig] {
        &self.agents
    }
    
    pub fn get(&self, id: &str) -> Option<&CustomAgentConfig> {
        self.agents.iter().find(|a| a.id == id)
    }
}
```

### 5.3 自定义智能体适配器

```rust
// aidaguard-plugins/src/adapters/custom_adapter.rs

use crate::{ToolAdapter, PluginManifest};
use crate::custom::CustomAgentConfig;

/// 动态生成的自定义智能体适配器
pub struct CustomAgentAdapter {
    config: CustomAgentConfig,
}

impl CustomAgentAdapter {
    pub fn new(config: CustomAgentConfig) -> Self {
        Self { config }
    }
}

impl ToolAdapter for CustomAgentAdapter {
    fn id(&self) -> &str {
        &self.config.id
    }
    
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn config_path(&self) -> &str {
        // 自定义智能体不修改本地配置，直接使用端点
        ""
    }
    
    fn detect(&self) -> bool {
        // 自定义智能体始终"已安装"
        true
    }
    
    fn current_endpoint(&self) -> Option<String> {
        Some(self.config.endpoint.clone())
    }
    
    fn current_model(&self) -> Option<String> {
        self.config.default_model.clone()
    }
    
    fn configure(&self, _proxy_url: &str) -> Result<(), String> {
        // 自定义智能体不修改本地配置
        // 用户需要手动将代理端点设置到智能体工具中
        Ok(())
    }
    
    fn restore(&self) -> Result<(), String> {
        Ok(())
    }
    
    fn is_configured(&self) -> bool {
        false
    }
    
    fn backup(&self, _backup_dir: &std::path::Path) -> Result<(), String> {
        Ok(())
    }
}

impl Plugin for CustomAgentAdapter {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            id: self.config.id.clone(),
            name: self.config.name.clone(),
            version: "custom".to_string(),
            description: self.config.description.clone().unwrap_or_default(),
            author: "Custom".to_string(),
            config_path_template: String::new(),
            categories: vec!["custom".to_string()],
        }
    }
}
```

### 5.4 文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `aidaguard-plugins/src/custom.rs` | 新增 | CustomAgentConfig, CustomAgentRegistry |
| `aidaguard-plugins/src/adapters/custom_adapter.rs` | 新增 | 自定义智能体适配器 |
| `aidaguard-plugins/src/lib.rs` | 修改 | 导出 custom 模块 |
| `aidaguard-tauri/.../commands/custom_agents.rs` | 新增 | 前端命令接口 |
| `aidaguard-tauri/src-tauri/src/main.rs` | 修改 | 注册新命令 |

---

## 六、验收标准

### 6.1 Phase 4.1 验收

- [ ] 主流大模型提供商接口分析完整
- [ ] TokenUsage 记录功能实现
- [ ] TokenPlan 配额管理实现
- [ ] CodingPlan 计数实现
- [ ] 前端统计展示界面

### 6.2 Phase 4.2 验收

- [ ] 至少 6 个新适配器实现
- [ ] 每个适配器包含完整文档
- [ ] 前端工具列表展示新增工具
- [ ] 配置/备份/恢复功能正常

### 6.3 Phase 4.3 验收

- [ ] 自定义智能体配置 UI
- [ ] 自定义智能体适配器生成
- [ ] 自定义智能体 CRUD 接口
- [ ] 前端自定义配置管理界面

---

## 七、风险与缓解

| 风险 | 等级 | 缓解措施 |
|------|------|----------|
| API 接口变更 | 中 | 版本化适配器，及时更新 |
| 认证方式复杂 | 中 | 文档化常见认证流程 |
| 计费数据准确 | 高 | 对比官方账单验证 |
| 工具兼容性 | 中 | 测试主流版本组合 |

---

## 八、时间规划

| 阶段 | 工作内容 | 预估时间 |
|------|----------|----------|
| 4.1 | 大模型计费统计 | 5-7 天 |
| 4.2 | AI 智能体工具插件化 | 6-8 天 |
| 4.3 | 自定义智能体配置 | 3-4 天 |
| 测试 | 集成测试 + 前端测试 | 2-3 天 |

**总计：** 16-22 天
