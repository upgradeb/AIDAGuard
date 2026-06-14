# 自定义 AI 智能体配置设计

**文档版本：** v1.0  
**更新日期：** 2026-05-16  
**用途：** AIDAGuard Phase 4.3 技术参考

---

## 一、需求背景

用户可能使用不在预设列表中的 AI 智能体工具，需要提供自定义配置能力：

1. **自建 API 服务** - 用户自建的 LLM 服务
2. **私有部署模型** - 企业内部部署的模型
3. **新兴工具** - 新出现的 AI 工具尚未支持
4. **实验性工具** - 开发中的实验性工具

---

## 二、功能设计

### 2.1 核心功能

| 功能 | 说明 |
|------|------|
| 自定义端点 | 用户指定 API 端点 URL |
| 协议选择 | OpenAI / Anthropic / 自定义 |
| 认证配置 | Bearer Token / API Key Header / 自定义 |
| 模型配置 | 模型 ID、上下文窗口、最大输出等 |
| 请求模板 | 自定义请求体格式（高级） |
| 配额管理 | Token 限制、预算控制 |

### 2.2 配置界面

```
┌─────────────────────────────────────────────────────────────┐
│  添加自定义智能体                                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  基本信息                                                    │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 名称: [________________________]                     │   │
│  │ 描述: [________________________]                     │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  API 配置                                                    │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 端点: [https://api.example.com/v1____________]      │   │
│  │ 协议: ○ OpenAI 兼容  ○ Anthropic 兼容  ○ 自定义     │   │
│  │ 认证: ○ Bearer Token  ○ API Key Header  ○ 无       │   │
│  │ API Key: [sk-xxx________________________________]   │   │
│  │ Header 名称: [X-API-Key________] (可选)             │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  模型配置                                                    │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 默认模型: [model-id_____________________]           │   │
│  │ 上下文窗口: [____4096____] tokens                   │   │
│  │ 最大输出:   [____2048____] tokens                   │   │
│  │ + 添加更多模型                                       │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  高级配置 (可选)                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 请求超时: [____60____] 秒                           │   │
│  │ 最大重试: [____3_____] 次                           │   │
│  │ □ 自定义请求模板                                    │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  配额管理 (可选)                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ □ 启用每日限制  [______] tokens                     │   │
│  │ □ 启用每月限制  [______] tokens                     │   │
│  │ □ 启用预算控制  $[______]                           │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  [取消]  [测试连接]  [保存]                                 │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 三、数据结构设计

### 3.1 自定义智能体配置

```rust
/// 自定义智能体配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomAgentConfig {
    // ── 基本信息 ──
    
    /// 唯一标识（自动生成或用户指定）
    pub id: String,
    
    /// 显示名称
    pub name: String,
    
    /// 描述（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    // ── API 配置 ──
    
    /// API 端点
    pub endpoint: String,
    
    /// API 协议
    pub protocol: ProtocolType,
    
    /// 认证方式
    pub auth_type: AuthType,
    
    /// API Key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    
    /// 自定义 Header 名称（用于 ApiKeyHeader 认证）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_header: Option<String>,
    
    // ── 模型配置 ──
    
    /// 默认模型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
    
    /// 模型列表
    #[serde(default)]
    pub models: Vec<CustomModel>,
    
    // ── 请求配置 ──
    
    /// 请求超时（秒）
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    
    /// 最大重试次数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<u32>,
    
    // ── 高级配置 ──
    
    /// 自定义请求模板
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_template: Option<RequestTemplate>,
    
    // ── 配额管理 ──
    
    /// Token 配额
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_quota: Option<TokenQuota>,
    
    // ── 元数据 ──
    
    /// 创建时间
    pub created_at: i64,
    
    /// 更新时间
    pub updated_at: i64,
    
    /// 是否启用
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_timeout() -> u64 { 60 }
fn default_true() -> bool { true }
```

### 3.2 自定义模型

```rust
/// 自定义模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomModel {
    /// 模型 ID
    pub id: String,
    
    /// 显示名称
    pub name: String,
    
    /// 上下文窗口大小
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window: Option<usize>,
    
    /// 最大输出 tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output: Option<usize>,
    
    /// 能力标签
    #[serde(default)]
    pub capabilities: Vec<String>,
    
    /// 输入价格（美元/1M tokens）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_price_usd: Option<f64>,
    
    /// 输出价格（美元/1M tokens）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_price_usd: Option<f64>,
}
```

### 3.3 请求模板

```rust
/// 自定义请求模板（高级功能）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestTemplate {
    /// 请求体模板（JSON 字符串，支持变量替换）
    /// 
    /// 支持变量:
    /// - {{model}} - 模型 ID
    /// - {{messages}} - 消息数组
    /// - {{max_tokens}} - 最大输出
    /// - {{temperature}} - 温度参数
    /// - {{stream}} - 是否流式
    /// 
    /// 示例:
    /// ```json
    /// {
    ///   "model": "{{model}}",
    ///   "input": {{messages}},
    ///   "config": {
    ///     "max_tokens": {{max_tokens}},
    ///     "temperature": {{temperature}}
    ///   }
    /// }
    /// ```
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_template: Option<String>,
    
    /// 自定义请求头
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    
    /// 响应解析器（JSONPath 表达式）
    /// 
    /// 示例:
    /// - `$.result.content` - 提取 result.content 字段
    /// - `$.data[0].text` - 提取数组第一个元素的 text 字段
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_parser: Option<String>,
    
    /// Token 使用量解析器
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_parser: Option<UsageParser>,
}

/// Token 使用量解析配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageParser {
    /// Prompt tokens 字段路径
    pub prompt_tokens_path: String,
    
    /// Completion tokens 字段路径
    pub completion_tokens_path: String,
    
    /// Total tokens 字段路径（可选，可计算）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens_path: Option<String>,
}
```

### 3.4 Token 配额

```rust
/// Token 配额配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenQuota {
    /// 每日限制（tokens）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily_limit: Option<u64>,
    
    /// 每月限制（tokens）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_limit: Option<u64>,
    
    /// 预算限制（美元）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_usd: Option<f64>,
    
    /// 警告阈值（百分比，如 80 表示 80%）
    #[serde(default = "default_warning_threshold")]
    pub warning_threshold: u8,
    
    /// 超限行为
    #[serde(default)]
    pub on_exceed: ExceedBehavior,
}

fn default_warning_threshold() -> u8 { 80 }

/// 超限行为
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExceedBehavior {
    #[default]
    Warn,      // 仅警告
    Block,     // 阻止请求
    Fallback,  // 回退到其他端点
}
```

---

## 四、存储设计

### 4.1 配置文件

```json
// ~/.aidaguard/custom_agents.json

{
  "version": 1,
  "agents": [
    {
      "id": "custom-001",
      "name": "我的私有模型",
      "description": "企业内部部署的 LLM 服务",
      "endpoint": "https://llm.internal.company.com/v1",
      "protocol": "openai_compatible",
      "authType": "bearer_token",
      "apiKey": "encrypted:xxx",
      "defaultModel": "company-llm-v1",
      "models": [
        {
          "id": "company-llm-v1",
          "name": "Company LLM v1",
          "contextWindow": 8192,
          "maxOutput": 2048
        }
      ],
      "timeoutSecs": 120,
      "tokenQuota": {
        "dailyLimit": 100000,
        "budgetUsd": 10.0,
        "warningThreshold": 80,
        "onExceed": "warn"
      },
      "createdAt": 1715817600000,
      "updatedAt": 1715817600000,
      "enabled": true
    }
  ]
}
```

### 4.2 注册表实现

```rust
/// 自定义智能体注册表
pub struct CustomAgentRegistry {
    /// 配置列表
    agents: Vec<CustomAgentConfig>,
    
    /// 配置文件路径
    config_path: PathBuf,
    
    /// 加密器（用于 API Key 加密）
    encryptor: Option<AesGcmEncryptor>,
}

impl CustomAgentRegistry {
    /// 创建注册表
    pub fn new(config_dir: PathBuf, encryptor: Option<AesGcmEncryptor>) -> Self {
        let config_path = config_dir.join("custom_agents.json");
        let mut registry = Self {
            agents: Vec::new(),
            config_path,
            encryptor,
        };
        registry.load();
        registry
    }
    
    /// 从文件加载
    fn load(&mut self) {
        if !self.config_path.exists() {
            return;
        }
        
        let content = match std::fs::read_to_string(&self.config_path) {
            Ok(c) => c,
            Err(_) => return,
        };
        
        let file: CustomAgentsFile = match serde_json::from_str(&content) {
            Ok(f) => f,
            Err(_) => return,
        };
        
        // 解密 API Key
        self.agents = file.agents.into_iter()
            .map(|mut agent| {
                if let Some(ref encryptor) = self.encryptor {
                    if let Some(ref encrypted) = agent.api_key {
                        if encrypted.starts_with("encrypted:") {
                            let encrypted_part = encrypted.strip_prefix("encrypted:").unwrap();
                            agent.api_key = encryptor.decrypt(encrypted_part).ok();
                        }
                    }
                }
                agent
            })
            .collect();
    }
    
    /// 保存到文件
    fn save(&self) -> Result<(), String> {
        let mut agents = self.agents.clone();
        
        // 加密 API Key
        if let Some(ref encryptor) = self.encryptor {
            for agent in &mut agents {
                if let Some(ref key) = agent.api_key {
                    agent.api_key = Some(format!("encrypted:{}", encryptor.encrypt(key)?));
                }
            }
        }
        
        let file = CustomAgentsFile {
            version: 1,
            agents,
        };
        
        let content = serde_json::to_string_pretty(&file)
            .map_err(|e| e.to_string())?;
        
        std::fs::write(&self.config_path, content)
            .map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    /// 添加自定义智能体
    pub fn add(&mut self, mut agent: CustomAgentConfig) -> Result<(), String> {
        // 验证
        self.validate(&agent)?;
        
        // 生成 ID
        if agent.id.is_empty() {
            agent.id = format!("custom-{}", uuid::Uuid::new_v4());
        }
        
        // 设置时间戳
        let now = chrono::Utc::now().timestamp_millis();
        agent.created_at = now;
        agent.updated_at = now;
        
        // 检查重复
        if self.agents.iter().any(|a| a.id == agent.id) {
            return Err(format!("Agent with id '{}' already exists", agent.id));
        }
        
        self.agents.push(agent);
        self.save()
    }
    
    /// 更新自定义智能体
    pub fn update(&mut self, mut agent: CustomAgentConfig) -> Result<(), String> {
        self.validate(&agent)?;
        
        let idx = self.agents.iter().position(|a| a.id == agent.id)
            .ok_or_else(|| format!("Agent '{}' not found", agent.id))?;
        
        agent.updated_at = chrono::Utc::now().timestamp_millis();
        agent.created_at = self.agents[idx].created_at;
        
        self.agents[idx] = agent;
        self.save()
    }
    
    /// 删除自定义智能体
    pub fn delete(&mut self, id: &str) -> Result<(), String> {
        self.agents.retain(|a| a.id != id);
        self.save()
    }
    
    /// 列出所有
    pub fn list(&self) -> &[CustomAgentConfig] {
        &self.agents
    }
    
    /// 获取单个
    pub fn get(&self, id: &str) -> Option<&CustomAgentConfig> {
        self.agents.iter().find(|a| a.id == id)
    }
    
    /// 验证配置
    fn validate(&self, agent: &CustomAgentConfig) -> Result<(), String> {
        // 验证端点
        if agent.endpoint.is_empty() {
            return Err("Endpoint is required".to_string());
        }
        
        if !agent.endpoint.starts_with("http://") && !agent.endpoint.starts_with("https://") {
            return Err("Endpoint must be a valid URL".to_string());
        }
        
        // 验证名称
        if agent.name.is_empty() {
            return Err("Name is required".to_string());
        }
        
        // 验证 API Key（如果需要认证）
        if agent.auth_type != AuthType::None && agent.api_key.is_none() {
            return Err("API Key is required for selected auth type".to_string());
        }
        
        Ok(())
    }
    
    /// 测试连接
    pub async fn test_connection(&self, agent: &CustomAgentConfig) -> Result<TestResult, String> {
        // 构建测试请求
        let client = reqwest::Client::new();
        
        let mut request = match agent.protocol {
            ProtocolType::OpenAiCompatible => {
                client
                    .post(format!("{}/chat/completions", agent.endpoint))
                    .json(&json!({
                        "model": agent.default_model.as_deref().unwrap_or("gpt-3.5-turbo"),
                        "messages": [{"role": "user", "content": "Hello"}],
                        "max_tokens": 10
                    }))
            }
            ProtocolType::AnthropicCompatible => {
                client
                    .post(format!("{}/messages", agent.endpoint))
                    .header("anthropic-version", "2023-06-01")
                    .json(&json!({
                        "model": agent.default_model.as_deref().unwrap_or("claude-3-haiku-20240307"),
                        "max_tokens": 10,
                        "messages": [{"role": "user", "content": "Hello"}]
                    }))
            }
        };
        
        // 添加认证
        request = match &agent.auth_type {
            AuthType::BearerToken => {
                request.bearer_auth(agent.api_key.as_deref().unwrap_or(""))
            }
            AuthType::ApiKeyHeader { header } => {
                request.header(header, agent.api_key.as_deref().unwrap_or(""))
            }
            AuthType::None => request,
        };
        
        // 发送请求
        let response = request
            .timeout(std::time::Duration::from_secs(agent.timeout_secs))
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        
        let status = response.status();
        let latency_ms = 0; // TODO: 计算延迟
        
        Ok(TestResult {
            success: status.is_success(),
            status_code: status.as_u16(),
            message: if status.is_success() {
                "Connection successful".to_string()
            } else {
                format!("HTTP {}", status)
            },
            latency_ms,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CustomAgentsFile {
    version: u32,
    agents: Vec<CustomAgentConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestResult {
    pub success: bool,
    pub status_code: u16,
    pub message: String,
    pub latency_ms: u64,
}
```

---

## 五、前端接口设计

### 5.1 Tauri 命令

```rust
// crates/aidaguard-tauri/src-tauri/src/commands/custom_agents.rs

use tauri::State;
use crate::state::AppState;
use aidaguard_plugins::custom::{CustomAgentConfig, TestResult};

#[tauri::command]
pub async fn list_custom_agents(
    state: State<'_, AppState>,
) -> Result<Vec<CustomAgentConfig>, String> {
    let registry = state.custom_agents.lock().await;
    Ok(registry.list().to_vec())
}

#[tauri::command]
pub async fn get_custom_agent(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<CustomAgentConfig>, String> {
    let registry = state.custom_agents.lock().await;
    Ok(registry.get(&id).cloned())
}

#[tauri::command]
pub async fn add_custom_agent(
    state: State<'_, AppState>,
    agent: CustomAgentConfig,
) -> Result<(), String> {
    let mut registry = state.custom_agents.lock().await;
    registry.add(agent)
}

#[tauri::command]
pub async fn update_custom_agent(
    state: State<'_, AppState>,
    agent: CustomAgentConfig,
) -> Result<(), String> {
    let mut registry = state.custom_agents.lock().await;
    registry.update(agent)
}

#[tauri::command]
pub async fn delete_custom_agent(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let mut registry = state.custom_agents.lock().await;
    registry.delete(&id)
}

#[tauri::command]
pub async fn test_custom_agent(
    state: State<'_, AppState>,
    agent: CustomAgentConfig,
) -> Result<TestResult, String> {
    let registry = state.custom_agents.lock().await;
    registry.test_connection(&agent).await
}
```

### 5.2 前端 API

```typescript
// src/api/customAgents.ts

import { invoke } from '@tauri-apps/api/core';

export interface CustomAgentConfig {
  id: string;
  name: string;
  description?: string;
  endpoint: string;
  protocol: 'openai_compatible' | 'anthropic_compatible';
  authType: 'bearer_token' | 'api_key_header' | 'none';
  apiKey?: string;
  authHeader?: string;
  defaultModel?: string;
  models: CustomModel[];
  timeoutSecs: number;
  maxRetries?: number;
  requestTemplate?: RequestTemplate;
  tokenQuota?: TokenQuota;
  createdAt: number;
  updatedAt: number;
  enabled: boolean;
}

export async function listCustomAgents(): Promise<CustomAgentConfig[]> {
  return invoke('list_custom_agents');
}

export async function getCustomAgent(id: string): Promise<CustomAgentConfig | null> {
  return invoke('get_custom_agent', { id });
}

export async function addCustomAgent(agent: CustomAgentConfig): Promise<void> {
  return invoke('add_custom_agent', { agent });
}

export async function updateCustomAgent(agent: CustomAgentConfig): Promise<void> {
  return invoke('update_custom_agent', { agent });
}

export async function deleteCustomAgent(id: string): Promise<void> {
  return invoke('delete_custom_agent', { id });
}

export async function testCustomAgent(agent: CustomAgentConfig): Promise<TestResult> {
  return invoke('test_custom_agent', { agent });
}
```

---

## 六、使用流程

### 6.1 添加自定义智能体

1. 用户打开"工具配置"页面
2. 点击"添加自定义智能体"
3. 填写基本信息（名称、描述）
4. 配置 API（端点、协议、认证）
5. 配置模型（默认模型、模型列表）
6. （可选）配置高级选项
7. （可选）配置配额管理
8. 点击"测试连接"验证配置
9. 点击"保存"完成添加

### 6.2 使用自定义智能体

添加后，自定义智能体会出现在工具列表中：

1. 在"工具配置"页面看到自定义智能体
2. 点击"配置代理"设置 AIDAGuard 代理
3. 在智能体工具中设置端点为 AIDAGuard 代理地址
4. 开始使用

### 6.3 管理配额

1. 在自定义智能体详情中查看使用统计
2. 设置每日/每月限制
3. 设置预算控制
4. 接近限制时收到警告
5. 超限后根据配置行为处理

---

## 七、安全考虑

### 7.1 API Key 加密

- API Key 在存储前加密
- 使用与审计存储相同的加密密钥
- 显示时脱敏（仅显示前 4 位和后 4 位）

### 7.2 输入验证

- 端点 URL 格式验证
- API Key 格式验证（可选）
- 配额值范围验证

### 7.3 权限控制

- 只有管理员可以添加/修改自定义智能体
- 敏感操作需要确认

---

## 八、扩展性

### 8.1 自定义请求模板

对于非标准 API，用户可以定义：

```json
{
  "bodyTemplate": {
    "model": "{{model}}",
    "input": {{messages}},
    "parameters": {
      "max_new_tokens": {{max_tokens}},
      "temperature": {{temperature}}
    }
  },
  "responseParser": "$.generated_text",
  "usageParser": {
    "promptTokensPath": "$.usage.prompt_tokens",
    "completionTokensPath": "$.usage.completion_tokens"
  }
}
```

### 8.2 插件系统

未来可以支持：

- 导入/导出自定义智能体配置
- 分享配置模板
- 社区贡献的配置库

---

## 九、测试清单

- [ ] 添加自定义智能体
- [ ] 编辑自定义智能体
- [ ] 删除自定义智能体
- [ ] 测试连接功能
- [ ] API Key 加密/解密
- [ ] 配额统计
- [ ] 超限警告
- [ ] 自定义请求模板
- [ ] 导入/导出配置
