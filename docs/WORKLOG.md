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

24 个单元测试全部通过：

- **detector**: 7 个（匹配、去重、空输入、重叠、身份证校验位）
- **replacer**: 6 个（替换、还原、掩码、空匹配）
- **proxy/stream**: 5 个（SSE 安全截断、部分占位符）
- **storage**: 6 个（加密往返、CRUD、空数据库、无效数据）

TypeScript 编译零错误。

---

## 项目结构总览

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
