# AIDAGuard 开发工作记录

**版本：** 0.5.0

## 项目概述

AIDAGuard 是一个本地 LLM API 代理，部署在 AI 客户端与大模型 API 之间，自动检测请求中的敏感数据（手机号、身份证、银行卡等），替换为占位符后再转发给大模型，并在响应中将占位符还原为原始数据。同时提供 Tauri 2.x 桌面应用进行可视化管理。

**技术栈：**
- 后端：Rust, Tokio, Axum 0.7, Reqwest 0.12
- 前端：Tauri 2.x, React 18, TypeScript, shadcn/ui, Tailwind CSS, Zustand

---

## 开发阶段

### MVP

- 本地 API 网关代理，敏感数据检测与流式占位符还原
- 核心 crate：`aidaguard-core`（类型、配置、检测器、替换器）、`aidaguard-proxy`（Axum 反向代理）、`aidaguard-storage`（加密审计存储）

### Phase 1: Tauri 桌面应用

- Tauri 2.x 项目脚手架
- 代理生命周期管理（启动/停止/状态查询）
- Dashboard 仪表盘：统计卡片、规则命中图表、实时事件流
- 系统托盘：空闲/运行/错误状态图标

### Phase 2: 审计日志

- 多条件过滤分页查询
- 审计记录详情查看
- CSV/JSON 导出

### Phase 3: 规则管理

- YAML 规则 CRUD（创建、编辑、删除、启用/禁用）
- 正则测试面板
- 规则文件热重载

### Phase 4: 工具适配器系统

- 声明式适配器引擎（JSON manifest）
- 25 个声明式适配器 + 6 个复杂适配器
- 支持 Cursor、Claude Code、Aider、Cline 等 31+ 工具
- 自动检测、配置、备份、还原

### Phase 5: UI 现代化

- 从 Ant Design 迁移到 shadcn/ui + Tailwind CSS
- 全局主题支持（浅色/深色/跟随系统）
- 大模型接入管理页面
- 规则预设与区域切换

### Phase 6: 多区域检测

- 按 PIPL/GDPR/CCPA/HIPAA 等合规框架组织规则
- 支持 cn/us/eu/gb 等区域规则预设
- 统一 YAML 规则架构，支持 validator、context_words、base_confidence 字段
- 30+ 内置模式识别器（含 Luhn 校验、身份证校验等）
- 可选 BERT NLP NER（`--features nlp`）
