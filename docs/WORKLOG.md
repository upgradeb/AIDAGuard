# Aidaguard 开发工作记录

**版本：** 0.5.0

## 项目概述

Aidaguard 是一个本地 LLM API 代理，部署在 AI 客户端与大模型 API 之间，自动检测请求中的敏感数据（手机号、身份证、银行卡等），替换为占位符后再转发给大模型，并在响应中将占位符还原为原始数据。同时提供 Tauri 2.x 桌面应用进行可视化管理。

**技术栈：**
- 后端：Rust, Tokio, Axum 0.7, Reqwest 0.12
- 前端：Tauri 2.x, React 18, TypeScript, shadcn/ui, Tailwind CSS, Zustand

---

## Git 提交历史

| 提交 | 描述 |
|------|------|
| `5ba1c7b` | feat: migrate frontend to shadcn/ui + Tailwind CSS |
| `bab11dc` | fix: use aws.proxy for CodeWhisperer proxy detection |
| `e4276b4` | feat: declarative tool adapter engine + 31 EchoBird-aligned adapters |
| `218e9b0` | feat: dual theme presets, Cline/Roo Code adapters, and bug fixes |
| `d6b9e92` | feat: Phase 4 tool adapters + v0.5.0 |
| `180bf83` | fix: rules_dir resolution, dark theme, collapsible audit body |
| `c47210e` | docs: add Phase 4 planning documents |
| `e0bdef0` | feat: enhance error handling (3.5) |
| `6a401c5` | feat: add plugin dynamic loading support (3.4) |
| `2bdde5a` | feat: enhance DetectionEngine trait (3.3) |
| `c8a02de` | feat: implement MemoryStorage and StorageFactory (3.2) |
| `54619de` | refactor: eliminate reverse dependency - core no longer depends on storage |
| `65240c8` | MVP — 本地 API 网关代理，敏感数据检测与流式占位符还原 |

---

## Phase 4: 工具适配器系统

### 声明式适配器引擎

实现了基于 JSON manifest 的声明式工具适配器引擎：

- 25 个声明式适配器（从 `manifests/*.json` 编译时嵌入）
- 6 个复杂适配器（`aider`, `codex`, `hermes_agent`, `gemini`, `codewhisperer`, `jetbrains_ai`）
- 支持 JSON/YAML/TOML/INI/ENV/XML 等多种配置格式
- JSON 路径查询（`$.key.nested[0]`）

### 支持的工具

| 类别 | 工具 |
|------|------|
| CLI | Claude Code, Aider, Codex, Gemini CLI, OpenClaw, OpenCode |
| IDE | Cursor, Zed, Windsurf, JetBrains AI |
| VS Code 扩展 | Cline, Continue.dev, Codeium, Cody, Tabnine, CodeWhisperer |

---

## Phase 5: UI 现代化

### shadcn/ui + Tailwind CSS 迁移

完成前端 UI 框架升级：

- 从 Ant Design 迁移到 shadcn/ui (Radix UI)
- 使用 Tailwind CSS 替代自定义 CSS
- 添加全局主题支持（浅色/深色/跟随系统）
- 统一组件样式和交互体验

**新增 UI 组件：**
- `button`, `card`, `dialog`, `input`, `select`, `switch`, `table`, `tabs`, `tooltip`, `alert`, `badge`, `checkbox`, `label`, `separator`, `skeleton`, `toggle`

---

## Phase 0: aidaguard-core 改造

### Embedded Proxy 架构

将原本独立运行的代理服务器改造为可嵌入 Tauri 进程的形态：

- `start_with_state(config, detector, storage, event_tx, shutdown_signal)` — 接受外部管理的共享状态和关闭信号，支持优雅关闭
- `start()` 内部调用 `start_with_state` 并传入 `std::future::pending()`（永不主动关闭），向后兼容
- 共享状态通过 `Arc<RwLock<Detector>>` 和 `Arc<Storage>` 在代理任务和 Tauri 命令之间共享

### DetectionEvent 实时事件

- 新增 `DetectionEvent` 结构体：`timestamp_ms, rule_id, strategy, placeholder, request_path, response_status`
- 代理检测到敏感数据后通过 `tokio::sync::broadcast` (容量 256) 广播事件
- Tauri 端 relay 任务将 broadcast 消息转发为 `app_handle.emit("detection-event", payload)` → 前端 `listen()` 接收

### UpstreamConfig 多上游支持

- `Config` 新增 `upstreams: Vec<UpstreamConfig>` 字段
- `UpstreamConfig` 字段：`name, url, api_key, default, timeout_secs, rate_limit_qps, models`
- `Config::save_to(path)` 和 `Config::load_from(path)` 方法用于 Tauri 设置页持久化

---

## Phase 1: Tauri 项目脚手架 + 代理生命周期 + Dashboard

### 后端 (Rust)

| 文件 | 功能 |
|------|------|
| [main.rs](../crates/aidaguard-tauri/src-tauri/src/main.rs) | Tauri 入口：注册插件、初始化状态、构建托盘、启动事件 relay、最小化到托盘 |
| [state.rs](../crates/aidaguard-tauri/src-tauri/src/state.rs) | `AppState` 结构体：config, detector, storage, proxy_handle, proxy_cancel, proxy_start_time |
| [commands/proxy.rs](../crates/aidaguard-tauri/src-tauri/src/commands/proxy.rs) | 代理生命周期命令：`start_proxy`, `stop_proxy`, `get_proxy_status` |
| [events.rs](../crates/aidaguard-tauri/src-tauri/src/events.rs) | 事件 relay：broadcast receiver → Tauri `emit("detection-event")` |
| [tray.rs](../crates/aidaguard-tauri/src-tauri/src/tray.rs) | 系统托盘：空闲/运行/错误状态图标，启动/停止/显示窗口/退出菜单 |

### 前端 (React + TypeScript)

| 文件 | 功能 |
|------|------|
| [types/index.ts](../crates/aidaguard-tauri/src/src/types/index.ts) | TypeScript 接口定义（与 Rust 结构体对齐） |
| [api/](../crates/aidaguard-tauri/src/src/api/) | Tauri `invoke()` 封装：proxy, audit, rules, config, upstream, events |
| [store/](../crates/aidaguard-tauri/src/src/store/) | Zustand 状态管理：useProxyStore, useAuditStore, useRulesStore, useConfigStore, useThemeStore, useUpstreamStore |
| [pages/Dashboard.tsx](../crates/aidaguard-tauri/src/src/pages/Dashboard.tsx) | 仪表盘：统计卡片（今日/本周/总计检测数、数据库大小）、规则命中饼图、实时事件流 |
| [components/StatCard.tsx](../crates/aidaguard-tauri/src/src/components/StatCard.tsx) | 统计卡片组件 |
| [components/RuleHitChart.tsx](../crates/aidaguard-tauri/src/src/components/RuleHitChart.tsx) | 规则命中分布饼图（Recharts） |
| [components/EventFeed.tsx](../crates/aidaguard-tauri/src/src/components/EventFeed.tsx) | 实时检测事件流（虚拟列表，最多 200 条） |

---

## Phase 2: 审计日志

### storage 扩展

- `list_filtered(limit, offset, rule_id, path, date_from, date_to)` — 多条件过滤分页查询
- `count_filtered(rule_id, path, date_from, date_to)` — 过滤条件下的准确计数
- `get_by_id(id)` — 单条查询
- `delete(id)` — 单条删除
- `stats()` — 汇总统计：total_count, today_count, week_count, rule_distribution, db_size_bytes

### 后端命令

| 命令 | 功能 |
|------|------|
| `list_audit` | 分页查询 + 多条件过滤（规则ID、路径、日期范围） |
| `get_audit_detail` | 按 ID 获取检测记录详情 |
| `delete_audit` | 删除单条记录 |
| `export_audit` | 导出为 CSV 文本 |
| `get_audit_stats` | 获取汇总统计数据 |

### 前端

| 文件 | 功能 |
|------|------|
| [pages/AuditLog.tsx](../crates/aidaguard-tauri/src/src/pages/AuditLog.tsx) | 审计日志页：搜索栏 + 表格 + 分页 + 导出按钮 |
| [components/AuditTable.tsx](../crates/aidaguard-tauri/src/src/components/AuditTable.tsx) | 审计记录表格（时间、规则、策略、路径、状态码） |
| [components/AuditDetailPanel.tsx](../crates/aidaguard-tauri/src/src/components/AuditDetailPanel.tsx) | 详情抽屉：原始数据、上下文、占位符、请求路径、替换后 body |

---

## Phase 3: 规则管理

### 后端命令

| 命令 | 功能 |
|------|------|
| `get_rules` | 读取所有 YAML 规则文件，展开为 RuleWithCategory 列表 |
| `save_rule` | 创建或更新规则，写入对应分类 YAML 文件，自动重载检测器 |
| `delete_rule` | 从分类中删除规则，重载检测器 |
| `toggle_rule` | 启用/禁用单条规则 |
| `test_rule` | 正则测试：编译模式 → 搜索匹配 → 执行替换，返回 MatchInfo + sanitized_text |
| `reload_rules` | 手动重载全部规则 |
| `get_rule_files` | 列出所有分类文件名 |

### 前端

| 文件 | 功能 |
|------|------|
| [pages/Rules.tsx](../crates/aidaguard-tauri/src/src/pages/Rules.tsx) | 规则管理页：分类筛选 + 表格 + 新建/编辑/删除/启用开关 |
| [components/RuleEditor.tsx](../crates/aidaguard-tauri/src/src/components/RuleEditor.tsx) | 规则编辑 Modal：ID、名称、正则模式、策略、优先级、测试文本 |
| [components/RuleTestPanel.tsx](../crates/aidaguard-tauri/src/src/components/RuleTestPanel.tsx) | 正则测试面板：显示匹配列表 + 替换后文本预览 |

---

## Phase 4: 设置

### 后端命令

| 命令 | 功能 |
|------|------|
| `get_config` | 读取配置文件并返回完整 Config |
| `save_config` | 保存 Config 到配置文件，应用运行中的更改 |

### 前端

| 文件 | 功能 |
|------|------|
| [pages/Settings.tsx](../crates/aidaguard-tauri/src/src/pages/Settings.tsx) | 设置页：代理设置、存储设置、日志设置、外观（主题切换）、通知、关于 |
| [components/ThemeSwitcher.tsx](../crates/aidaguard-tauri/src/src/components/ThemeSwitcher.tsx) | 明/暗主题切换器 |

---

## Phase 5: 上游管理

### 后端命令

| 命令 | 功能 |
|------|------|
| `test_upstream_connectivity` | 使用 reqwest 直连测试上游连通性，返回状态码和延迟（不含响应体） |

### 前端

| 文件 | 功能 |
|------|------|
| [pages/Upstreams.tsx](../crates/aidaguard-tauri/src/src/pages/Upstreams.tsx) | 上游管理页：列表 + 添加/编辑 Modal + 连通性测试 + 设置默认 |

---

## Phase 6: 通知与收尾

### 桌面通知

- [hooks/useNotification.ts](../crates/aidaguard-tauri/src/src/hooks/useNotification.ts) — Web Notification API，频率限制 1次/规则/分钟，点击导航到审计日志

### 窗口行为

- 关闭窗口 → 最小化到托盘（不退出）
- 托盘菜单"显示窗口" → 恢复并聚焦
- 托盘菜单"退出" → 停止代理 + 退出应用

### Bug 修复

| 问题 | 修复 |
|------|------|
| 前端 `invoke()` 参数名使用 camelCase（如 `ruleIdFilter`） | 改为 snake_case 以匹配 Rust 参数名（Tauri v2 严格要求） |
| `get_proxy_status` 未检测代理任务自行终止 | 添加 `JoinHandle::is_finished()` 检查，自动清空已完成的 handle |
| 审计列表 `total` 未反映过滤条件 | 新增 `count_filtered()` 替换 `count()` |

---

## Phase 7: 启动测试

### 环境验证
- Tauri CLI (`cargo-tauri`) 已安装
- Node.js v24.14.1 / npm 11.11.0
- Rust toolchain 就绪
- 前端依赖安装：177 packages

### 编译与测试结果

| 检查项 | 结果 |
|--------|------|
| `cargo check -p aidaguard-tauri` | 通过 |
| `cargo build -p aidaguard-tauri` | 通过 |
| `npx tsc --noEmit` (TypeScript) | 零错误 |
| `cargo test -p aidaguard-core` (24 tests) | 全部通过 |

### 运行状态

```
cargo tauri dev
├── Vite dev server  → http://localhost:1420/  (HTTP 200)
├── React HMR         → 正常编译 TSX 模块
├── Tauri 窗口        → macOS 桌面可见 (aidaguard-tauri 进程)
└── 热重载            → 监听 aidaguard-core 和 aidaguard-tauri 变更
```

---

## Phase 8: 易用性与 UI 改进

### 设置页重构

| 变更 | 说明 |
|------|------|
| 移除 `target_url` 字段 | 上游 LLM API 地址不再在设置中直接输入 |
| 移除 `api_key` 字段 | API Key 不再在设置中直接输入 |
| 新增上游选择器 | 下拉切换默认上游，实时显示当前默认上游名称和地址 |
| "管理"按钮 | 点击跳转到「大模型接入」页面 |
| 新增 `rules_dir` 字段 | 在设置中配置规则文件目录 |

### 代理启动逻辑增强

- `start_proxy` 命令：当 `config.target_url` 为空时，自动从 `upstreams` 列表中查找 `default: true` 的上游，使用其 URL 和 API Key
- 新增 `set_default_upstream` Tauri 命令：设置指定上游为默认（同时取消其他上游的默认标记）

### 规则管理与配置关联

- Settings 表单新增 `rules_dir` 输入框
- `save_config` 命令保存时同步更新 `state.rules_dir` 运行时状态
- 修改规则目录后无需重启应用即可生效

### 配置清理

- `default_target_url()` 返回值清空（原为千帆测试地址）
- `config.toml` 删除明文 API Key `bce-v3/ALTAKSP-...`
- `config.toml` 已被 `.gitignore` 排除，不再进入 git 追踪
- `config.example.toml` 移除 `api_key` 和 `target_url` 顶级字段，加入 `[[upstreams]]` 示例

### 端口一致性验证

确认端口配置全链路一致：
```
config.port (default: 19000)
  → AppState.proxy_port
    → axum bind addr: 127.0.0.1:{port}
      → Settings Form.Item name="port"
```

### 新增文件

| 文件 | 变更类型 |
|------|----------|
| [commands/config.rs](../crates/aidaguard-tauri/src-tauri/src/commands/config.rs) | 修改 — 添加 rules_dir 同步 |
| [commands/proxy.rs](../crates/aidaguard-tauri/src-tauri/src/commands/proxy.rs) | 修改 — 上游解析逻辑 |
| [commands/upstream.rs](../crates/aidaguard-tauri/src-tauri/src/commands/upstream.rs) | 修改 — 新增 set_default_upstream |
| [main.rs](../crates/aidaguard-tauri/src-tauri/src/main.rs) | 修改 — 注册新命令 |
| [config.rs](../crates/aidaguard-core/src/config.rs) | 修改 — 清空测试 URL |
| [Settings.tsx](../crates/aidaguard-tauri/src/src/pages/Settings.tsx) | 重写 — 移除上游字段，新增选择器和 rules_dir |
| [useUpstreamStore.ts](../crates/aidaguard-tauri/src/src/store/useUpstreamStore.ts) | 修改 — 新增 setDefaultUpstream |
| [api/upstream.ts](../crates/aidaguard-tauri/src/src/api/upstream.ts) | 修改 — 新增 setDefaultUpstream |
| [config.toml](../config.toml) | 清理 — 删除测试 API Key |
| [config.example.toml](../config.example.toml) | 更新 — 移除顶级 api_key/target_url |

---

## 安全审计

对全部代码进行了安全审计，发现并修复了以下问题：

### 高严重性（已修复）

| 问题 | 文件 | 修复 |
|------|------|------|
| **ReDoS** — 用户提供的正则无大小限制 | [detector/mod.rs](../crates/aidaguard-core/src/detector/mod.rs), [rules.rs](../crates/aidaguard-tauri/src-tauri/src/commands/rules.rs) | 添加 `RegexBuilder::size_limit(1 << 20)` + 模式长度上限 2000 字符 + 测试文本上限 100,000 字符 |
| **弱密钥派生** — 单次 SHA-256 容易暴力破解 | [storage/mod.rs](../crates/aidaguard-core/src/storage/mod.rs) | 升级为 PBKDF2-HMAC-SHA256 (600,000 迭代 + 16 字节随机 salt) |
| **敏感数据日志泄露** | [server.rs](../crates/aidaguard-core/src/proxy/server.rs) | 原文匹配和替换后文本降级为 `debug!`，不再记录到生产日志 |
| **响应体泄露** — 连通性测试返回完整响应 | [upstream.rs](../crates/aidaguard-tauri/src-tauri/src/commands/upstream.rs) | 仅返回状态码和延迟，不返回响应体内容 |

### 中低严重性（已知，设计权衡）

| 问题 | 说明 |
|------|------|
| API Key 明文存储 | `keyring` crate 已在依赖中，后续可迁移到 OS 密钥链 |
| 审计数据通过 IPC 明文返回 | 本地桌面应用场景下的可接受风险 |
| CSV 导出无速率限制 | 可添加导出频率限制 |
| Tauri capabilities 范围较宽 | 生产部署前应收紧权限 |

---

## 测试覆盖

25 个单元测试全部通过：

- **detector**: 8 个（匹配、去重、空输入、重叠、身份证校验位、email 排除误报）
- **replacer**: 6 个（替换、还原、掩码、空匹配）
- **proxy/stream**: 5 个（SSE 安全截断、部分占位符）
- **storage**: 6 个（加密往返、CRUD、空数据库、无效数据）

TypeScript 编译零错误。

---

## Phase 9: 规则增强与 UI 优化

### 9.1 检测模式（Detect/Filter）

为规则引擎新增 `Mode` 枚举，支持两种工作模式：

| 模式 | 行为 |
|------|------|
| `Filter` | 检测并替换为占位符（原有行为） |
| `Detect` | 仅检测，记录审计但不替换请求体 |

**涉及文件：**
- [detector/mod.rs](../crates/aidaguard-core/src/detector/mod.rs) — 新增 `Mode` 枚举 + `Match` 增加 `mode` 字段
- [server.rs](../crates/aidaguard-core/src/proxy/server.rs) — 审计记录逻辑重构：分离 filter 和 detect 命中写入
- [Rules.tsx](../crates/aidaguard-tauri/src/src/pages/Rules.tsx) — 模式切换列 + 批量模式切换
- [RuleEditor.tsx](../crates/aidaguard-tauri/src/src/components/RuleEditor.tsx) — 模式选择下拉框

### 9.2 规则名持久化到数据库

审计记录中新增 `rule_name` 列，不再依赖前端加载规则来映射 ID → 名称。

**涉及文件：**
- [storage/mod.rs](../crates/aidaguard-core/src/storage/mod.rs) — `DetectionRecord` 增加 `rule_name` 字段 + `ALTER TABLE ADD COLUMN` 迁移 + 所有 CRUD 查询适配
- [server.rs](../crates/aidaguard-core/src/proxy/server.rs) — `storage.record()` 传入 `rule_name`，通过 `detector.rule_name()` 查询
- [AuditTable.tsx](../crates/aidaguard-tauri/src/src/components/AuditTable.tsx) — 直接使用 `record.ruleName`
- [AuditDetailPanel.tsx](../crates/aidaguard-tauri/src/src/components/AuditDetailPanel.tsx) — 同上

### 9.3 detect 模式审计记录修复

**Bug**: 全部规则设为 `detect` 时，审计记录不写入。根因：写入逻辑 gated on `placeholder_map.is_some()`，但 detect 模式不创建占位符。

**修复：** 重构审计写入逻辑，detect 命中在独立循环中写入，placeholder 为空字符串。

### 9.4 规则分类管理

**新增命令：**

| 命令 | 功能 |
|------|------|
| `create_category` | 创建新的规则分类（.yaml 文件），校验命名规则 |
| `delete_category` | 删除分类及其文件，同时重载检测器 |
| `rename_category` | 重命名分类文件，同时重载检测器 |

**前端：**
- 分类管理弹窗（创建 / 删除）
- 每个分类 Card 上的重命名和删除按钮

### 9.5 批量操作

每个分类 Card 的 extra 区域增加两个批量开关：

| 开关 | 功能 |
|------|------|
| 启用 | 批量启用/禁用该分类下全部规则 |
| 过滤/检测 | 批量切换该分类下全部规则的 mode |

### 9.6 正则误报排除机制

**Bug**: 邮箱规则 `[\w.+-]+@[\w-]+\.\w+` 误匹配 Retina 图片文件名（如 `128x128@2x.png`），但不能简单要求 local part 包含字母（`123456@qq.com` 是合法邮箱）。

**方案**: `RuleDef` 新增可选字段 `exclude`，命中文本若同时匹配排除正则则跳过。

```
检测命中 → exclude_regex.is_match(命中文本)
              ├─ true  → 跳过（误报）
              └─ false → 保留
```

**涉及文件：**
- [detector/mod.rs](../crates/aidaguard-core/src/detector/mod.rs) — `RuleDef` 增加 `exclude: Option<String>`，`CompiledRule` 增加 `exclude_regex: Option<Regex>`，`detect()` 中过滤
- [general.yaml](../rules/general.yaml) — 邮箱规则增加 `exclude: '@\d+x\.(?:png|jpg|jpeg|gif|svg|webp|ico|pdf)\b'`
- [types/index.ts](../crates/aidaguard-tauri/src/src/types/index.ts) — `RuleDef` 增加 `exclude?: string`

### 9.7 UI 滚动优化

**问题**: 规则管理页和审计记录页出现页面级竖向滚动条，规则明细区域无法独立滚动。

**方案**: Content 改为 `height: calc(100vh - 64px)` + `overflow: hidden`，禁止页面级滚动。各页面内部自行管理滚动。

| 页面 | 滚动策略 |
|------|---------|
| Dashboard | 外层 div `overflow: auto` |
| 审计记录 | 表格 `scroll.y` 仅表体滚动，操作列 `fixed: right` |
| 规则管理 | Flex 布局，工具栏固定顶部，仅规则明细区域 `overflow: auto` |
| 设置 | 外层 div `overflow: auto` |
| 大模型接入 | 外层 div `overflow: auto` |

### 9.8 其他修复

| 问题 | 修复 |
|------|------|
| RuleEditor Modal 过高，Save 按钮被遮挡 | 表单 `maxHeight: 60vh` + `overflow: auto` |
| 审计表格"规则"列标题 | 改为"规则名" |
| 审计列表表头列名为"规则" | 与详情面板统一为"规则名" |
| 审计记录无代理运行时无法查看 | main.rs 启动时初始化 Storage，start_proxy 复用已有实例 |
| 审计记录无数据时表格区域不可见 | 添加 `scroll.y` 确保表格区域始终可见 |

---

## Phase 10: 后续功能计划

### 10.1 系统通知集成

对接操作系统通知栏，检测到敏感数据时发送桌面通知。

**技术方案：**
- 后端：在 events relay 中，收到 `DetectionEvent` 后调用 `tauri::Notification` API（Tauri 2.x 内置）
- 通知内容：规则名 + 策略 + 请求路径摘要
- 频率限制：同一规则 1 分钟内最多 1 次通知，避免刷屏
- 通知点击：点击通知导航到审计日志详情
- 设置页：通知开关 + 频率限制配置

**涉及文件：**
- `events.rs` — relay 中新增通知发送逻辑
- `tauri.conf.json` — 开启 notification 权限
- `capabilities/default.json` — 添加 `notification:default`
- `Settings.tsx` — 通知开关配置

### 10.2 大模型生成检测规则

输入测试样例文本，由大语言模型自动生成检测规则（正则表达式）。

**技术方案：**
- 前端：测试样例输入框 + "生成规则"按钮
- 后端：调用当前默认上游 LLM API，构造 prompt 让模型根据样例生成正则规则
- Prompt 模板：提供样例 + 要求输出格式（Regex + 策略 + 模式）
- 解析模型返回的正则，写入规则编辑表单供用户确认和调整
- 用户确认后保存到规则文件

**涉及文件：**
- [commands/rules.rs](../crates/aidaguard-tauri/src-tauri/src/commands/rules.rs) — 新增 `generate_rule` 命令
- [forwarder.rs](../crates/aidaguard-core/src/proxy/forwarder.rs) — 复用 HTTP 转发能力调用 LLM
- [Rules.tsx](../crates/aidaguard-tauri/src/src/pages/Rules.tsx) — "生成规则"按钮入口
- 新增 [components/GenerateRuleModal.tsx](../crates/aidaguard-tauri/src/src/components/GenerateRuleModal.tsx)

### 10.3 仪表盘增强

将上游 LLM 配置从设置页移到仪表盘，提升操作效率。

**变更：**
- 仪表盘新增"代理信息"区域：显示代理地址 + 端口（绿色/灰色状态指示）
- 仪表盘新增"活跃规则"计数：显示当前启用的规则数 / 总规则数
- 仪表盘新增"当前模型"选择器：下拉切换默认上游 LLM（直接复用上游列表），显示当前模型的名称和地址
- `start_proxy` 命令适配：支持在仪表盘切换模型后自动更新代理目标
- 设置页移除上游相关配置（已在 Phase 8 完成），保留"管理上游"跳转链接

**涉及文件：**
- [Dashboard.tsx](../crates/aidaguard-tauri/src/src/pages/Dashboard.tsx) — 新增代理信息卡片 + 模型选择器
- [commands/proxy.rs](../crates/aidaguard-tauri/src-tauri/src/commands/proxy.rs) — 支持运行时切换上游
- [useProxyStore.ts](../crates/aidaguard-tauri/src/src/store/useProxyStore.ts) — 新增上游切换 action
- [Settings.tsx](../crates/aidaguard-tauri/src/src/pages/Settings.tsx) — 进一步精简上游字段

### 10.4 一键配置 AI 工具

代理启动后，自动检测并修改主流 AI 工具的 LLM 配置，将所有 API 请求指向本地代理。支持备份原始配置和一键恢复。

#### 架构设计

```
┌──────────────────────────────────────────────────┐
│                   Frontend                        │
│  ┌─────────────────────────────────────────────┐ │
│  │  AI 工具配置页                               │ │
│  │  ├─ 检测结果列表 (工具名 / 状态 / 当前端点)  │ │
│  │  ├─ 配置预览 (before → after diff)           │ │
│  │  ├─ [一键配置] [一键恢复]                     │ │
│  │  └─ 手动切换每个工具的启用/禁用               │ │
│  └─────────────────────────────────────────────┘ │
└──────────────────────┬───────────────────────────┘
                       │ Tauri invoke
┌──────────────────────┴───────────────────────────┐
│                   Backend (Rust)                  │
│  ┌─────────────────────────────────────────────┐ │
│  │  tools/                                      │ │
│  │  ├─ mod.rs          ToolAdapter trait        │ │
│  │  ├─ detect.rs       文件检测 + 路径解析      │ │
│  │  ├─ backup.rs       备份/恢复管理            │ │
│  │  └─ adapters/       各工具适配器             │ │
│  │      ├─ roo_code.rs                          │ │
│  │      ├─ cline.rs                             │ │
│  │      ├─ continue.rs                          │ │
│  │      ├─ cursor.rs                            │ │
│  │      ├─ windsurf.rs                          │ │
│  │      ├─ zed.rs                               │ │
│  │      ├─ aider.rs                             │ │
│  │      └─ claude_code.rs                       │ │
│  └─────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

#### ToolAdapter Trait

```rust
pub trait ToolAdapter: Send + Sync {
    /// 工具标识
    fn id(&self) -> &str;
    /// 工具显示名称
    fn name(&self) -> &str;
    /// 检测工具是否已安装（配置文件是否存在）
    fn detect(&self) -> Result<bool, String>;
    /// 读取当前配置
    fn read_config(&self) -> Result<ToolConfig, String>;
    /// 备份原始配置
    fn backup(&self) -> Result<(), String>;
    /// 将 API 端点重定向到代理
    fn configure(&self, proxy_url: &str) -> Result<(), String>;
    /// 从备份恢复原始配置
    fn restore(&self) -> Result<(), String>;
    /// 获取配置文件路径（用于前端展示）
    fn config_path(&self) -> &str;
}
```

#### 各工具配置详情

| 工具 | 平台 | 配置文件 | 修改字段 | 复杂度 |
|------|------|---------|---------|--------|
| **Roo Code** | VS Code | `~/Library/Application Support/Code/User/globalStorage/rooveterinaryinc.roo-cline/settings/` | `apiProvider`, `apiBase` (改为 `http://127.0.0.1:{port}`), `apiKey` (改为 aidaguard key) | 中 — JSON 文件 + VS Code settings.json |
| **Cline** | VS Code | `~/Library/Application Support/Code/User/globalStorage/saoudrizwan.claude-dev/settings/` | 同上 | 中 — 类似 Roo Code |
| **Continue** | VS Code / JetBrains | `~/.continue/config.json` | `models[].apiBase` → proxy URL | 低 — 标准 JSON，结构清晰 |
| **Aider** | CLI | `~/.aider.conf.yml` | `openai-api-base`, `anthropic-api-base` | 低 — 标准 YAML |
| **Claude Code** | CLI | `~/.claude/settings.json` 或环境变量 | `ANTHROPIC_BASE_URL` | 低 — JSON / env |
| **Cursor** | IDE | `~/Library/Application Support/Cursor/User/settings.json` | `cursor.apiBase` | 中 — JSON，需处理已有键 |
| **Windsurf** | IDE | `~/.codeium/windsurf/config.json` | API endpoint 配置 | 中 — 需调研具体字段 |
| **Zed** | IDE | `~/.config/zed/settings.json` | `openai_api_url` | 低 — 标准 JSON |

> 注：路径为 macOS 示例。Rust 端通过 `dirs` crate 或 Tauri path resolver 获取跨平台路径。

#### 安全设计

```
配置流程：
1. detect()  → 扫描已安装工具，列出可配置项
2. backup()  → 将原始配置复制到 ~/.aidaguard/backups/{tool_id}/{timestamp}/
3. 展示 diff → 前端显示每个工具配置的前后对比
4. configure() → 用户确认后写入新配置
5. restore() → 停用代理时一键恢复所有原始配置
```

**关键约束：**
- 绝不删除原始配置，backup 保留完整副本（含时间戳）
- 只修改 API endpoint 和 key 字段，保留其他配置不变
- 每个工具可独立勾选，不强制全部修改
- 恢复时校验备份完整性，防止部分恢复

#### 前端交互

```
┌─────────────────────────────────────────────────┐
│  AI 工具配置                          [一键配置] │
│                                                 │
│  ☑ Roo Code         已检测 ✓   3 个配置文件     │
│    当前端点: https://api.openai.com              │
│    → 将修改为: http://127.0.0.1:19000           │
│                                                 │
│  ☑ Continue         已检测 ✓   ~/.continue/     │
│    当前端点: http://localhost:11434              │
│    → 将修改为: http://127.0.0.1:19000           │
│                                                 │
│  ☐ Cline            未安装 —   跳过             │
│                                                 │
│  ☑ Claude Code      已检测 ✓   settings.json    │
│    当前端点: (默认)                              │
│    → 将修改为: http://127.0.0.1:19000           │
│                                                 │
│  ─────────────────────────────────────          │
│  备份位置: ~/.aidaguard/backups/                │
│  [恢复全部原始配置]                              │
└─────────────────────────────────────────────────┘
```

#### Tauri 命令

| 命令 | 功能 |
|------|------|
| `detect_tools` | 扫描所有支持的 AI 工具，返回安装状态和当前配置摘要 |
| `preview_tool_config` | 预览单个工具的配置变更（before/after） |
| `apply_tool_config` | 备份并写入指定工具的新配置 |
| `restore_tool_config` | 从备份恢复指定工具的原始配置 |
| `restore_all_tools` | 恢复所有已修改工具的原始配置 |

#### 涉及文件

| 文件 | 变更类型 |
|------|----------|
| `crates/aidaguard-tauri/src-tauri/src/tools/mod.rs` | 新增 — ToolAdapter trait + 统一入口 |
| `crates/aidaguard-tauri/src-tauri/src/tools/detect.rs` | 新增 — 工具检测 + 路径解析 |
| `crates/aidaguard-tauri/src-tauri/src/tools/backup.rs` | 新增 — 备份/恢复管理 |
| `crates/aidaguard-tauri/src-tauri/src/tools/adapters/*.rs` | 新增 — 8 个工具适配器 |
| `crates/aidaguard-tauri/src-tauri/src/commands/tools.rs` | 新增 — 5 个 Tauri 命令 |
| `crates/aidaguard-tauri/src-tauri/src/main.rs` | 修改 — 注册 tools 模块和命令 |
| `crates/aidaguard-tauri/src-tauri/src/state.rs` | 修改 — 可能新增 backups_dir |
| `crates/aidaguard-tauri/src-tauri/Cargo.toml` | 修改 — 新增 dirs crate 依赖 |
| [pages/ToolsConfig.tsx](../crates/aidaguard-tauri/src/src/pages/ToolsConfig.tsx) | 新增 — AI 工具配置页 |
| [api/tools.ts](../crates/aidaguard-tauri/src/src/api/tools.ts) | 新增 — Tauri invoke 封装 |
| [store/useToolsStore.ts](../crates/aidaguard-tauri/src/src/store/useToolsStore.ts) | 新增 — Zustand store |
| [App.tsx](../crates/aidaguard-tauri/src/src/App.tsx) | 修改 — 新增路由 + 侧边栏菜单项 |
| [types/index.ts](../crates/aidaguard-tauri/src/src/types/index.ts) | 修改 — 新增 ToolInfo, ToolConfig 类型 |

---

## 优先级建议

| 优先级 | 功能 | 理由 |
|--------|------|------|
| 1 | **10.1 系统通知** | 投入小、Tauri 原生支持、用户感知强 |
| 2 | **10.3 仪表盘增强** | 核心交互流程优化，代理信息一目了然 |
| 3 | **10.4 一键配置 AI 工具** | 工作量大（8 个适配器），但对用户价值最高 |
| 4 | **10.2 LLM 生成规则** | 锦上添花，依赖上游 LLM 可用性 |

```
aidaguard/
├── Cargo.toml                         # workspace 根
├── .gitignore
├── config.example.toml
├── rules/                             # YAML 规则文件
│   ├── general.yaml
│   ├── finance.yaml
│   └── medical.yaml
├── docs/
│   ├── DEVELOPMENT.md                 # 开发总结（MVP 阶段）
│   ├── UI_DESIGN.md                   # 桌面应用 UI 设计规格
│   └── WORKLOG.md                     # 本文件
└── crates/
    ├── aidaguard-core/                # 核心引擎
    │   ├── Cargo.toml
    │   └── src/
    │       ├── main.rs                # CLI 入口
    │       ├── lib.rs                 # 模块声明
    │       ├── config.rs              # 配置结构体 + 加载/保存
    │       ├── detector/mod.rs        # 正则检测引擎 + 去重
    │       ├── replacer/mod.rs        # 占位符替换/还原
    │       ├── storage/mod.rs         # 加密 SQLite 审计存储
    │       └── proxy/
    │           ├── mod.rs             # DetectionEvent 定义
    │           ├── server.rs          # Axum 代理服务器
    │           ├── forwarder.rs       # HTTP 请求转发器
    │           └── stream.rs          # SSE 流处理 + 占位符还原
    └── aidaguard-tauri/               # Tauri 桌面应用
        ├── src/                       # 前端 (React)
        │   ├── package.json
        │   ├── vite.config.ts
        │   ├── index.html
        │   └── src/
        │       ├── main.tsx           # React 入口
        │       ├── App.tsx            # 根布局 + 路由
        │       ├── types/index.ts     # 类型定义
        │       ├── api/               # Tauri invoke 封装
        │       ├── store/             # Zustand stores
        │       ├── pages/             # 页面组件
        │       ├── components/        # 通用组件
        │       └── hooks/             # 自定义 hooks
        └── src-tauri/                 # 后端 (Rust)
            ├── Cargo.toml
            ├── tauri.conf.json
            ├── capabilities/
            ├── icons/
            └── src/
                ├── main.rs            # Tauri 入口
                ├── lib.rs             # 模块声明
                ├── state.rs           # AppState 定义
                ├── tray.rs            # 系统托盘
                ├── events.rs          # 事件 relay
                └── commands/          # Tauri 命令
                    ├── proxy.rs       # 代理生命周期
                    ├── audit.rs       # 审计日志
                    ├── rules.rs       # 规则管理
                    ├── config.rs      # 配置读写
                    └── upstream.rs    # 上游管理
```

---

## Phase 11 — Roadmap (2026-05-05)

后续重点方向：

1. **AI 工具配置插件化** — 将工具适配器重构为插件架构，每个 AI 工具的配置逻辑独立加载、独立更新，无需重新编译核心。

2. **检测引擎升级（参考 Presidio）** — 参考 Microsoft Presidio 项目的设计，使用 Rust 原生实现新的检测引擎，包含实体识别、验证、匿名化 pipeline。

3. **多国规则库独立扩展** — 根据不同国家/地区的要求，支持独立扩展检测规则库，用户可按需启用所在司法管辖区的规则。

4. **合规引用规则** — 建立规则前先收集各国对敏感数据保护的具体要求，每条规则明确标注遵循了哪条法规要求。同一条规则可能对应多个不同国家/地区的要求（例如 GDPR 与中国《个人信息保护法》对同一类 PII 都有检测要求）。
```

---

## Phase 12 — Presidio 架构分析与 Rust 检测引擎设计 (2026-05-05)

开展检测引擎升级前，对 Microsoft Presidio 项目进行了全面技术分析。

### 核心发现

1. **双层引擎** — Analyzer（检测）+ Anonymizer（脱敏），通过 `RecognizerResult` 衔接
2. **三层检测** — Regex 宽口径匹配（~0.3 分）→ Checksum 校验（Luhn / mod-97 等）→ Context 上下文增强 → 综合输出置信度
3. **校验三态返回** — `True`(升满分) / `False`(丢弃) / `None`(保持原始分)
4. **NLP 必要性** — 结构化实体(regex)覆盖 80%，但人名/地名/机构名/日期等需要 ONNX NER 模型（~50MB 按需加载）

### 实体类型全景

- Presidio 用 spaCy OntoNotes 5.0（18 类 NER 实体），但对敏感数据检测远不够
- 敏感数据应分三类：
  - **结构化**（regeex+校验）：信用卡、身份证、手机号、SSN、IBAN 等
  - **非结构化**（NLP NER）：人名、地名、机构、日期、民族/宗教（NORP 特殊类别）
  - **网络/系统**（regex）：Email、IP、API Key、JWT、私钥等

### 多区域 NER 策略

- 全局实体：CreditCard、Email、IP 等所有地区通用
- 区域实体：身份证(CN)、SSN(US)、NINO(UK)、CPF(BR) 等
- NLP 按语言加载：中文 `bert-base-chinese`、英文 `bert-base-NER`、德文 `bert-base-german-cased` 等
- 区域 Preset Bundle：CN/US/EU-{DE,FR}/UK/JP/BR/IN/SG/KR 各预设实体+语言+法规

### Rust 引擎项目结构

```
crates/aidaguard-detector/       ← 新 crate
├── core/          entity_type, recognizer trait, registry, result
├── recognizers/   pattern/ (regex+校验), nlp/ (ONNX推理)
├── validation/    luhn, mod_n, email, context
├── anonymizer/    replace, mask, hash, encrypt
└── pipeline.rs    编排 Analyze → PostProcess → Anonymize
```

- **NlpEngine trait** 可插拔：空实现 / ONNX / Candle / rust-bert
- **NlpEngineProvider trait** 按语言加载/卸载模型

详细分析见: memory/presidio-analysis.md

---

## Phase 13 — 统一项目文件架构 (2026-05-05)

### 13.1 当前架构问题

| 问题 | 说明 |
|------|------|
| `aidaguard-core` 职责混乱 | 既是 library 又是 binary，混入 5 个独立模块：detector、replacer、proxy、storage、config |
| `proxy/server.rs` 隐式紧耦合 | 定义自己的 `ProxyState`，与 `AppState` 部分重复 |
| 依赖重复声明 | `aidaguard-tauri` 重新声明 regex、serde_yaml、notify、reqwest，本应由 core 透传 |
| 逻辑重复 | `commands/rules.rs` 的 `read_rule_files()` 与 `detector/mod.rs` 的 `load_from_dir()` 各自实现一遍 YAML 遍历+解析 |
| 无预留扩展位 | 即将引入 `aidaguard-detector`、`aidaguard-upstream`，当前 flat 布局无处安放 |

### 13.2 目标架构

```
crates/
├── aidaguard-core/            ← 零依赖类型 + trait (最底层)
│   └── src/
│       ├── types/
│       │   ├── match.rs        ← Match, Strategy, Mode
│       │   ├── rule.rs         ← RuleDef, RuleFile, CompiledRule
│       │   ├── entity.rs       ← EntityType, EntityCategory
│       │   └── config.rs       ← Config (全局设置，不含 upstreams)
│       ├── engine.rs           ← DetectionEngine trait
│       └── lib.rs
│
├── aidaguard-upstream/        ← 大模型供应商管理 (NEW)
│   └── src/
│       ├── protocol.rs         ← ProtocolType, AuthType (内置枚举)
│       ├── provider.rs         ← ProviderConfig (从 YAML 加载)
│       ├── model.rs            ← ModelInfo (id, context, capabilities, cost)
│       ├── client.rs           ← 统一 LLM 客户端 (openai/anthropic 协议分发)
│       ├── manager.rs          ← UpstreamManager (CRUD、默认、切换)
│       ├── connectivity.rs     ← 连通性测试 + 延迟测量
│       └── types.rs            ← UpstreamConfig, ToolAssignment
│
├── aidaguard-detector/        ← Presidio 风格检测引擎 (NEW)
│   └── src/
│       ├── core/               ← recognizer trait, registry, result
│       ├── recognizers/
│       │   ├── pattern/        ← PatternRecognizer (regex + checksum)
│       │   └── nlp/            ← NlpRecognizer (ONNX NER, 可选)
│       ├── validation/         ← Luhn, mod-N, context scoring
│       └── anonymizer/         ← replace, mask, hash, encrypt
│
├── aidaguard-proxy/           ← 代理服务器 (从 core 抽出)
│   └── src/
│       ├── server.rs           ← HTTP proxy + detect + replace + restore
│       ├── forwarder.rs        ← 调用 upstream::client 转发
│       └── stream.rs           ← SSE 流处理
│
├── aidaguard-storage/         ← 审计存储 (从 core 抽出)
│   └── src/
│       ├── storage.rs          ← SQLite + AES 加密
│       └── models.rs           ← DetectionRecord, AuditGroup
│
├── aidaguard-plugins/         ← AI 工具插件系统 (从 tauri 抽出)
│   └── src/
│       ├── plugin.rs           ← PluginManifest, Plugin trait
│       ├── registry.rs         ← PluginRegistry + 状态持久化
│       ├── backup.rs           ← 配置备份/还原
│       └── adapters/           ← 13 个工具适配器
│
└── aidaguard-tauri/           ← 桌面壳 (薄层，只做命令编排)
    └── src-tauri/src/
        ├── state.rs            ← AppState 组装所有引擎
        ├── commands/           ← 薄 Tauri command 包装
        │   ├── proxy.rs        ← start/stop proxy
        │   ├── rules.rs        ← 规则 CRUD + test + generate
        │   ├── tools.rs        ← 插件 enable/disable/configure/restore
        │   ├── config.rs       ← 全局 settings 读写
        │   ├── upstream.rs     ← 上游 CRUD + 连接测试
        │   └── audit.rs        ← 审计日志查询 + 导出
        ├── events.rs
        └── tray.rs
```

### 13.3 依赖方向 (单向，无循环)

```
                      aidaguard-core (types + traits)
                     /     |        |         \
                    /      |        |          \
           upstream   detector   storage     plugins
               \         |         |           /
                \        |         |          /
                 \       |         |         /
                  aidaguard-proxy (组合所有引擎)
                          |
                    aidaguard-tauri (组装一切, thin shell)
```

### 13.4 各 crate 职责

| Crate | 职责 | 核心导出 |
|---|---|---|
| `aidaguard-core` | 共享类型 + 引擎 trait | `Match`, `Strategy`, `DetectionEngine`, `RuleDef`, `Config` |
| `aidaguard-upstream` | 大模型供应商管理 + 统一客户端 | `UpstreamManager`, `UpstreamClient`, `ProviderConfig`, `ModelInfo` |
| `aidaguard-detector` | 检测：regex + NLP + 校验 + 匿名化 | `AnalyzerEngine`, `Recognizer`, `EntityType` |
| `aidaguard-proxy` | HTTP 代理：拦截 → 检测 → 替换 → 转发 → 还原 | `start()`, `start_with_state()` |
| `aidaguard-storage` | 审计持久化 | `Storage`, `DetectionRecord` |
| `aidaguard-plugins` | AI 工具配置管理 | `PluginRegistry`, `PluginManifest` |
| `aidaguard-tauri` | 桌面壳 + Tauri commands | 无 (二进制) |

### 13.5 LLM 供应商声明式设计

供应商和模型作为 **数据** 而非 **代码**，用户可通过 YAML 文件自行添加新厂商，无需编译。

**内置协议层**（Rust，覆盖 99% 场景）：

```rust
pub enum ProtocolType {
    OpenAiCompatible,      // /v1/chat/completions
    AnthropicCompatible,   // /v1/messages
}

pub enum AuthType {
    BearerToken,           // Authorization: Bearer <key>
    ApiKeyHeader(String),  // x-api-key, x-goog-api-key, ...
}
```

**供应商文件**（YAML，用户可自行添加、修改、分享）：

```yaml
# providers/deepseek.yaml
id: deepseek
name: DeepSeek
protocol: openai_compatible
auth: bearer_token
endpoint: https://api.deepseek.com/v1
models:
  - id: deepseek-v4
    name: DeepSeek V4
    context: 128000
    max_output: 8192
    capabilities: [chat, streaming]
```

**内置供应商目录**：

```
providers/
├── openai.yaml
├── anthropic.yaml
├── gemini.yaml
├── deepseek.yaml
├── qwen.yaml
├── zhipu.yaml
├── groq.yaml
├── aws_bedrock.yaml
└── custom_example.yaml
```

只有协议层需要 Rust 代码。当某个厂商的协议无法声明式覆盖时（如 AWS SigV4 签名），才内置新的 `AuthType` 变体。

### 13.6 三种声明式扩展模式

项目中出现了统一的扩展模式：

| 扩展点 | 格式 | 目录 | 加载 |
|---|---|---|---|
| 检测规则 | YAML | `rules/` | `Detector::load_from_dir()` |
| 工具插件 | Rust struct (编译进) | — | `PluginRegistry::register()` |
| LLM 供应商 | YAML | `providers/` | `ProviderRegistry::load_from_dir()` |

### 13.7 新旧对比

| | 当前 | 统一后 |
|---|---|---|
| 上游数据 | `UpstreamConfig` 扁平结构，手动填 URL | `ProviderConfig` 声明式，选厂商自动填端点 |
| 协议适配 | `if is_anthropic { ... } else { ... }` | `ProtocolType` 枚举 + 统一 client |
| 模型管理 | 逗号分隔字符串 `models: "gpt-5,claude-4"` | `ModelInfo` 结构化，带能力和上下文窗口 |
| 添加厂商 | 改代码 | 写一个 YAML 文件 |
| 切换模型 | 手动改配置文件 | UI 一键切换，自动更新所有工具配置 |

### 13.8 迁移步骤

| 步骤 | 内容 | 涉及文件 |
|------|------|---------|
| 1 | `aidaguard-core` 精简为 types + traits | 移走 proxy/、storage/ 到独立 crate |
| 2 | 新建 `aidaguard-storage` | 移入 storage/ |
| 3 | 新建 `aidaguard-plugins` | 移入 tools/ (已部分完成) |
| 4 | 新建 `aidaguard-upstream` | 定义 Provider、统一 client、内置供应商 YAML |
| 5 | 新建 `aidaguard-proxy` | 移入 server/forwarder/stream，对接 upstream::client |
| 6 | 新建 `aidaguard-detector` 骨架 | 预留 crate，Phase 14+ 实现 |
| 7 | `aidaguard-tauri` 改为只做组装 | 删除重复依赖，commands 改为调用各 crate |
| 8 | 删除 `aidaguard-core` 的 `[[bin]]` | CLI 入口移到 tauri 或独立 binary crate |

---

## Phase 14 — 新架构迁移开发计划 (2026-05-05)

### 14.1 总览

| Step | 内容 | 类型 | 预计工作量 | 依赖 |
|------|------|------|-----------|------|
| S1 | 新建 `aidaguard-storage` | 拆分 | 小 | — |
| S2 | 新建 `aidaguard-plugins` | 拆分 | 小 | — |
| S3 | 新建 `aidaguard-detector` 骨架 | 新建 | 极小 | — |
| S4 | 新建 `aidaguard-upstream` | 新建 | 中 | — |
| S5 | 新建 `aidaguard-proxy` | 拆分 | 中 | S4 |
| S6 | 重构 `aidaguard-core` | 精简 | 中 | S1, S5 |
| S7 | 清理 `aidaguard-tauri` | 精简 | 中 | S1, S2, S4, S5, S6 |
| S8 | 移除 CLI binary | 收尾 | 极小 | S7 |

### 14.2 依赖关系图

```
S1 (storage) ──────────────────────┐
                                    │
S2 (plugins) ──────────────────────┤
                                    ├──→ S6 (core 精简) ──┐
S4 (upstream) ──→ S5 (proxy) ──────┘                      │
                                                           ├──→ S7 (tauri 清理) ──→ S8 (移除 binary)
S3 (detector 骨架) ────────────────────────────────────────┘
```

S1、S2、S3、S4 可以并行开始。S5 依赖 S4。S6 依赖 S1+S5。S7 依赖所有前置步骤。S8 收尾。

---

### 14.3 Step 1: 新建 `aidaguard-storage`

**目标**：将 `aidaguard_core::storage` 模块独立为新 crate。

**当前状态**：
- `aidaguard_core::storage` 包含 `Storage` 结构体 + `DetectionRecord` + `AuditGroup` + `AuditStats` + `RuleCount`
- 依赖：`rusqlite`, `aes-gcm`, `pbkdf2`, `sha2`, `rand` (全部 workspace deps)
- 消费者：`proxy/server.rs`, `commands/audit.rs`, `commands/proxy.rs`, `events.rs`, `main.rs`

**操作清单**：

| # | 操作 | 文件 |
|---|------|------|
| 1.1 | 创建 crate 目录和 Cargo.toml | `crates/aidaguard-storage/Cargo.toml` (NEW) |
| 1.2 | 移入 storage 模块 | `crates/aidaguard-storage/src/lib.rs` (NEW，内容 = 原 storage/mod.rs) |
| 1.3 | 在 workspace 注册 | `Cargo.toml` — members 新增 `"crates/aidaguard-storage"` |
| 1.4 | aidaguard-core 透传 | `crates/aidaguard-core/src/storage/mod.rs` 改为 `pub use aidaguard_storage::*;` |
| 1.5 | aidaguard-tauri 直接依赖 storage | `crates/aidaguard-tauri/src-tauri/Cargo.toml` 新增 `aidaguard-storage = { path = "../../aidaguard-storage" }` |
| 1.6 | 更新 use 语句 | `proxy/server.rs`, `commands/audit.rs`, `commands/proxy.rs`, `events.rs`, `main.rs` — `use aidaguard_core::storage::*` → `use aidaguard_storage::*` |

**验证**：`cargo build -p aidaguard-tauri` + `cargo test -p aidaguard-storage` 全部通过。

---

### 14.4 Step 2: 新建 `aidaguard-plugins`

**目标**：将 `tools/` 模块从 `aidaguard-tauri` 独立为新 crate。

**当前状态**：
- `tools/mod.rs` — PluginManifest, Plugin trait, ToolAdapter trait, ToolInfo, ToolConfig
- `tools/registry.rs` — PluginRegistry
- `tools/backup.rs` — 配置备份/还原
- `tools/adapters/` — 13 个适配器
- 只有 adapter 的 `configure()` 方法需要 `proxy_port: u16`（纯数值，无 Tauri 依赖）

**操作清单**：

| # | 操作 | 文件 |
|---|------|------|
| 2.1 | 创建 crate 目录和 Cargo.toml | `crates/aidaguard-plugins/Cargo.toml` (NEW) — 依赖 serde, serde_json, dirs |
| 2.2 | 移入 plugins 核心 | `crates/aidaguard-plugins/src/lib.rs` (NEW) |
| 2.3 | 移入 PluginManifest + Plugin trait | `crates/aidaguard-plugins/src/plugin.rs` (NEW，内容 = 原 tools/mod.rs) |
| 2.4 | 移入 PluginRegistry | `crates/aidaguard-plugins/src/registry.rs` (NEW，内容 = 原 tools/registry.rs) |
| 2.5 | 移入 backup | `crates/aidaguard-plugins/src/backup.rs` (NEW，内容 = 原 tools/backup.rs) |
| 2.6 | 移入 adapters | `crates/aidaguard-plugins/src/adapters/` (NEW，13 个 adapter 文件) |
| 2.7 | 在 workspace 注册 | `Cargo.toml` — members 新增 `"crates/aidaguard-plugins"` |
| 2.8 | aidaguard-tauri 依赖 plugins | `Cargo.toml` 新增 `aidaguard-plugins = { path = "../../aidaguard-plugins" }` |
| 2.9 | 更新 tauri 端 use 语句 | `commands/tools.rs`, `state.rs`, `main.rs` — `use crate::tools::*` → `use aidaguard_plugins::*` |
| 2.10 | 删除 tauri 端 tools/ 目录 | 移除 `tools/mod.rs`, `tools/registry.rs`, `tools/backup.rs`, `tools/adapters/` |

**验证**：`cargo build -p aidaguard-tauri` 编译通过，前端 ToolsConfig 页面正常工作。

---

### 14.5 Step 3: 新建 `aidaguard-detector` 骨架

**目标**：创建空 crate 作为后续 Presidio 引擎的占位。

**操作清单**：

| # | 操作 | 文件 |
|---|------|------|
| 3.1 | 创建 Cargo.toml | `crates/aidaguard-detector/Cargo.toml` (NEW) — 依赖 aidaguard-core |
| 3.2 | 创建占位 lib.rs | `crates/aidaguard-detector/src/lib.rs` (NEW) — `pub use aidaguard_core::DetectionEngine;` + 空 doc |
| 3.3 | 在 workspace 注册 | `Cargo.toml` — members 新增 `"crates/aidaguard-detector"` |

**验证**：`cargo build -p aidaguard-detector` 编译通过。

---

### 14.6 Step 4: 新建 `aidaguard-upstream`

**目标**：创建供应商管理系统 + 声明式 YAML 供应商 + 统一 LLM 客户端。

**操作清单**：

| # | 操作 | 文件 |
|---|------|------|
| 4.1 | 创建 Cargo.toml | `crates/aidaguard-upstream/Cargo.toml` (NEW) — 依赖 aidaguard-core, serde, serde_yaml, reqwest |
| 4.2 | 定义类型 | `crates/aidaguard-upstream/src/types.rs` (NEW) — `ProtocolType`, `AuthType`, `ProviderConfig`, `ModelInfo`, `ModelCapabilities`, `ToolAssignment` |
| 4.3 | 实现 ProviderConfig 加载 | `crates/aidaguard-upstream/src/provider.rs` (NEW) — `ProviderRegistry`, `load_from_dir()`, 内置供应商索引 |
| 4.4 | 实现统一 LLM 客户端 | `crates/aidaguard-upstream/src/client.rs` (NEW) — `UpstreamClient` with `chat()`, `chat_json()`, `test_connectivity()` |
| 4.5 | 实现管理器 | `crates/aidaguard-upstream/src/manager.rs` (NEW) — `UpstreamManager` CRUD + 默认切换 |
| 4.6 | 创建内置供应商 YAML | `providers/openai.yaml`, `anthropic.yaml`, `deepseek.yaml`, `qwen.yaml`, `zhipu.yaml`, `groq.yaml`, `gemini.yaml`, `custom_example.yaml` (NEW) |
| 4.7 | 在 workspace 注册 | `Cargo.toml` — members 新增 `"crates/aidaguard-upstream"` |
| 4.8 | 迁移现有 UpstreamConfig → 新类型 | `aidaguard_core::config` 移除 `UpstreamConfig`/`UpstreamProtocol`，上游管理改用 `aidaguard_upstream::types` |

**核心类型设计**：

```rust
// types.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolType {
    OpenAiCompatible,
    AnthropicCompatible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthType {
    BearerToken,
    ApiKeyHeader { header: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub protocol: ProtocolType,
    pub auth: AuthType,
    pub endpoint: String,
    #[serde(default)]
    pub models: Vec<ModelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub context: usize,
    #[serde(default)]
    pub max_output: usize,
    #[serde(default)]
    pub capabilities: Vec<String>,  // "chat", "vision", "function_calling", "streaming"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamConfig {
    #[serde(flatten)]
    pub provider: ProviderConfig,   // 从 YAML 加载的供应商信息
    pub api_key: Option<String>,    // 用户填入的密钥
    #[serde(default)]
    pub is_default: bool,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    #[serde(default)]
    pub rate_limit_qps: u32,
}
```

**验证**：`cargo build -p aidaguard-upstream` + `cargo test -p aidaguard-upstream` 通过。内置供应商 YAML 全部可解析。

---

### 14.7 Step 5: 新建 `aidaguard-proxy`

**目标**：将 proxy 模块从 `aidaguard-core` 移入独立 crate，并对接 `aidaguard-upstream::client`。

**操作清单**：

| # | 操作 | 文件 |
|---|------|------|
| 5.1 | 创建 Cargo.toml | `crates/aidaguard-proxy/Cargo.toml` (NEW) — 依赖 aidaguard-core, aidaguard-upstream, aidaguard-storage, axum, reqwest, tokio, tracing |
| 5.2 | 移入 proxy 模块 | `crates/aidaguard-proxy/src/lib.rs` (NEW) + `server.rs` + `forwarder.rs` + `stream.rs` |
| 5.3 | 替换 LLM 调用逻辑 | `forwarder.rs` — 删除内联的 Anthropic/OpenAI 协议分支，改为调用 `UpstreamClient::chat()` |
| 5.4 | 替换 server.rs 中的上游解析 | `server.rs` — 删除 `start()` 中的内联上游解析，改用 `UpstreamManager` |
| 5.5 | 在 workspace 注册 | `Cargo.toml` — members 新增 `"crates/aidaguard-proxy"` |
| 5.6 | aidaguard-core 透传 | `crates/aidaguard-core/src/proxy/mod.rs` 改为 `pub use aidaguard_proxy::*;` |
| 5.7 | aidaguard-tauri 直接依赖 proxy | `Cargo.toml` 新增 `aidaguard-proxy = { path = "../../aidaguard-proxy" }` |

**forwarder 重构对比**：

```
Before (内联协议判断):
  if is_anthropic { header x-api-key, body with messages }
  else { header Authorization: Bearer, body with chat/completions }

After (统一客户端):
  upstream_client.chat(&upstream, &body).await
```

**验证**：`cargo build -p aidaguard-tauri` + `cargo test -p aidaguard-proxy` 通过。代理启动后正常转发请求。

---

### 14.8 Step 6: 重构 `aidaguard-core`

**目标**：精简 core 为纯 types + traits，旧的 `Detector` 实现 `DetectionEngine` trait。

**操作清单**：

| # | 操作 | 文件 |
|---|------|------|
| 6.1 | 新建 `DetectionEngine` trait | `crates/aidaguard-core/src/engine.rs` (NEW) |
| 6.2 | 重构目录结构 | 创建 `crates/aidaguard-core/src/types/` 目录，拆入 match.rs, strategy.rs, rule.rs, entity.rs |
| 6.3 | 实现 DetectionEngine for Detector | `detector/mod.rs` — `impl DetectionEngine for Detector { ... }` |
| 6.4 | 清理 Cargo.toml | 移除 axum, reqwest, rusqlite, futures, notify 等不再需要的 deps |
| 6.5 | 更新 lib.rs | 精简 exports，移除 `pub mod proxy; pub mod storage;` |

**`DetectionEngine` trait**：

```rust
pub trait DetectionEngine: Send + Sync {
    fn detect(&self, text: &str) -> Vec<Match>;
    fn rule_count(&self) -> usize;
    fn rule_name(&self, id: &str) -> Option<&str>;
}
```

旧的 `Detector` 已经拥有这三个方法，只需 `impl DetectionEngine for Detector` 即可无缝对接。

**验证**：`cargo build -p aidaguard-core` + `cargo test -p aidaguard-core` 通过。所有原有测试继续通过。

---

### 14.9 Step 7: 清理 `aidaguard-tauri`

**目标**：删除重复依赖，命令改为调用各独立 crate 的 API。

**操作清单**：

| # | 操作 | 文件 |
|---|------|------|
| 7.1 | 清理 Cargo.toml 依赖 | 移除 regex, serde_yaml, notify, reqwest, csv, serde_json（已由各 crate 声明或透传） |
| 7.2 | 更新 state.rs | `AppState` — 删除 `Arc<RwLock<Detector>>`，改为 `Arc<RwLock<Box<dyn DetectionEngine>>>` 或使用具体类型 |
| 7.3 | 更新 commands/rules.rs | 删除 `read_rule_files()` / `write_rule_file()`，改为调用 `aidaguard-core` 的统一实现 |
| 7.4 | 更新 commands/upstream.rs | 删除内联 HTTP 测试逻辑，改为调用 `UpstreamClient::test_connectivity()` |
| 7.5 | 更新 commands/proxy.rs | 初始化改为使用 `aidaguard_proxy::start_with_state()` |
| 7.6 | 更新 commands/config.rs | 上游管理字段引用改为 `aidaguard_upstream::types` |
| 7.7 | 更新 events.rs | storage 引用改为 `aidaguard_storage` |

**验证**：`cargo build -p aidaguard-tauri` + `npx tsc --noEmit` + `cargo test` 全部通过。`cargo tauri dev` 正常启动，所有功能可用。

---

### 14.10 Step 8: 移除 CLI binary

**目标**：删除 `aidaguard-core` 的 `[[bin]]`。

**操作清单**：

| # | 操作 | 文件 |
|---|------|------|
| 8.1 | 删除 [[bin]] 声明 | `crates/aidaguard-core/Cargo.toml` — 移除 `[[bin]]` section |
| 8.2 | 删除 main.rs | `crates/aidaguard-core/src/main.rs` — 删除文件 |

> 如果后续需要独立 CLI 工具，可以在 `crates/aidaguard-cli/` 新建独立 binary crate，而非污染 core。

**验证**：`cargo build --workspace` 通过，无 binary 输出。

---

### 14.11 风险与缓解

| 风险 | 缓解 |
|------|------|
| `cargo check` 中途长时间不可用 | 每步结束时验证编译，不跨步 |
| 类型引用路径变更导致大面积修改 | Step 1-6 期间 aidaguard-core 保留 `pub use` 透传，等 Step 7 统一切 |
| `AppState` 改动影响所有 commands | Step 7 集中处理，前置步骤不碰 state |
| aidaguard-tauri Cargo.toml 改动破坏构建 | 先确认 workspace.dependencies 中所有 crate 可用 |

### 14.12 各 Step 后的 workspace Cargo.toml 变化

```
Before:
members = ["crates/aidaguard-core", "crates/aidaguard-tauri/src-tauri"]

After Step 1-5:
members = [
    "crates/aidaguard-core",
    "crates/aidaguard-storage",     # S1
    "crates/aidaguard-plugins",     # S2
    "crates/aidaguard-detector",    # S3
    "crates/aidaguard-upstream",    # S4
    "crates/aidaguard-proxy",       # S5
    "crates/aidaguard-tauri/src-tauri",
]
```

---

## Phase 15 — 业务逻辑验证与 core 模块精确定义 (2026-05-05)

### 15.1 逐业务流追踪

逐条验证每个用户操作在新架构下的完整调用链路。

#### 流 1: 启动代理

```
用户点击 "Start Proxy"
  → Tauri command start_proxy()
    → 从 UpstreamManager 解析默认上游 (url + api_key + name)
      ↑ aidaguard-upstream
    → 从 Config 读取 port, max_body_size, rules_dir
      ↑ aidaguard-core
    → engine.write().await.reload(&rules_dir)
      ↑ aidaguard-core::DetectionEngine trait
    → Storage::open(db_path, enc_key)
      ↑ aidaguard-storage
    → start_with_state(port, max_body_size, engine, storage, upstream, ...)
      ↑ aidaguard-proxy
        → server.rs: 每个请求:
          engine.read().await.detect(text)  → Vec<Match>
          replacer::replace(text, &matches) → (String, PlaceholderMap)
          forwarder.forward(...)             → Response
          replacer::restore(resp, &map)      → String
          storage.record(...)                → ()
```

✅ 所有依赖单向流动，无循环。

#### 流 2: 规则 CRUD

```
用户 创建/编辑/删除 规则
  → Tauri command save_rule / delete_rule / toggle_rule
    → read_rule_files(&dir) → Vec<(String, RuleFile)>
      ↑ aidaguard-core (RuleDef, RuleFile 类型)
    → 修改内存中的 RuleFile, write_rule_file()
    → engine.write().await.reload(&dir)
      ↑ aidaguard-core::DetectionEngine trait
```

✅ 规则文件 I/O 在 tauri commands 中（薄层），类型在 core。

#### 流 3: 正则测试

```
用户 在 RuleEditor 中点击 "Run Test"
  → Tauri command test_rule(pattern, test_text)
    → compile_regex(&pattern) → Regex
      ↑ aidaguard-core
    → regex.find_iter(&test_text) → 手动构造 Vec<Match>
    → replacer::replace(&test_text, &matches) → (String, PlaceholderMap)
      ↑ aidaguard-core
    → 返回 MatchInfo + sanitized_text
```

✅ 所有依赖都在 core，不经过 engine trait。

#### 流 4: 大模型生成规则

```
用户 输入样例文本，点击 "Generate Rule"
  → Tauri command generate_rule(sample_text)
    → UpstreamManager::default_upstream() → UpstreamConfig
      ↑ aidaguard-upstream
    → UpstreamClient::chat_json(system_prompt, user_prompt) → GeneratedRule
      ↑ aidaguard-upstream (统一 client，不再内联 if is_anthropic)
    → 返回 GeneratedRule 到前端供用户确认
```

✅ 生成逻辑不再散落在 rules.rs 中，由 upstream client 统一处理。

#### 流 5: AI 工具配置

```
用户 进入 ToolsConfig 页面
  → Tauri command detect_tools()
    → PluginRegistry::all_plugins() → Vec<ToolInfo>
      ↑ aidaguard-plugins

用户 点击 "Configure" / "Restore"
  → Plugin::configure(proxy_port) / Plugin::restore()
    ↑ aidaguard-plugins (纯文件操作，无 Tauri 依赖)
```

✅ plugins crate 完全独立，只依赖 core 的类型。

#### 流 6: 审计日志查询

```
用户 查看审计日志
  → Tauri command list_audit / get_audit_detail / get_audit_stats
    → Storage::list_filtered / get_by_id / stats()
      ↑ aidaguard-storage
```

✅ storage crate 完全独立。

#### 流 7: 上游管理

```
用户 添加/编辑/删除 上游
  → Tauri command add_upstream / update_upstream / delete_upstream
    → UpstreamManager::add / update / delete
      ↑ aidaguard-upstream
    → 持久化到 upstreams.json

用户 测试连通性
  → UpstreamClient::test_connectivity(url, api_key, timeout)
    ↑ aidaguard-upstream
```

✅ upstream crate 独立管理自己的持久化。

---

### 15.2 core 模块精确定义

经过业务流验证，`aidaguard-core` 的最终范围确定如下：

```
crates/aidaguard-core/
├── Cargo.toml                  ← deps: regex, serde, serde_yaml, toml, uuid, anyhow, tracing, tokio, notify
├── src/
│   ├── lib.rs                  ← re-exports
│   ├── types/
│   │   ├── mod.rs
│   │   ├── match_.rs           ← Match, Strategy, Mode
│   │   ├── rule.rs             ← RuleDef, RuleFile, CompiledRule, compile_regex()
│   │   ├── entity.rs           ← EntityType, EntityCategory (Phase 14+)
│   │   └── config.rs           ← Config, StorageConfig, NotificationConfig
│   ├── engine.rs               ← DetectionEngine trait
│   ├── detector.rs             ← Detector (基线 regex 实现, impls DetectionEngine)
│   ├── replacer.rs             ← replace(), restore(), PlaceholderMap
│   └── watcher.rs              ← watch_rules() (generic over DetectionEngine)
```

#### 每个模块的理由

| 模块 | 放在 core 的理由 | 不放的理由 |
|------|-----------------|-----------|
| **types/** | 所有 crate 都引用 Match/Strategy/RuleDef/Config，必须在最底层 | — |
| **engine.rs** | trait 是抽象接口，定义在类型旁边最合理 | trait 只有一个 impl，过度抽象 |
| **detector.rs** | 基线引擎，始终可用，所有 consumer 需要它作为默认 impl | 可移到 aidaguard-detector 作为 PatternRecognizer |
| **replacer.rs** | 纯函数，186 行，仅依赖 Match/Strategy，被 proxy 和 rules 使用 | 可移到 detector（作为 anonymizer 的一部分） |
| **watcher.rs** | 规则热加载，与 Detector 紧密配合 | 只有 tauri 调用，可移到 tauri |
| **config.rs** | 应用配置格式，被 main.rs 和 commands/config.rs 使用 | 只是 Tauri 设置页使用，可移到 tauri |

**最终裁决**：

- **replacer** — 留在 core。它是 Match 的配套工具，移到 detector 会让简单引用（如 test_rule）也依赖 detector。
- **watcher** — 留在 core。抽成 generic `watch_rules<E: DetectionEngine>`，未来旧 Detector 和新 AnalyzerEngine 都能用。
- **detector (旧)** — 留在 core。它是 DetectionEngine 的唯一实现，删掉后 tauri 无法启动。等 aidaguard-detector 的 AnalyzerEngine 完成后，可考虑废弃。
- **config** — 留在 core。虽然只有 tauri 用 Config 做持久化，但它定义了 config.toml 的 schema，是所有模块的配置来源。proxy 的 `start_with_state` 也接受 Config 参数。

### 15.3 关键设计决策

#### D1: `DetectionEngine::reload()` 的签名

```rust
pub trait DetectionEngine: Send + Sync {
    fn detect(&self, text: &str) -> Vec<Match>;
    fn rule_count(&self) -> usize;
    fn rule_name(&self, id: &str) -> Option<&str>;
    fn reload(&mut self, dir: &Path) -> Result<usize>;
}
```

`reload` 放在 trait 中（而不是外部函数），因为：
- 旧 Detector 和新 AnalyzerEngine 的 reload 逻辑不同（YAML vs YAML+NLP）
- commands/rules.rs 中 7 处调用 `detector.load_from_dir()`，需要统一接口
- watcher.rs 需要 generic reload

#### D2: Config 暂时不动

当前 `Config` 包含 `upstreams: Vec<UpstreamConfig>` 和 `target_url`、`api_key`。在迁移阶段：
- UpstreamManager 使用独立的 `upstreams.json` 持久化
- Config 的旧字段标记 `#[deprecated]`，但保留兼容
- 下一个大版本清理

这不影响架构，纯粹是兼容性考量。

#### D3: DetectionEvent 的位置

`DetectionEvent` 从 `aidaguard_core::proxy` 移动到 `aidaguard_proxy`。因为：
- 它是代理检测事件的广播载体，与 proxy 逻辑紧密相关
- 消费者（events.rs）已经在 tauri 层，tauri 直接依赖 proxy crate

#### D4: `start_with_state` 参数优化

当前接受整个 `Config`（20+ 字段），实际只用 5 个：

```rust
// Before (臃肿):
start_with_state(config: Config, ...)

// After (精简):
start_with_state(
    port: u16,
    max_body_size: usize,
    engine: Arc<RwLock<Box<dyn DetectionEngine>>>,
    storage: Option<Arc<Storage>>,
    event_tx: Option<Sender<DetectionEvent>>,
    shutdown_signal: F,
    upstream_name: String,
    api_key: String,
    target_url: String,
)
```

proxy crate 不再依赖 Config 类型。

### 15.4 AppState 变更

```rust
// Before:
pub struct AppState {
    pub detector: Arc<RwLock<Detector>>,
    // ...
}

// After:
pub struct AppState {
    pub engine: Arc<RwLock<Box<dyn DetectionEngine>>>,  // ← 重命名 + trait object
    // ...
}
```

受影响位置 (6 处)：
- `main.rs:76` — 构造: `Arc::new(RwLock::new(Box::new(Detector::new())))`
- `commands/proxy.rs:76,231` — 读写 engine
- `commands/rules.rs:148,176,206,265,323,542` — engine.write().await.reload()
- `proxy/server.rs:182,198,301,352` — engine.read().await.detect() / rule_count()

全部是机械替换，无逻辑变更。

### 15.5 各 crate 依赖确认

| Crate | 直接依赖 |
|------|---------|
| **aidaguard-core** | regex, serde, serde_yaml, toml, uuid, anyhow, tracing, tokio, notify |
| **aidaguard-storage** | aidaguard-core, rusqlite, aes-gcm, pbkdf2, sha2, rand, serde_json, tracing |
| **aidaguard-plugins** | aidaguard-core, serde, serde_json, tracing |
| **aidaguard-upstream** | aidaguard-core, serde, serde_yaml, reqwest, tracing |
| **aidaguard-detector** | aidaguard-core (仅 skeleton) |
| **aidaguard-proxy** | aidaguard-core, aidaguard-upstream, aidaguard-storage, axum, reqwest, tokio, tracing |
| **aidaguard-tauri** | 以上全部 + tauri, tauri-plugin-notification, csv, tracing-subscriber |

### 15.6 验证清单

迁移完成后逐项验证：

| # | 验证项 | 方法 |
|---|--------|------|
| 1 | `cargo build --workspace` 通过 | CI |
| 2 | `cargo test --workspace` 全部通过 | 25 个现有测试 + 新增 |
| 3 | `npx tsc --noEmit` 零错误 | 前端类型检查 |
| 4 | `cargo tauri dev` 启动正常 | 手动 |
| 5 | 启动代理 → 发送测试请求 → 检测 → 审计记录 | 手动端到端 |
| 6 | 规则 CRUD：新建/编辑/删除/切换/测试 | 手动 |
| 7 | 工具配置：检测/配置/还原 | 手动 |
| 8 | 上游管理：添加/编辑/删除/连接测试/切换默认 | 手动 |
| 9 | 设置保存 & 重启后恢复 | 手动 |
| 10 | 热加载：修改 YAML 规则文件 → 代理自动重载 | 手动 |

---

## Phase 16 — v0.3.0 测试用例设计 (2026-05-05)

### 16.1 版本信息

Workspace 版本号升级至 `0.3.0`（`Cargo.toml` line 10）。

### 16.2 现有测试盘点

迁移前共有 **25 个单元测试**，均通过：

| 模块 (当前) | 测试数 | 目标 crate (迁移后) |
|------------|--------|-------------------|
| `detector/mod.rs` | 8 | aidaguard-core |
| `replacer/mod.rs` | 7 | aidaguard-core |
| `storage/mod.rs` | 5 | aidaguard-storage |
| `proxy/stream.rs` | 5 | aidaguard-proxy |
| **合计** | **25** | — |

所有现有测试在迁移后继续保留，仅 `use` 路径随 crate 移动更新。

---

### 16.3 测试用例总览

| Crate | 测试数 | 状态 | 优先级 |
|-------|--------|------|--------|
| aidaguard-core | 43 | 15 已有 + 28 新增 | P0 |
| aidaguard-storage | 15 | 5 已有 + 10 新增 | P0 |
| aidaguard-plugins | 18 | 0 已有 + 18 新增 | P0 |
| aidaguard-upstream | 22 | 0 已有 + 22 新增 | P0 |
| aidaguard-proxy | 17 | 5 已有 + 12 新增 | P1 |
| aidaguard-detector | 0 | skeleton only | P2 |
| aidaguard-tauri | 0 | 手动端到端验证 | P2 |
| **合计** | **115** | — | — |

> P0 = 迁移完成即需要，P1 = 迁移后补充，P2 = 后续版本

---

### 16.4 aidaguard-core (43 tests)

#### 16.4.1 types/match.rs — 3 tests (NEW)

| # | 测试 | 验证点 |
|---|------|--------|
| T-CORE-01 | `test_match_creation` | Match struct 所有字段可构造，值正确 |
| T-CORE-02 | `test_strategy_serde` | Strategy::Placeholder → `"placeholder"`, Strategy::Mask → `"mask"` |
| T-CORE-03 | `test_mode_serde` | Mode::Detect → `"detect"`, Mode::Filter → `"filter"` |

#### 16.4.2 types/rule.rs — 5 tests (NEW)

| # | 测试 | 验证点 |
|---|------|--------|
| T-CORE-04 | `test_rule_def_deserialize` | YAML 最小字段 → RuleDef，默认值正确 (enabled=true, priority=100, strategy=placeholder, mode=filter) |
| T-CORE-05 | `test_rule_file_deserialize` | 多规则 YAML 文件 → RuleFile，rules 数组正确展开 |
| T-CORE-06 | `test_compile_regex_valid` | 合法 pattern → Ok(Regex) |
| T-CORE-07 | `test_compile_regex_invalid` | 非法 pattern → Err |
| T-CORE-08 | `test_compile_regex_size_limit` | pattern 超过 2000 字符 → Err |

#### 16.4.3 types/entity.rs — 6 tests (NEW)

| # | 测试 | 验证点 |
|---|------|--------|
| T-CORE-09 | `test_entity_type_display` | 每个 EntityType 变体的 Display 输出正确 |
| T-CORE-10 | `test_entity_category_assignment` | Structured 变体归入 EntityCategory::Structured 等 |
| T-CORE-11 | `test_entity_type_from_str` | "CREDIT_CARD" → EntityType::CreditCard |
| T-CORE-12 | `test_entity_type_serde` | EntityType 序列化 → 反序列化 roundtrip |
| T-CORE-13 | `test_entity_type_all_regions` | 每个区域至少有一个对应的结构化实体 |
| T-CORE-14 | `test_custom_entity` | EntityType::Custom("my_type") 的 Display/Serde |

#### 16.4.4 types/config.rs — 4 tests (NEW)

| # | 测试 | 验证点 |
|---|------|--------|
| T-CORE-15 | `test_config_defaults` | Config::default() 各字段默认值正确 |
| T-CORE-16 | `test_config_save_load_roundtrip` | Config → save_to → load_from → 值一致 |
| T-CORE-17 | `test_config_load_missing_file` | 文件不存在 → None |
| T-CORE-18 | `test_storage_config_defaults` | StorageConfig 默认 enabled=false |

#### 16.4.5 engine.rs — 2 tests (NEW)

| # | 测试 | 验证点 |
|---|------|--------|
| T-CORE-19 | `test_detector_impls_engine` | Detector 满足 `DetectionEngine` trait bound (编译期) |
| T-CORE-20 | `test_engine_trait_object` | `Box<dyn DetectionEngine>` 可构造并调用 detect |

#### 16.4.6 detector.rs — 13 tests (8 已有 + 5 NEW)

**已有测试（迁移）**：

| # | 测试 | 来源 |
|---|------|------|
| T-CORE-21 | `test_detect_phone` | 现有 — 手机号匹配 |
| T-CORE-22 | `test_detect_multiple` | 现有 — 多个规则同时匹配 |
| T-CORE-23 | `test_no_match` | 现有 — 无敏感数据 |
| T-CORE-24 | `test_overlap_same_priority` | 现有 — 重叠去重 |
| T-CORE-25 | `test_id_card_with_x` | 现有 — 身份证校验位 X |
| T-CORE-26 | `test_deduplication` | 现有 — 同位置去重 |
| T-CORE-27 | `test_empty_input` | 现有 — 空文本 |
| T-CORE-28 | `test_email_exclude_retina` | 现有 — 排除正则在混合场景过滤 |

**新增**：

| # | 测试 | 验证点 |
|---|------|--------|
| T-CORE-29 | `test_detect_only_mode` | Mode::Detect 规则产生匹配但不进入 filter_hits |
| T-CORE-30 | `test_priority_ordering` | 高优先级规则先于低优先级匹配 |
| T-CORE-31 | `test_reload_from_dir` | load_from_dir 替换已有规则集 |
| T-CORE-32 | `test_reload_empty_dir` | 空目录 → 0 条规则 |
| T-CORE-33 | `test_add_rule_disabled` | 添加 disabled 规则后 detect 无匹配 |

#### 16.4.7 replacer.rs — 10 tests (7 已有 + 3 NEW)

**已有测试（迁移）**：

| # | 测试 | 来源 |
|---|------|------|
| T-CORE-34 | `test_replace_placeholder_single` | 现有 |
| T-CORE-35 | `test_replace_placeholder_multiple` | 现有 |
| T-CORE-36 | `test_replace_then_restore` | 现有 |
| T-CORE-37 | `test_mask_phone` | 现有 |
| T-CORE-38 | `test_mask_short` | 现有 |
| T-CORE-39 | `test_no_matches` | 现有 |
| T-CORE-40 | `test_restore_empty` | 现有 |

**新增**：

| # | 测试 | 验证点 |
|---|------|--------|
| T-CORE-41 | `test_mask_email` | 邮箱地址掩码保留首尾 (如 `t***@e***.com`) |
| T-CORE-42 | `test_replace_mixed_strategies` | 同文本中同时使用 Placeholder 和 Mask |
| T-CORE-43 | `test_placeholder_uniqueness` | 两次相同 match 生成不同的占位符 UUID |

---

### 16.5 aidaguard-storage (15 tests)

#### 已有测试（5 个，迁移）

| # | 测试 | 来源 |
|---|------|------|
| T-STO-01 | `test_record_and_list` | 现有 |
| T-STO-02 | `test_multiple_records` | 现有 |
| T-STO-03 | `test_empty_db` | 现有 |
| T-STO-04 | `test_encrypt_decrypt_roundtrip` | 现有 |
| T-STO-05 | `test_decrypt_invalid_data` | 现有 |

#### 新增测试 (10 个)

| # | 测试 | 验证点 |
|---|------|--------|
| T-STO-06 | `test_delete_record` | 删除单条 → count 减 1，get_by_id 返回 None |
| T-STO-07 | `test_list_filtered_by_rule_id` | rule_id_filter 过滤正确 |
| T-STO-08 | `test_list_filtered_by_path` | path_filter 过滤正确 |
| T-STO-09 | `test_list_filtered_by_date` | date_from_ms / date_to_ms 范围过滤 |
| T-STO-10 | `test_list_filtered_by_strategy` | strategy_filter 过滤正确 |
| T-STO-11 | `test_count_filtered` | count_filtered 返回过滤后总数 |
| T-STO-12 | `test_stats_today` | stats().today_count 正确 |
| T-STO-13 | `test_stats_rule_distribution` | stats().rule_distribution 聚合正确 |
| T-STO-14 | `test_stats_db_size` | stats().db_size_bytes > 0 |
| T-STO-15 | `test_migration_add_column` | 打开旧 schema DB 自动 ALTER TABLE ADD COLUMN |

---

### 16.6 aidaguard-plugins (18 tests, 全部 NEW)

#### 16.6.1 plugin.rs — 5 tests

| # | 测试 | 验证点 |
|---|------|--------|
| T-PLG-01 | `test_manifest_serialization` | PluginManifest → JSON → PluginManifest roundtrip |
| T-PLG-02 | `test_manifest_required_fields` | id/name/version 必填 |
| T-PLG-03 | `test_tool_adapter_trait` | 编译期验证 Trait 方法签名 |
| T-PLG-04 | `test_tool_info_from_manifest` | PluginManifest → ToolInfo 转换正确 |
| T-PLG-05 | `test_plugin_categories` | categories 字段序列化/反序列化 |

#### 16.6.2 registry.rs — 7 tests

| # | 测试 | 验证点 |
|---|------|--------|
| T-PLG-06 | `test_register_plugin` | register → all_plugins 包含该插件 |
| T-PLG-07 | `test_enable_disable_plugin` | disable → is_enabled=false, enable → is_enabled=true |
| T-PLG-08 | `test_disable_twice_ok` | 重复 disable 不报错 |
| T-PLG-09 | `test_enable_twice_ok` | 重复 enable 不报错 |
| T-PLG-10 | `test_get_plugin_by_id` | get("claude_code") → Some |
| T-PLG-11 | `test_get_nonexistent` | get("nonexistent") → None |
| T-PLG-12 | `test_state_persistence` | disable → 新建 registry → is_enabled=false (持久化验证) |

#### 16.6.3 adapters — 6 tests

| # | 测试 | 验证点 |
|---|------|--------|
| T-PLG-13 | `test_adapter_detect_installed` | 模拟已安装工具 → detect 返回 true |
| T-PLG-14 | `test_adapter_detect_not_installed` | 工具未安装 → detect 返回 false |
| T-PLG-15 | `test_adapter_read_config` | 读取模拟配置文件 → ToolConfig |
| T-PLG-16 | `test_adapter_backup_restore` | configure → backup 存在 → restore → 配置恢复 |
| T-PLG-17 | `test_adapter_configure_twice_idempotent` | 连续配置两次不报错 |
| T-PLG-18 | `test_adapter_restore_no_backup` | 无备份时 restore → Err |

---

### 16.7 aidaguard-upstream (22 tests, 全部 NEW)

#### 16.7.1 types.rs — 4 tests

| # | 测试 | 验证点 |
|---|------|--------|
| T-UPS-01 | `test_protocol_type_serde` | "openai_compatible" / "anthropic_compatible" 反序列化 |
| T-UPS-02 | `test_auth_type_serde` | BearerToken / ApiKeyHeader 反序列化 |
| T-UPS-03 | `test_model_info_deserialize` | YAML → ModelInfo (带 capabilities 列表) |
| T-UPS-04 | `test_upstream_config_from_provider` | ProviderConfig + api_key → UpstreamConfig |

#### 16.7.2 provider.rs — 5 tests

| # | 测试 | 验证点 |
|---|------|--------|
| T-UPS-05 | `test_load_provider_yaml` | 解析 openai.yaml → ProviderConfig (8 fields) |
| T-UPS-06 | `test_load_all_builtin_providers` | providers/ 目录全部 YAML 可解析 |
| T-UPS-07 | `test_provider_yaml_missing_required` | 缺少 id 字段 → Err |
| T-UPS-08 | `test_provider_yaml_invalid_protocol` | 未知 protocol → Err |
| T-UPS-09 | `test_model_capabilities_filter` | ModelInfo.capabilities 包含 ["chat", "streaming"] |

#### 16.7.3 client.rs — 6 tests

| # | 测试 | 验证点 |
|---|------|--------|
| T-UPS-10 | `test_build_openai_request` | OpenAI 协议 → URL=`/v1/chat/completions`, Auth=`Bearer <key>` |
| T-UPS-11 | `test_build_anthropic_request` | Anthropic 协议 → URL=`/v1/messages`, Header=`x-api-key` |
| T-UPS-12 | `test_build_anthropic_request_with_version` | 包含 `anthropic-version` header |
| T-UPS-13 | `test_client_timeout` | 超时 → Err (不 hang) |
| T-UPS-14 | `test_chat_json_parse` | 模拟 LLM JSON 响应 → 正确解析为目标类型 |
| T-UPS-15 | `test_chat_json_parse_fenced` | ````json {...}```` 代码块 → 正确提取并解析 |

#### 16.7.4 connectivity.rs — 4 tests

| # | 测试 | 验证点 |
|---|------|--------|
| T-UPS-16 | `test_connectivity_success` | 模拟 200 响应 → Ok(TestResult) |
| T-UPS-17 | `test_connectivity_timeout` | 超时 → Err |
| T-UPS-18 | `test_connectivity_auth_error` | 401 → Err 带状态码 |
| T-UPS-19 | `test_connectivity_invalid_url` | 无效 URL → Err |

#### 16.7.5 manager.rs — 3 tests

| # | 测试 | 验证点 |
|---|------|--------|
| T-UPS-20 | `test_add_upstream` | 添加上游 → list 包含 |
| T-UPS-21 | `test_set_default_upstream` | set_default → 旧 default 取消，新 default 设置 |
| T-UPS-22 | `test_delete_upstream` | 删除 → list 不包含 |

---

### 16.8 aidaguard-proxy (17 tests)

#### 16.8.1 stream.rs — 5 tests (已有，迁移)

| # | 测试 | 来源 |
|---|------|------|
| T-PRX-01 | `test_find_safe_len_no_placeholder` | 现有 |
| T-PRX-02 | `test_find_safe_len_partial_prefix` | 现有 |
| T-PRX-03 | `test_find_safe_len_complete_placeholder` | 现有 |
| T-PRX-04 | `test_find_safe_len_complete_then_incomplete` | 现有 |
| T-PRX-05 | `test_find_safe_len_single_bracket` | 现有 |

#### 16.8.2 forwarder.rs — 4 tests (NEW)

| # | 测试 | 验证点 |
|---|------|--------|
| T-PRX-06 | `test_forward_headers_skip_hop_by_hop` | host/authorization/connection 等被过滤 |
| T-PRX-07 | `test_forward_headers_preserve_custom` | 自定义 header 保留 |
| T-PRX-08 | `test_extract_model_openai_format` | `{"model": "gpt-5"}` → "gpt-5" |
| T-PRX-09 | `test_extract_model_anthropic_format` | `{"model": "claude-opus-4-7"}` → "claude-opus-4-7" |

#### 16.8.3 server.rs — 8 tests (NEW)

| # | 测试 | 验证点 |
|---|------|--------|
| T-PRX-10 | `test_health_check` | GET /health → 200, status=ok, version/rules_count 存在 |
| T-PRX-11 | `test_reload_endpoint` | POST /reload → 200, rules_count 更新 |
| T-PRX-12 | `test_proxy_handler_no_sensitive_data` | 无敏感数据 → 直接转发，body 不变 |
| T-PRX-13 | `test_proxy_handler_sensitive_data_filter` | 敏感数据 Filter 模式 → 替换后转发 |
| T-PRX-14 | `test_proxy_handler_sensitive_data_detect` | 敏感数据 Detect 模式 → 不替换，原样转发 |
| T-PRX-15 | `test_proxy_body_too_large` | Content-Length > max → 413 |
| T-PRX-16 | `test_extract_client_name_user_agent` | User-Agent header → tool_name |
| T-PRX-17 | `test_extract_context` | 敏感数据周围 N 字符上下文提取 |

---

### 16.9 测试开发计划

#### 阶段 1: 核心类型测试 (配合 S6 core 重构)

| 步骤 | 内容 | 测试数 |
|------|------|--------|
| T1 | types/match.rs + types/rule.rs | 8 (T-CORE-01 ~ T-CORE-08) |
| T2 | types/entity.rs + types/config.rs | 10 (T-CORE-09 ~ T-CORE-18) |
| T3 | engine.rs + detector.rs 补充 | 7 (T-CORE-19~20, T-CORE-29~33) |
| T4 | replacer.rs 补充 | 3 (T-CORE-41~43) |

#### 阶段 2: 存储测试 (配合 S1 storage 拆分)

| 步骤 | 内容 | 测试数 |
|------|------|--------|
| T5 | storage 已有测试迁移 | 5 (T-STO-01~05) |
| T6 | storage 新增测试 | 10 (T-STO-06~15) |

#### 阶段 3: 插件测试 (配合 S2 plugins 拆分)

| 步骤 | 内容 | 测试数 |
|------|------|--------|
| T7 | plugin.rs + registry.rs | 12 (T-PLG-01~12) |
| T8 | adapters 测试 | 6 (T-PLG-13~18) |

#### 阶段 4: 上游测试 (配合 S4 upstream 新建)

| 步骤 | 内容 | 测试数 |
|------|------|--------|
| T9 | types.rs + provider.rs | 9 (T-UPS-01~09) |
| T10 | client.rs + connectivity.rs | 10 (T-UPS-10~19) |
| T11 | manager.rs | 3 (T-UPS-20~22) |

#### 阶段 5: 代理测试 (配合 S5 proxy 拆分)

| 步骤 | 内容 | 测试数 |
|------|------|--------|
| T12 | stream.rs 已有测试迁移 | 5 (T-PRX-01~05) |
| T13 | forwarder.rs + server.rs 新增 | 12 (T-PRX-06~17) |

#### 依赖关系

```
T1 ──→ T3 ──→ T4
  ──→ T2
            T5 ──→ T6
            T7 ──→ T8
            T9 ──→ T10 ──→ T11
            T12 ──→ T13
```

T1~T4、T5、T7、T9、T12 可以并行开始。

### 16.10 覆盖率目标

| Crate | 行覆盖率目标 | 分支覆盖率目标 |
|-------|------------|--------------|
| aidaguard-core | ≥ 90% | ≥ 85% |
| aidaguard-storage | ≥ 85% | ≥ 80% |
| aidaguard-plugins | ≥ 80% | ≥ 75% |
| aidaguard-upstream | ≥ 85% | ≥ 80% |
| aidaguard-proxy | ≥ 80% | ≥ 75% |
| aidaguard-detector | — (skeleton) | — |

### 16.11 运行命令

```bash
# 全部测试
cargo test --workspace

# 单个 crate
cargo test -p aidaguard-core
cargo test -p aidaguard-storage
cargo test -p aidaguard-plugins
cargo test -p aidaguard-upstream
cargo test -p aidaguard-proxy

# 带覆盖率 (需要 cargo-llvm-cov)
cargo llvm-cov --workspace --html
```
