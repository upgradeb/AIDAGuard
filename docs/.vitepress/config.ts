import { defineConfig } from 'vitepress'

export default defineConfig({
  lang: 'zh-CN',
  title: 'AIDAGuard',
  description: 'AI 时代的隐私守护者 — 本地 LLM API 网关',
  base: '/AIDAGuard/',
  ignoreDeadLinks: true,

  head: [['link', { rel: 'icon', href: '/logo.png' }]],

  themeConfig: {
    logo: '/logo.png',

    nav: [
      { text: '首页', link: '/' },
      { text: '文档', link: '/ARCHITECTURE' },
      {
        text: '参考',
        items: [
          { text: 'LLM 提供商', link: '/reference/llm-providers' },
          { text: '工具适配器', link: '/reference/tool-adapters' },
          { text: '适配器架构', link: '/reference/adapter-architecture' },
        ],
      },
      { text: 'GitHub', link: 'https://github.com/upgradeb/AIDAGuard' },
    ],

    sidebar: [
      {
        text: '指南',
        items: [
          { text: '简介', link: '/' },
          { text: '系统架构', link: '/ARCHITECTURE' },
          { text: '开发指南', link: '/DEVELOPMENT' },
          { text: 'UI 设计', link: '/UI_DESIGN' },
        ],
      },
      {
        text: '参考',
        items: [
          { text: 'LLM 提供商', link: '/reference/llm-providers' },
          { text: '工具适配器', link: '/reference/tool-adapters' },
          { text: '适配器架构', link: '/reference/adapter-architecture' },
          { text: '自定义代理配置', link: '/reference/custom-agent-config' },
        ],
      },
      {
        text: '其他',
        items: [{ text: '开发记录', link: '/WORKLOG' }],
      },
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/upgradeb/AIDAGuard' },
    ],

    search: {
      provider: 'local',
    },
  },
})
