import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

import dotenv from "dotenv"
dotenv.config();

const config: Config = {
  title: "Komodo",
  tagline: "Build and deployment system",
  favicon: "img/favicon.ico",

  // Set the production url of your site here
  url: "https://komo.do",
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  // baseUrl: "/komodo/",
  baseUrl: "/",

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: "moghtech", // Usually your GitHub org/user name.
  projectName: "komodo", // Usually your repo name.
  trailingSlash: false,
  deploymentBranch: "gh-pages-docs",

  onBrokenLinks: "throw",
  onBrokenMarkdownLinks: "warn",

  // Even if you don't use internationalization, you can use this field to set
  // useful metadata like html lang. For example, if your site is Chinese, you
  // may want to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: "en",
    locales: ["en"],
  },

  presets: [
    [
      "classic",
      {
        docs: {
          sidebarPath: "./sidebars.ts",
          editUrl: "https://github.com/moghtech/komodo/tree/main/docsite",
        },
        blog: {
          showReadingTime: true,
          editUrl: "https://github.com/moghtech/komodo/tree/main/docsite",
        },
        theme: {
          customCss: "./src/css/custom.css",
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: "img/monitor-lizard.png",
    docs: {
      sidebar: {
        autoCollapseCategories: true,
      },
    },
    navbar: {
      title: "Komodo",
      logo: {
        alt: "monitor lizard",
        src: "img/komodo-512x512.png",
        width: "34px",
      },
      items: [
        {
          type: "docSidebar",
          sidebarId: "docs",
          position: "left",
          label: "docs",
        },
        {
          href: "https://opencollective.com/komodo",
          label: "Donate",
          position: "right",
        },
        {
          href: "https://docs.rs/komodo_client/latest/komodo_client",
          label: "Docs.rs",
          position: "right",
        },
        {
          href: "https://github.com/moghtech/komodo",
          label: "Github",
          position: "right",
        },
      ],
    },
    footer: {
      style: "dark",
      copyright: `Built with Docusaurus`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ["bash", "yaml", "toml"],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
