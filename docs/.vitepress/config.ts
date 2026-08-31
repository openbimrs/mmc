import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'openbim-mmc',
  description: 'Safe, loss-aware MMC 2.0 archive and XML mechanics for Rust.',
  lang: 'en-US',
  base: '/mmc/',
  cleanUrls: true,
  lastUpdated: true,
  srcExclude: ['**/AGENTS.md', '**/PLAN.md', 'adr/_template.md'],
  rewrites: { 'ROADMAP.md': 'project/roadmap.md' },
  sitemap: { hostname: 'https://openbimrs.github.io/mmc/' },
  head: [
    ['meta', { name: 'theme-color', content: '#6d28d9' }],
    ['meta', { name: 'robots', content: 'index,follow' }],
  ],
  themeConfig: {
    logo: '/logo.svg',
    siteTitle: 'openbim-mmc',
    nav: [
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'Capabilities', link: '/capabilities' },
      { text: 'Architecture', link: '/architecture/' },
      { text: 'Security', link: '/security' },
      { text: 'API', link: '/api/rust' },
      { text: 'Roadmap', link: '/project/roadmap' },
    ],
    sidebar: {
      '/guide/': [
        {
          text: 'Guide',
          items: [{ text: 'Getting started', link: '/guide/getting-started' }],
        },
      ],
      '/architecture/': [
        {
          text: 'Architecture',
          items: [
            { text: 'Format boundary', link: '/architecture/' },
            { text: 'ADR 0001', link: '/adr/0001-format-boundary' },
          ],
        },
      ],
      '/api/': [
        {
          text: 'API reference',
          items: [
            { text: 'Rust API', link: '/api/rust' },
            { text: 'Generated docs', link: 'https://docs.rs/openbim-mmc' },
          ],
        },
      ],
      '/project/': [
        {
          text: 'Project',
          items: [{ text: 'Roadmap', link: '/project/roadmap' }],
        },
      ],
      '/': [
        {
          text: 'Start here',
          items: [
            { text: 'Overview', link: '/' },
            { text: 'Getting started', link: '/guide/getting-started' },
            { text: 'Capabilities', link: '/capabilities' },
            { text: 'Security', link: '/security' },
            { text: 'Roadmap', link: '/project/roadmap' },
          ],
        },
      ],
    },
    socialLinks: [{ icon: 'github', link: 'https://github.com/openbimrs/mmc' }],
    editLink: {
      pattern: 'https://github.com/openbimrs/mmc/edit/main/docs/:path',
      text: 'Edit this page on GitHub',
    },
    search: { provider: 'local' },
    footer: {
      message: 'AGPL-3.0-or-later licensed. Normative DIN material and public-prior-art schemas are not redistributed.',
      copyright: 'Copyright © 2026 openbimrs contributors',
    },
  },
})
