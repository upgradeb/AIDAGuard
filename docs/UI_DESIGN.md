# Aidaguard 桌面客户端 UI 设计

**版本：** 0.5.0
**技术栈：** Tauri 2.x + React 18 + shadcn/ui + Tailwind CSS + Zustand

---

## 功能关系图

```
                         ┌─────────────────────────┐
                         │      系统托盘             │
                         │  启动/停止 · 状态 · 退出   │
                         └───────────┬─────────────┘
                                     │ 控制代理生命周期
                  ┌──────────────────┼──────────────────┐
                  │                  │                  │
                  ▼                  ▼                  ▼
    ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
    │    仪表盘        │  │   大模型接入      │  │     设置         │
    │  Dashboard.tsx   │  │  Upstreams.tsx   │  │  Settings.tsx    │
    └────────┬────────┘  └───────┬─────────┘  └────────┬────────┘
             │                   │                     │
             ▼                   ▼                     ▼
    ┌─────────────────────────────────────────────────────────┐
    │                    代理核心 (aidaguard-core)               │
    │  detector · replacer · forwarder · stream · storage       │
    └────────┬──────────────┬──────────────────┬───────────────┘
             │              │                  │
             ▼              ▼                  ▼
    ┌───────────────┐ ┌───────────────┐ ┌───────────────┐
    │   审计记录      │ │   规则管理      │ │   工具配置      │
    │ AuditLog.tsx   │ │  Rules.tsx     │ │ ToolsConfig.tsx│
    └───────────────┘ └───────────────┘ └───────────────┘
```

---

## 页面结构

| 页面 | 文件 | 功能 |
|------|------|------|
| 仪表盘 | `Dashboard.tsx` | 代理状态、统计卡片、规则命中图表、实时事件流 |
| 大模型接入 | `Upstreams.tsx` | 上游 LLM 提供商管理 |
| 审计记录 | `AuditLog.tsx` | 检测记录查询、详情、导出 |
| 规则管理 | `Rules.tsx` | 规则 CRUD、测试、预设切换 |
| 工具配置 | `ToolsConfig.tsx` | AI 工具适配器配置 |
| 设置 | `Settings.tsx` | 代理、存储、外观、通知设置 |

---

## 组件清单

### 业务组件 (`src/components/`)

| 组件 | 文件 | 功能 |
|------|------|------|
| StatCard | `StatCard.tsx` | 统计卡片（检测次数、数据库大小等） |
| RuleHitChart | `RuleHitChart.tsx` | 规则命中分布饼图 |
| EventFeed | `EventFeed.tsx` | 实时检测事件流 |
| AuditTable | `AuditTable.tsx` | 审计记录表格 |
| AuditDetailPanel | `AuditDetailPanel.tsx` | 审计详情侧边栏 |
| RuleEditor | `RuleEditor.tsx` | 规则编辑表单 |
| RuleTestPanel | `RuleTestPanel.tsx` | 规则测试面板 |
| GenerateRuleModal | `GenerateRuleModal.tsx` | AI 生成规则对话框 |
| PresetSwitcher | `PresetSwitcher.tsx` | 规则预设切换器 |
| OperationGuide | `OperationGuide.tsx` | 操作指南 |
| ThemeSwitcher | `ThemeSwitcher.tsx` | 主题切换 |
| Logo | `Logo.tsx` | 应用 Logo |

### UI 组件 (`src/components/ui/`)

基于 shadcn/ui (Radix UI + Tailwind CSS)：

| 组件 | 说明 |
|------|------|
| `button` | 按钮 |
| `card` | 卡片容器 |
| `dialog` | 对话框 |
| `input` | 输入框 |
| `select` | 下拉选择 |
| `switch` | 开关 |
| `table` | 表格 |
| `tabs` | 标签页 |
| `textarea` | 多行文本 |
| `tooltip` | 提示 |
| `alert` | 警告提示 |
| `badge` | 徽章 |
| `checkbox` | 复选框 |
| `label` | 标签 |
| `separator` | 分隔线 |
| `skeleton` | 骨架屏 |
| `toggle` | 切换按钮 |

---

## 模块详细设计

### 一、系统托盘

```
系统托盘
├── 菜单项
│   ├── 代理状态指示（运行中 ● / 已停止 ○ / 出错 ✕）
│   ├── 启动代理
│   ├── 停止代理
│   ├── ──────────────
│   ├── 打开主窗口
│   └── 退出 Aidaguard
└── 状态
    ├── idle      — 代理未启动，灰色图标
    ├── running   — 代理运行中，绿色图标
    └── error     — 启动失败，红色图标
```

---

### 二、仪表盘 (Dashboard)

```
仪表盘
├── 顶部状态栏
│   ├── 代理状态徽章（运行中/已停止）
│   ├── 监听端口
│   ├── 上游名称
│   └── 运行时长
│
├── 统计卡片行 (StatCard × 4)
│   ├── 今日检测次数
│   ├── 本周检测次数
│   ├── 总计检测次数
│   └── 审计数据库大小
│
├── 规则命中分布 (RuleHitChart)
│   └── 饼图：按规则名聚合
│
└── 最近事件流 (EventFeed)
    ├── 实时 WebSocket 推送
    └── 点击跳转审计详情
```

---

### 三、大模型接入 (Upstreams)

```
大模型接入
├── 上游列表（左侧面板）
│   ├── 列表项：名称 · 默认标签 · 状态
│   └── 拖拽排序
│
├── 上游详情（右侧面板）
│   ├── 基本信息：名称、URL、API Key
│   ├── 模型管理：模型列表、自动拉取
│   ├── 高级设置：超时、速率限制
│   └── 连通性测试
│
└── 操作：添加、复制、删除、保存
```

---

### 四、审计记录 (AuditLog)

```
审计记录
├── 工具栏
│   ├── 搜索框
│   ├── 时间筛选
│   └── 导出 CSV/JSON
│
├── 记录列表 (AuditTable)
│   └── 列：时间 · 规则 · 策略 · 请求路径 · 状态码
│
└── 详情面板 (AuditDetailPanel)
    ├── 基本信息
    ├── 敏感数据（解密查看）
    └── 请求体预览
```

---

### 五、规则管理 (Rules)

```
规则管理
├── 工具栏
│   ├── 预设切换 (PresetSwitcher)
│   ├── 搜索框
│   ├── 添加规则
│   └── 重载规则
│
├── 规则列表
│   └── 列：状态 · 名称 · ID · 正则 · 策略 · 操作
│
├── 规则编辑 (RuleEditor)
│   └── 表单：ID、名称、正则、策略、优先级
│
└── 规则测试 (RuleTestPanel)
    ├── 输入测试文本
    └── 显示匹配结果
```

---

### 六、工具配置 (ToolsConfig)

```
工具配置
├── 工具列表
│   ├── 已配置：显示上游名称
│   └── 未配置：显示"未设置"
│
├── 配置面板
│   ├── 选择上游
│   └── 写入工具配置文件
│
└── 支持的工具
    ├── Claude Code
    ├── Aider
    ├── Codex CLI
    ├── Gemini CLI
    ├── JetBrains AI
    └── ...
```

---

### 七、设置 (Settings)

```
设置
├── 代理设置
│   ├── 监听端口
│   └── 请求体大小限制
│
├── 存储设置
│   ├── 启用审计记录
│   ├── 数据库路径
│   └── 加密密钥
│
├── 外观设置
│   └── 主题：浅色 / 深色 / 跟随系统
│
├── 通知设置
│   └── 启用桌面通知
│
└── 关于
    └── 版本号
```

---

## 前端技术栈

| 层 | 技术 | 版本 |
|----|------|------|
| 桌面框架 | Tauri | 2.x |
| 前端框架 | React | 18.x |
| 语言 | TypeScript | 6.x |
| UI 组件 | shadcn/ui (Radix UI) | - |
| 样式 | Tailwind CSS | 3.4.x |
| 图表 | Recharts | 2.x |
| 状态管理 | Zustand | 4.x |
| 表单 | React Hook Form + Zod | - |
| 日期 | date-fns | 4.x |
| 通知 | Sonner | 2.x |

---

## 项目结构

```
aidaguard-tauri/
├── src-tauri/                    # Rust 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs               # Tauri 入口
│       ├── commands/             # Tauri commands
│       │   ├── proxy.rs
│       │   ├── audit.rs
│       │   ├── rules.rs
│       │   ├── config.rs
│       │   └── upstream.rs
│       └── events.rs
│
└── src/                          # React 前端
    ├── main.tsx                  # 入口
    ├── App.tsx                   # 路由
    ├── pages/                    # 页面
    │   ├── Dashboard.tsx
    │   ├── Upstreams.tsx
    │   ├── AuditLog.tsx
    │   ├── Rules.tsx
    │   ├── ToolsConfig.tsx
    │   └── Settings.tsx
    ├── components/               # 业务组件
    │   ├── StatCard.tsx
    │   ├── RuleHitChart.tsx
    │   ├── EventFeed.tsx
    │   ├── AuditTable.tsx
    │   ├── AuditDetailPanel.tsx
    │   ├── RuleEditor.tsx
    │   ├── RuleTestPanel.tsx
    │   ├── GenerateRuleModal.tsx
    │   ├── PresetSwitcher.tsx
    │   ├── OperationGuide.tsx
    │   ├── ThemeSwitcher.tsx
    │   ├── Logo.tsx
    │   └── ui/                   # shadcn/ui 组件
    ├── store/                    # Zustand 状态
    │   └── useThemeStore.ts
    ├── i18n/                     # 国际化
    │   ├── en.ts
    │   └── zh.ts
    ├── lib/                      # 工具函数
    │   └── utils.ts
    └── globals.css               # Tailwind 样式
```

---

## Tauri Commands

| Command | 功能 |
|---------|------|
| `start_proxy` | 启动代理 |
| `stop_proxy` | 停止代理 |
| `get_health` | 获取代理状态 |
| `list_audit` | 查询审计记录 |
| `get_audit_detail` | 获取审计详情 |
| `get_rules` | 获取规则列表 |
| `save_rule` | 保存规则 |
| `delete_rule` | 删除规则 |
| `test_rule` | 测试规则 |
| `reload_rules` | 重载规则 |
| `get_config` | 获取配置 |
| `save_config` | 保存配置 |
| `test_upstream` | 测试上游连通性 |
