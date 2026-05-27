# AIDAGuard 工具适配器架构 V2 — 声明式设计

**参考项目：** [EchoBird](https://github.com/edison7009/EchoBird)  
**文档版本：** v1.0  
**更新日期：** 2026-05-26  
**状态：** 设计阶段

---

## 一、现状分析

### 1.1 当前架构

AIDAGuard 目前有 17 个工具适配器，每个是一个独立的 Rust 文件，实现了 `ToolAdapter` trait：

```
adapters/
├── aider.rs          (175 行)
├── claude_code.rs    (110 行)
├── cline.rs          (232 行)  ← 最复杂：双配置源
├── codeium.rs        (228 行)  ← 双配置源
├── codewhisperer.rs  (237 行)  ← 多文件 + INI 格式
├── codex.rs          (259 行)  ← 最复杂：多格式
├── cody.rs           (201 行)  ← 双配置源
├── continue_dev.rs   (148 行)  ← 数组遍历
├── cursor.rs         (163 行)
├── gemini.rs         (201 行)  ← ENV 文件格式
├── hermes_agent.rs   (173 行)  ← YAML 行编辑
├── jetbrains_ai.rs   (280 行)  ← XML 格式 + 多实例
├── openclaw.rs       (171 行)
├── opencode.rs       (198 行)
├── tabnine.rs        (193 行)  ← 双配置源
├── windsurf.rs       (99 行)   ← STUB
└── zed.rs            (162 行)
```

**总计 ~3,200 行 Rust 代码**，核心逻辑高度重复。

### 1.2 横向对比

| 维度 | 指标 |
|------|------|
| 总适配器数 | 17 |
| 其中 STUB（未实现） | 1（Windsurf） |
| id/manifest 不一致 | 2（Continue `"continue"` vs `"continue_dev"`，Gemini `"gemini"` vs `"gemini_cli"`） |
| config_path/manifest_template 不一致 | 2（OpenCode, OpenClaw） |
| 多文件适配器 | 6（Cline, Codeium, Cody, Tabnine, CodeWhisperer, Gemini CLI） |
| 非 JSON 格式 | 5（Aider YAML, Hermes YAML, Codex TOML, Gemini ENV, JetBrains XML） |
| 自定义格式 | 1（JetBrains XML 字符串编辑） |

### 1.3 14 个适配器遵循同一模式

```
detect()        → 检查目录/文件是否存在
                ↓
is_configured() → current_endpoint() 是否包含 "127.0.0.1" 或 "localhost"
                ↓
backup()        → 复制配置文件到备份目录
                ↓
configure()     → 读取配置 → 设置特定 key = proxy_url → 写回
                ↓
restore()       → 从备份恢复配置文件
```

**差异仅在于：**
1. 配置文件路径不同
2. 检测目录/文件不同
3. 写入的 JSON/YAML/TOML key 名称不同
4. 部分需要写 2 个配置文件

### 1.4 核心问题

| 问题 | 表现 | 影响 |
|------|------|------|
| **样板代码** | 每个适配器 ~150 行重复逻辑 | 新增工具成本高 |
| **硬编码** | 字段名、路径、格式写死在 Rust 代码中 | 修改需重新编译 |
| **id 不一致** | 2 个适配器 id() 与 manifest.id 不同 | 运行时查找错误 |
| **路径不一致** | 2 个 config_path 与 manifest.template 不匹配 | 前端显示错误 |
| **重复的 is_configured** | 13 个适配器实现完全相同的检查逻辑 | 维护成本 |
| **重复的 backup/restore** | 相同的文件复制逻辑 | 无法统一改进 |

---

## 二、设计目标

### 2.1 核心原则

1. **声明式优先** — 简单工具用 JSON 描述，不写 Rust 代码
2. **渐进式复杂** — 复杂工具可用 custom 标记降级为 Rust 实现
3. **统一引擎** — 一个 `DeclarativeAdapter` 处理所有声明式工具
4. **向后兼容** — 保留 `ToolAdapter` trait，两种方式共存

### 2.2 量化目标

| 指标 | 当前 | 目标 |
|------|------|------|
| 适配器 Rust 代码 | ~3,200 行 | ~800 行（引擎 + 仅 complex 适配器） |
| 新增简单工具成本 | ~100 行 Rust | 1 个 JSON manifest 文件 |
| 简单/复杂工具比 | 0:17 | 10:7（manifest-only : manifest + Rust） |
| id 不一致 bug | 2 个 | 0 个 |
| 编译时配置变更 | 需要 | 不需要（modify manifest JSON, restart） |

---

## 三、Manifest Schema 设计

### 3.1 完整 Schema

```json
{
  // ── 元信息（原 PluginManifest）──
  "id": "cline",
  "name": "Cline",
  "version": "1.0.0",
  "description": "Autonomous coding agent (cline/cline)",
  "author": "Cline",
  "categories": ["cli-tool", "vscode-extension", "openai-compatible"],

  // ── 检测 ──
  "detect": {
    "type": "dir_exists",
    "path": "~/.cline"
  },

  // ── 配置读写（主文件）──
  "config": {
    "path": "~/.cline/data/globalState.json",
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
    }
  },

  // ── 辅助配置（可选，如 VS Code settings.json）──
  "secondaryConfigs": [
    {
      "path": "~/Library/Application Support/Code/User/settings.json",
      "format": "json",
      "endpoint": {
        "read": ["cline.openAiBaseUrl", "cline.anthropicBaseUrl"],
        "write": {
          "cline.openAiBaseUrl": "proxyUrl",
          "cline.anthropicBaseUrl": "proxyUrl"
        }
      }
    }
  ],

  // ── 自定义标记 ──
  "custom": false
}
```

### 3.2 字段说明

#### `detect` — 检测方式

| type | 说明 | 示例 |
|------|------|------|
| `dir_exists` | 目录存在即认为已安装 | `"~/.cline"` |
| `file_exists` | 文件存在即认为已安装 | `"~/.continue/config.json"` |
| `any_file_exists` | 任一文件存在 | `["~/.codeium/config.json"]` |
| `dir_has_prefix` | 目录下存在指定前缀的文件 | `{"dir":"~/.vscode/extensions","prefix":"codeium."}` |
| `custom` | 自定义检测逻辑 | 在 Rust 中实现 |

#### `config` — 主配置文件

| 字段 | 类型 | 说明 |
|------|------|------|
| `path` | String | 配置文件路径，支持 `~` 展开 |
| `format` | `"json"` \| `"toml"` \| `"yaml"` \| `"env"` | 配置格式 |
| `endpoint.read` | `String[]` | 候选 JSON 路径列表，按顺序尝试，取第一个存在的 |
| `endpoint.write` | `{path: value}` | 写入映射。value 为 `"proxyUrl"` 时写入代理地址；为空字符串时删除该键 |
| `model.read` | `String[]` | 模型字段的候选路径 |
| `isConfigured` | `{path: String}` | 可选，用于判断"已配置"状态的检查路径。默认使用 endpoint.read |

#### `secondaryConfigs` — 辅助配置文件

数组，结构同 `config`。用于 Cline/Codeium/Cody/Tabnine 等需要同时写入 VS Code settings 的工具。

#### `custom` — 自定义标记

| 值 | 说明 |
|----|------|
| `false` | 纯声明式，由引擎处理（10 个简单适配器） |
| `true` | 需要自定义 Rust 逻辑（7 个复杂适配器） |

### 3.3 简单工具示例

#### 示例 1：Cursor（最简 JSON 适配器）

```json
{
  "id": "cursor",
  "name": "Cursor",
  "description": "AI-first code editor",
  "author": "Cursor / Anysphere",
  "version": "1.0.0",
  "categories": ["editor", "openai-compatible"],
  "detect": { "type": "file_exists", "path": "~/Library/Application Support/Cursor/User/settings.json" },
  "config": {
    "path": "~/Library/Application Support/Cursor/User/settings.json",
    "format": "json",
    "endpoint": {
      "read": ["cursor.apiBase", "openai.baseUrl"],
      "write": {
        "cursor.apiBase": "proxyUrl",
        "openai.baseUrl": "proxyUrl"
      }
    },
    "model": {
      "read": ["cursor.model", "openai.model"]
    }
  },
  "custom": false
}
```

#### 示例 2：Claude Code（嵌套 env 路径）

```json
{
  "id": "claude_code",
  "name": "Claude Code",
  "description": "Anthropic's official CLI agent for Claude",
  "author": "Anthropic",
  "version": "1.0.0",
  "categories": ["cli-tool", "anthropic-compatible"],
  "detect": { "type": "dir_exists", "path": "~/.claude" },
  "config": {
    "path": "~/.claude/settings.json",
    "format": "json",
    "endpoint": {
      "read": ["env.ANTHROPIC_BASE_URL"],
      "readEnvFallback": "ANTHROPIC_BASE_URL",
      "write": {
        "env.ANTHROPIC_BASE_URL": "proxyUrl"
      }
    },
    "model": {
      "read": ["env.ANTHROPIC_MODEL", "env.ANTHROPIC_DEFAULT_OPUS_MODEL", "env.ANTHROPIC_DEFAULT_SONNET_MODEL"],
      "readEnvFallback": "ANTHROPIC_MODEL"
    }
  },
  "custom": false
}
```

#### 示例 3：Continue（遍历数组）

```json
{
  "id": "continue",
  "name": "Continue",
  "description": "Open-source AI code assistant",
  "author": "Continue Dev",
  "version": "1.0.0",
  "categories": ["vscode-extension", "openai-compatible"],
  "detect": { "type": "file_exists", "path": "~/.continue/config.json" },
  "config": {
    "path": "~/.continue/config.json",
    "format": "json",
    "endpoint": {
      "read": ["models.0.apiBase"],
      "write": {
        "models.*.apiBase": "proxyUrl"
      }
    },
    "model": {
      "read": ["models.0.model"]
    }
  },
  "custom": false
}
```

> `models.*.apiBase` 中的 `*` 是通配符，表示遍历数组所有元素。

### 3.4 需要 custom: true 的工具

| 工具 | 复杂度 | 自定义原因 |
|------|--------|-----------|
| JetBrains AI | XML | XML 格式，多 IDE 实例，PROXY_HOST/PROXY_PORT 不同于其他工具 |
| Gemini CLI | ENV | .env KEY=VALUE 格式，非标准配置 |
| Codex | 多格式 | 同一工具支持 JSON/YAML/TOML 三种格式的配置文件 |
| Aider | YAML | YAML 行编辑（非结构化解析），特定缩进规则 |
| Hermes Agent | YAML | YAML 行编辑 |
| CodeWhisperer | INI + multi | AWS credentials INI 格式 + 多个配置文件 |
| Windsurf | STUB | 当前未实现，保留占位 |

---

## 四、DeclarativeAdapter 引擎设计

### 4.1 架构图

```
┌─────────────────────────────────────────────────────┐
│                  PluginRegistry                      │
│                                                     │
│  Vec<Box<dyn Plugin>>                               │
│    ├── DeclarativeAdapter(cursor.json)   ← 引擎实例  │
│    ├── DeclarativeAdapter(cline.json)               │
│    ├── DeclarativeAdapter(zed.json)                 │
│    ├── ... (10 个声明式)                             │
│    ├── JetBrainsAI         ← 传统 Rust 适配器        │
│    ├── Codex               ← 传统 Rust 适配器        │
│    └── ... (7 个 custom)                            │
└─────────────────────────────────────────────────────┘
```

### 4.2 DeclarativeAdapter 结构

```rust
/// 从 JSON manifest 文件加载的声明式适配器
pub struct DeclarativeAdapter {
    manifest: ToolManifest,      // 解析后的 manifest
}

/// Manifest JSON 的 Rust 表示
pub struct ToolManifest {
    // 元信息
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub categories: Vec<String>,

    // 检测
    pub detect: DetectConfig,

    // 主配置
    pub config: FileConfig,

    // 辅助配置
    pub secondary_configs: Vec<FileConfig>,

    // 是否自定义
    pub custom: bool,
}

pub struct FileConfig {
    pub path: String,                       // 配置文件路径模板
    pub format: ConfigFormat,               // json | toml | yaml | env
    pub endpoint: Option<ReadWriteConfig>,  // 端点读写映射
    pub model: Option<ReadConfig>,          // 模型读写映射
}

pub struct ReadWriteConfig {
    pub read: Vec<String>,                   // 读取候选路径
    pub write: HashMap<String, WriteValue>,  // 写入映射
    pub read_env_fallback: Option<String>,   // 环境变量 fallback
}

pub enum WriteValue {
    ProxyUrl,              // 写入代理地址
    StaticString(String),  // 写入固定值
    Delete,                // 删除该键（对应 "" ）
}
```

### 4.3 核心方法实现（伪代码）

#### detect()

```rust
fn detect(&self) -> bool {
    match &self.manifest.detect {
        DetectConfig::DirExists { path } => expand_path(path).exists(),
        DetectConfig::FileExists { path } => expand_path(path).exists(),
        DetectConfig::AnyFileExists { paths } =>
            paths.iter().any(|p| expand_path(p).exists()),
        DetectConfig::DirHasPrefix { dir, prefix } =>
            read_dir(expand_path(dir))
                .map(|entries| entries.any(|e| e.starts_with(prefix)))
                .unwrap_or(false),
    }
}
```

#### current_endpoint()

```rust
fn current_endpoint(&self) -> Option<String> {
    // 按优先级：secondary configs → primary config → env fallback
    for cfg in self.all_configs() {
        if let Some(ref rw) = cfg.endpoint {
            let json = read_config(cfg);
            for path in &rw.read {
                if let Some(val) = json.pointer(path).as_str() {
                    return Some(val.to_string());
                }
            }
        }
    }
    // 环境变量 fallback
    self.read_env_fallback()
}
```

#### is_configured()

```rust
fn is_configured(&self) -> bool {
    self.current_endpoint()
        .map(|ep| ep.contains("127.0.0.1") || ep.contains("localhost"))
        .unwrap_or(false)
}
```

#### configure()

```rust
fn configure(&self, proxy_url: &str) -> Result<(), String> {
    for cfg in self.all_configs() {
        let mut json = read_or_create_config(cfg)?;
        if let Some(ref rw) = cfg.endpoint {
            for (path, value) in &rw.write {
                match value {
                    WriteValue::ProxyUrl =>
                        json_set(&mut json, path, proxy_url),
                    WriteValue::StaticString(s) =>
                        json_set(&mut json, path, s),
                    WriteValue::Delete =>
                        json_remove(&mut json, path),
                }
            }
        }
        write_config(cfg, &json)?;
    }
    Ok(())
}
```

#### backup()

```rust
fn backup(&self, backup_dir: &Path) -> Result<(), String> {
    for cfg in self.all_configs() {
        let path = expand_path(&cfg.path);
        if path.exists() {
            backup_config(&path, backup_dir)?;
        }
    }
    Ok(())
}
```

#### restore()

```rust
fn restore(&self, backup_dir: &Path) -> Result<(), String> {
    for cfg in self.all_configs() {
        let path = expand_path(&cfg.path);
        restore_config(&path, backup_dir)?;
    }
    Ok(())
}
```

### 4.4 JSON 路径解析

引擎需要实现一个通用的 JSON 路径读写器：

| 路径语法 | 含义 | 示例 |
|----------|------|------|
| `key` | 顶层键 | `openAiBaseUrl` |
| `a.b.c` | 嵌套键 | `env.ANTHROPIC_BASE_URL` |
| `a.*.c` | 遍历数组/对象 | `models.*.apiBase` |
| `a.0.c` | 数组索引 | `models.0.model` |

**读取时**：按顺序尝试 `read` 中的候选路径，返回第一个非空值。

**写入时**：自动创建不存在的中间层级（如 `env.ANTHROPIC_BASE_URL` 会先创建 `env: {}` 对象）。

---

## 五、文件结构

### 5.1 新目录布局

```
crates/aidaguard-plugins/
├── src/
│   ├── lib.rs                        # ToolAdapter trait + ToolInfo（不变）
│   ├── registry.rs                   # PluginRegistry（不变）
│   ├── backup.rs                     # 备份/还原（不变）
│   ├── declarative/
│   │   ├── mod.rs                    # 模块导出
│   │   ├── engine.rs                 # DeclarativeAdapter 引擎实现
│   │   ├── manifest.rs               # ToolManifest 结构体 + 反序列化
│   │   ├── json_path.rs              # JSON 路径读写器
│   │   └── format.rs                 # 多格式支持（JSON/TOML/YAML/ENV）
│   └── adapters/
│       ├── mod.rs                    # register_all() — 加载 manifest + complex 适配器
│       ├── complex/                  # 仅保留需要 custom: true 的适配器
│       │   ├── mod.rs
│       │   ├── jetbrains_ai.rs       # XML + 多实例
│       │   ├── codex.rs              # 多格式
│       │   ├── aider.rs              # YAML 行编辑
│       │   ├── hermes_agent.rs       # YAML 行编辑
│       │   ├── gemini.rs             # ENV 格式
│       │   ├── codewhisperer.rs      # INI + 多文件
│       │   └── windsurf.rs           # STUB
│       └── manifests/               # 10 个声明式 manifest JSON 文件
│           ├── cursor.json
│           ├── cline.json
│           ├── claude_code.json
│           ├── codeium.json
│           ├── cody.json
│           ├── continue.json
│           ├── openclaw.json
│           ├── opencode.json
│           ├── tabnine.json
│           └── zed.json
```

### 5.2 代码量估算

| 模块 | 类型 | 估算行数 |
|------|------|----------|
| `declarative/engine.rs` | 新增 | ~200 行 |
| `declarative/manifest.rs` | 新增 | ~80 行 |
| `declarative/json_path.rs` | 新增 | ~120 行 |
| `declarative/format.rs` | 新增 | ~60 行 |
| `adapters/manifests/*.json` | 新增 | ~250 行（10 文件 × ~25 行） |
| `adapters/complex/*.rs` | 保留 | ~1,500 行（7 个） |
| `adapters/mod.rs` | 重写 | ~40 行 |
| **总计** | | **~2,250 行**（vs 当前 ~3,200 行） |

---

## 六、实现计划

### Phase 1: 基础设施（不删除任何现有代码）

1. 创建 `declarative/` 模块骨架
2. 实现 `manifest.rs` — `ToolManifest` 结构体 + serde 反序列化
3. 实现 `json_path.rs` — JSON 路径读写器（含 `*` 通配符）
4. 实现 `format.rs` — ConfigFormat 枚举 + 解析/序列化
5. 实现 `engine.rs` — `DeclarativeAdapter` struct + `ToolAdapter` trait impl
6. 单元测试

### Phase 2: Manifest 文件 + 注册

7. 为 10 个简单适配器编写 manifest JSON 文件
8. 重写 `adapters/mod.rs` 的 `register_all()`：
   - 优先尝试从 manifest JSON 创建 `DeclarativeAdapter`
   - 保留 7 个 complex 适配器的直接注册
9. 端到端测试（detect, configure, restore）

### Phase 3: 清理

10. 删除 10 个已声明式化的旧 `.rs` 适配器文件
11. 更新 `AI_TOOLS_ADAPTERS.md` 和 `ARCHITECTURE.md`
12. 更新前端（如有必要）

---

## 七、风险与注意事项

### 7.1 Cline 的特殊性

Cline 的 `restore()` 不直接恢复 VS Code settings.json（会丢失其他扩展的配置），而是移除 `cline.*` 键。这种"选择性清理"逻辑需要特殊处理：在 manifest 的 `secondaryConfigs[0]` 中添加 `restoreMode: "removeKeys"` 标记。

### 7.2 JSON 路径 `*` 通配符

Continue 的 `models.*.apiBase` 和 OpenClaw/OpenCode 的 provider 遍历需要通配符支持。实现时需区分：
- 数组遍历（`models.*.apiBase`）— 遍历数组每个元素
- 对象遍历（`provider.*.options.baseURL`）— 遍历对象每个 value

### 7.3 环境变量 fallback

Claude Code 和 Gemini CLI 可通过环境变量配置。引擎需要支持 `readEnvFallback` 字段，在文件读取失败时 fallback 到环境变量。

### 7.4 is_configured 的 CodeWhisperer 特例

CodeWhisperer 检查 `http.proxy` 键是否存在（不检查值内容），与其他适配器的 `contains("127.0.0.1")` 不同。保留为 custom 适配器可避免引擎复杂化。

---

## 八、参考

- [EchoBird 工具配置参考](./ECHOBIRD_TOOLS_REFERENCE.md)
- [AI 工具适配器分析（当前版本）](./AI_TOOLS_ADAPTERS.md)
- [AIDAGuard 架构文档](./ARCHITECTURE.md)
