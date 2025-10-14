import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const config: Config = {
  title: 'Evaluate',
  tagline: 'An LLM Eval Framework made in Rust',
  favicon: 'img/favicon.ico',
  
  future: {
    v4: true,
  },
  
  url: 'https://your-docusaurus-site.example.com',
  baseUrl: '/',
  
  organizationName: 'RGGH',
  projectName: 'evaluate',
  
  onBrokenLinks: 'throw',
  
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },
  
  presets: [
    [
      'classic',
      {
        docs: {
          routeBasePath: '/', // Docs at root
          sidebarPath: require.resolve('./sidebars.js'),
        },
        blog: false,
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      } satisfies Preset.Options,
    ],
  ],
  
  themeConfig: {
    image: 'img/docusaurus-social-card.jpg',
    colorMode: {
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'Evaluate',
      logo: {
        alt: 'Evaluate Logo',
        src: 'img/logo.svg',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'tutorialSidebar',
          position: 'left',
          label: 'Docs',
        },
        {
          href: 'https://github.com/RGGH/evaluate',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {
              label: 'Getting Started',
              to: '/',
            },
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/RGGH/evaluate',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Evaluate. Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['rust', 'toml', 'bash'],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;