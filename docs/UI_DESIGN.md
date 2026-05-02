# Aidaguard 桌面客户端 UI 设计

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
    │  只读统计视图     │  │  管理上游 LLM    │  │  全局参数配置     │
    └────────┬────────┘  └───────┬─────────┘  └────────┬────────┘
             │                   │                     │
             │ 统计数据来源       │ 提供上游配置给代理    │ 配置持久化
             │                   │                     │
             ▼                   ▼                     ▼
    ┌─────────────────────────────────────────────────────────┐
    │                    代理核心 (aidaguard-core)               │
    │  detector · replacer · forwarder · stream · storage       │
    └────────┬──────────────┬──────────────────┬───────────────┘
             │              │                  │
             │ 检测事件      │ 审计写入          │ 规则读写
             ▼              ▼                  ▼
    ┌───────────────┐ ┌───────────────┐ ┌───────────────┐
    │    通知        │ │   审计记录      │ │   规则管理      │
    │  桌面弹窗提醒   │ │ 查看·搜索·导出  │ │ 增删改查·测试   │
    └───────────────┘ └───────────────┘ └───────────────┘
             ▲              │                  │
             │              │                  │
             └──────────────┴──────────────────┘
                    审计记录可关联到具体规则
```

---

## 模块详细分解

### 一、系统托盘

```
系统托盘
├── 菜单项
│   ├── 代理状态指示（运行中 ● / 已停止 ○ / 出错 ✕）
│   ├── 启动代理
│   ├── 停止代理
│   ├── ──────────────
│   ├── 最近检测: N 次
│   ├── 当前规则: N 条
│   ├── ──────────────
│   ├── 打开主窗口
│   └── 退出 Aidaguard
└── 状态
    ├── idle      — 代理未启动，灰色图标
    ├── running   — 代理运行中，绿色图标
    └── error     — 启动失败，红色图标（hover 显示错误原因）
```

**与其他模块关系：**
- 调用代理核心 `start(config)` / 进程 kill
- 显示仪表盘统计的快照数据
- 退出时检查是否有未保存的规则/配置变更

---

### 二、仪表盘

```
仪表盘
├── 顶部状态栏
│   ├── 代理状态徽章（运行中/已停止）
│   ├── 监听端口: 19000
│   ├── 上游名称: 千帆 / DeepSeek / ...
│   └── 运行时长: 2h 34m
│
├── 统计卡片行
│   ├── 今日检测次数 ───────── 关联审计记录（按时间筛选）
│   ├── 本周检测次数
│   ├── 总计检测次数
│   └── 审计数据库大小
│
├── 规则命中分布
│   ├── 饼图或柱状图
│   ├── 维度: 规则名
│   └── 时间范围切换: 今日 / 本周 / 全部
│       └── 关联审计记录（GROUP BY rule_id）
│
└── 最近事件流
    ├── 实时 WebSocket 推送（代理检测到敏感数据时广播）
    ├── 每条事件显示: 时间 · 规则名 · 策略 · 请求路径
    ├── 不显示敏感值原文
    └── 点击跳转到审计记录详情 ──→ 关联审计模块
```

**与其他模块关系：**
- 数据来源：storage 查询、health API、WebSocket 事件
- 规则命中分布 → 关联规则管理（点击跳转）
- 最近事件 → 关联审计记录详情
- 上游名称 → 来自大模型接入配置

---

### 三、大模型接入

```
大模型接入
├── 上游列表（左侧面板）
│   ├── 列表项（名称 · 默认标签 · 状态指示灯）
│   ├── 设为默认
│   └── 拖拽排序（优先级）
│
├── 上游详情（右侧面板）
│   ├── 基本信息
│   │   ├── 名称            — 如"千帆"
│   │   ├── API 基础 URL    — https://qianfan.baidubce.com/v2/coding
│   │   ├── API Key         — 密码输入框，可切换可见
│   │   └── 备注
│   │
│   ├── 模型管理
│   │   ├── 模型列表（表格: 模型名 · 状态 · 操作）
│   │   ├── 手动添加模型
│   │   └── 自动拉取模型列表（调用上游 /models API）
│   │
│   ├── 高级设置
│   │   ├── 请求超时（秒）
│   │   ├── 速率限制（QPS）
│   │   ├── 最大重试次数
│   │   └── 自定义请求头（KV 列表）
│   │
│   └── 连通性测试
│       ├── 发送测试 ping 请求
│       └── 显示结果（延迟 ms、模型列表、错误信息）
│
└── 操作按钮
    ├── 添加上游
    ├── 复制上游
    ├── 删除上游（确认对话框）
    └── 保存修改 ──→ 写入 config.toml [upstreams] section
```

**config.toml 对应结构：**
```toml
[[upstreams]]
name = "千帆"
url = "https://qianfan.baidubce.com/v2/coding"
api_key = "..."
default = true
timeout_secs = 300
rate_limit_qps = 0
models = ["ernie-4.0", "ernie-3.5"]

[[upstreams]]
name = "DeepSeek"
url = "https://api.deepseek.com/v1"
api_key = "..."
models = ["deepseek-chat", "deepseek-reasoner"]
```

**与其他模块关系：**
- 配置写入 config.toml → 代理核心读取
- 默认上游 → 仪表盘状态栏显示
- 连通性测试 → 调用代理核心 forwarder

---

### 四、审计记录

```
审计记录
├── 工具栏
│   ├── 搜索框（规则名、请求路径）
│   ├── 时间筛选
│   │   ├── 今天
│   │   ├── 近 7 天
│   │   ├── 近 30 天
│   │   └── 自定义范围（日期选择器）
│   └── 导出按钮 ──→ 导出 CSV / JSON
│
├── 记录列表（表格）
│   ├── 列: 时间 · 规则 · 策略 · 请求路径 · 状态码
│   ├── 分页（每页 20/50/100 条）
│   └── 点击行展开详情
│
└── 详情面板（展开行内嵌）
    ├── 基本信息
    │   ├── 记录 ID
    │   ├── 精确时间
    │   ├── 规则名称
    │   └── 响应状态码
    ├── 敏感数据
    │   ├── 占位符
    │   ├── 原始值（点击"解密查看"后显示，带确认）
    │   └── 上下文片段（解密后显示原文高亮匹配位置）
    ├── 请求体
    │   ├── 替换后的完整请求体（JSON 格式化显示）
    │   └── 复制按钮
    └── 操作
        └── 删除单条记录

导出格式:
  CSV:  id, time, rule_id, strategy, placeholder, request_path, response_status
  JSON: [{id, time, rule_id, strategy, placeholder, request_path, response_status}, ...]
  注意: original 和 context 不导出（始终加密）
```

**与其他模块关系：**
- 数据来源：storage.list() / storage.count()
- 规则筛选 → 关联规则管理（跳转到对应规则）
- 仪表盘统计 → 聚合查询审计表
- 详情解密 → 调用 storage 解密接口

---

### 五、规则管理

```
规则管理
├── 工具栏
│   ├── 分类筛选: 全部 / 通用 / 金融 / 医疗 / 自定义
│   ├── 搜索框（规则名、ID）
│   ├── 添加规则按钮
│   └── 重载规则按钮 ──→ POST /reload
│
├── 规则列表（表格）
│   ├── 列: 状态 · 名称 · ID · 正则模式 · 策略 · 优先级 · 操作
│   ├── 启用/禁用开关
│   └── 操作按钮: 编辑 · 删除 · 测试
│
├── 规则编辑对话框
│   ├── 基本信息
│   │   ├── 规则 ID（唯一标识，创建后不可改）
│   │   ├── 规则名称（人类可读）
│   │   ├── 正则表达式
│   │   ├── 策略选择: Placeholder / Mask
│   │   ├── 优先级（数字）
│   │   └── 所属分类
│   └── 保存 / 取消
│
└── 规则测试面板
    ├── 输入测试文本（多行）
    ├── "运行测试" 按钮
    └── 结果展示
        ├── 匹配数量
        ├── 每条匹配: 规则名 · 匹配文本 · 位置 · 替换结果
        └── 替换后文本预览

规则文件映射:
  rules/general.yaml  → 通用分类
  rules/finance.yaml  → 金融分类
  rules/medical.yaml  → 医疗分类
  rules/custom.yaml   → 自定义分类（新增文件）
```

**与其他模块关系：**
- 规则数据 → 写入 YAML 文件
- 重载 → POST /reload API
- 审计记录可跳转到对应规则
- 仪表盘规则分布 → 按规则聚合统计
- 测试面板 → 调用 detector.detect() + replacer.replace()

---

### 六、设置

```
设置
├── 代理设置
│   ├── 监听端口（默认 19000）
│   └── 请求体大小限制 MB（默认 10）
│
├── 存储设置
│   ├── 启用审计记录（开关）
│   ├── 数据库文件路径
│   ├── 加密密钥（密码输入框）
│   └── 数据库信息（位置 · 大小 · 记录数）
│
├── 日志设置
│   ├── 日志级别（下拉: trace / debug / info / warn / error）
│   └── 打开日志目录
│
├── 外观设置
│   ├── 主题: 浅色 / 深色 / 跟随系统
│   └── 实时预览切换
│
├── 通知设置
│   ├── 启用桌面通知（开关）
│   └── 通知过滤: 仅通知高危规则 / 全部规则
│
└── 关于
    ├── 版本号
    ├── 许可证
    └── 检查更新

保存策略:
  - 代理端口、请求体大小 → 需重启代理生效
  - 存储设置 → 需重启代理生效
  - 日志级别 → 运行时生效
  - 外观设置 → 立即生效（主题存储于本地 localStorage，无需写入 config.toml）
  - 通知设置 → 立即生效
```

**与其他模块关系：**
- 所有设置 → 写入 config.toml
- 代理设置变更 → 提示需要重启
- 存储设置 → 关联审计模块（启用/禁用影响记录写入）

---

### 七、通知

```
通知
├── 触发条件: 代理检测到敏感数据
├── 通知内容
│   ├── 标题: "Aidaguard 检测到敏感数据"
│   ├── 正文: "规则: {rule_id} | 请求: {path}"
│   └── 不包含敏感值原文
├── 点击行为: 打开主窗口 → 跳转到审计记录
└── 频率控制: 同一规则 1 分钟内最多通知 1 次（防刷屏）
```

**与其他模块关系：**
- 依赖设置模块的通知开关
- 点击跳转审计记录
- 数据来源：代理核心检测事件（WebSocket 或 IPC 推送）

---

## 前端技术选型（Tauri）

| 层 | 技术 | 说明 |
|----|------|------|
| 桌面框架 | Tauri 2.x | Rust 后端 + Web 前端 |
| 前端框架 | React + TypeScript | 组件化 UI |
| UI 组件库 | Ant Design / Radix | 表格、表单、图表 |
| 图表 | recharts / echarts | 仪表盘统计图 |
| 状态管理 | Zustand / Jotai | 轻量状态 |
| IPC 通信 | Tauri invoke / event | Rust ↔ React |
| 数据库查询 | 通过 Tauri command 调用 storage API | 不在前端直接操作 SQLite |
| 实时事件 | Tauri event system | 检测事件推送到前端 |

---

## Tauri 与 aidaguard-core 的关系

```
aidaguard-tauri/                 # Tauri 桌面应用
├── src-tauri/
│   ├── Cargo.toml               # 依赖 aidaguard-core
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs              # Tauri 入口，注册 commands
│       ├── commands/
│       │   ├── proxy.rs         # 启动/停止代理
│       │   ├── audit.rs         # 审计查询 command
│       │   ├── rules.rs         # 规则 CRUD command
│       │   ├── config.rs        # 配置读写 command
│       │   └── upstream.rs      # 上游管理 command
│       └── events.rs            # 检测事件广播
│
└── src/                         # React 前端
    ├── App.tsx
    ├── pages/
    │   ├── Dashboard.tsx
    │   ├── Upstreams.tsx
    │   ├── AuditLog.tsx
    │   ├── Rules.tsx
    │   └── Settings.tsx
    ├── components/
    └── hooks/
```

每个 Tauri command 封装 aidaguard-core 的对应 API：

| Command | 调用 Core API |
|---------|--------------|
| `start_proxy(config)` | `aidaguard_core::proxy::start(config)` |
| `stop_proxy()` | kill child process |
| `get_health()` | 内置 health 查询 |
| `list_audit(limit, offset, filter)` | `storage.list()` / `storage.count()` |
| `get_audit_detail(id)` | `storage.list()` with single record |
| `get_rules()` | 读取 YAML 文件 |
| `save_rule(rule)` | 写入 YAML 文件 |
| `delete_rule(id)` | 编辑 YAML 文件 |
| `test_rule(pattern, text)` | `detector.detect()` |
| `reload_rules()` | POST /reload |
| `get_config()` | `config::Config::load()` |
| `save_config(config)` | 写入 config.toml |
| `test_upstream(url, key)` | `forwarder.forward()` 测试请求 |
