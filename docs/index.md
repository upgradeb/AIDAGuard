---
layout: home

hero:
  name: Aidaguard
  text: AI 时代的隐私守护者
  tagline: 本地 LLM API 网关，守护每一个 Token
  actions:
    - theme: brand
      text: 快速开始
      link: /ARCHITECTURE
    - theme: alt
      text: GitHub
      link: https://github.com/upgradeb/AIDAGuard

features:
  - title: 本地 API 网关
    details: 无需证书，只需将 API Base URL 指向 localhost:19000，所有数据本地处理
  - title: 智能检测
    details: 双引擎架构：YAML 规则 + 30+ 模式识别器，含 Luhn 校验、身份证校验、IBAN 验证
  - title: 无缝替换与还原
    details: 发送前替换敏感数据为占位符，响应时自动还原，对上层客户端完全透明
  - title: 流式支持
    details: 完整支持 SSE 流式响应，带滑动缓冲区还原，兼容所有主流 AI 工具
  - title: 多区域合规
    details: 按 PIPL/GDPR/CCPA/HIPAA 等合规框架组织规则，支持 cn/us/eu/gb 区域预设
  - title: 加密存储
    details: 映射表本地存储，AES-256-GCM 加密，PBKDF2 60 万次迭代
  - title: 工具适配
    details: 自动配置 Cursor、Claude Code、Aider 等 31+ AI 工具的代理设置
  - title: 可选 NLP
    details: BERT NER 命名实体识别，支持非结构化文本中的敏感数据检测
  - title: 跨平台
    details: 基于 Tauri 2.x，支持 macOS、Windows、Linux
---
