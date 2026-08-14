import { defineConfig } from 'vitepress'

export default defineConfig({
  title: '档案管理OS',
  description: '任务提醒与物业档案管理工具操作文档',
  base: '/archivemanage/',
  lang: 'zh-CN',
  lastUpdated: true,
  themeConfig: {
    nav: [
      { text: '首页', link: '/' },
      { text: '操作指南', link: '/guide/' },
      { text: '常见问题', link: '/guide/faq' },
    ],
    sidebar: {
      '/guide/': [
        {
          text: '开始',
          items: [
            { text: '文档概述', link: '/guide/' },
            { text: '安装与启动', link: '/guide/install' },
          ],
        },
        {
          text: '功能模块',
          items: [
            { text: '仪表盘', link: '/guide/dashboard' },
            { text: '任务管理', link: '/guide/tasks' },
            { text: '人员管理', link: '/guide/members' },
            { text: '档案管理', link: '/guide/archives' },
            { text: '档案盒维护', link: '/guide/archive-boxes' },
            { text: '档案分类', link: '/guide/archive-categories' },
            { text: '档案标签', link: '/guide/archive-tags' },
            { text: '设置', link: '/guide/settings' },
          ],
        },
        {
          text: '其他',
          items: [
            { text: '常见问题', link: '/guide/faq' },
          ],
        },
      ],
    },
    socialLinks: [
      { icon: 'github', link: 'https://github.com/chivalry1314/archivemanage' },
    ],
    footer: {
      message: '档案管理OS 在线操作文档',
      copyright: 'Copyright © 2026',
    },
    search: {
      provider: 'local',
    },
  },
})
