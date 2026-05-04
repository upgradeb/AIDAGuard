# Aidaguard 开发工作记录

## 项目概述

Aidaguard 是一个本地 API 网关代理，部署在 AI 客户端与大模型 API 之间，自动检测请求中的敏感数据（手机号、身份证、银行卡等），替换为占位符后再转发给大模型，并在响应中将占位符还原为原始数据。同时提供 Tauri 2.x 桌面应用进行可视化管理。

**技术栈：** Rust, Tokio, Axum 0.7, Reqwest 0.12, Tauri 2.x, React 18, Ant Design 5, Zustand, TypeScript

---

## Git 提交历史

| 提交 | 描述 |
|------|------|
| `65240c8` | MVP — 本地 API 网关代理，敏感数据检测与流式占位符还原 |
| `c448014` | 配置系统、加密审计存储、健康检查端点、HTTP 转发器 |
| `ada046f` | 代理增强：请求体大小限制、reasoning_content 还原、规则重载 API |
| `9c75f9c` | Tauri 2.x 桌面应用 + aidaguard-core 安全加固 |
| `81aedda` | 开发工作记录文档 |
| `9fb8089` | 易用性改进：设置页解耦上游配置、规则目录关联配置、清理测试数据 |

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
