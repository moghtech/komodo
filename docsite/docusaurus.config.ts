import { themes as prismThemes } from "prism-react-renderer";
import type { Config } from "@docusaurus/types";
import type * as Preset from "@docusaurus/preset-classic";

import dotenv from "dotenv";
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

  markdown: {
    hooks: {
      onBrokenMarkdownLinks: "warn",
    },
  },

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
        blog: false,
        theme: {
          customCss: "./src/css/custom.css",
        },
      } satisfies Preset.Options,
    ],
  ],

  themes: [
    [
      "@easyops-cn/docusaurus-search-local",
      {
        hashed: true,
        indexBlog: false,
        searchBarShortcutHint: false,
      },
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
      title: "KOMODO",
      hideOnScroll: true,
      logo: {
        alt: "monitor lizard",
        src: "img/komodo-512x512.png",
        width: "32px",
      },
      items: [
        {
          type: "docSidebar",
          sidebarId: "docs",
          position: "left",
          label: "Docs",
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
          label: "GitHub",
          position: "right",
        },
      ],
    },
    footer: {
      style: "dark",
      links: [
        {
          title: "Docs",
          items: [
            { label: "Docs Home", to: "/docs" },
            { label: "What Is Komodo", to: "/docs/intro" },
            { label: "Setup", to: "/docs/setup" },
            { label: "Resources", to: "/docs/resources" },
          ],
        },
        {
          title: "Ecosystem",
          items: [
            { label: "CLI", to: "/docs/ecosystem/cli" },
            { label: "API", to: "/docs/ecosystem/api" },
            { label: "Community", to: "/docs/ecosystem/community" },
          ],
        },
        {
          title: "Project",
          items: [
            { label: "GitHub", href: "https://github.com/moghtech/komodo" },
            { label: "Donate", href: "https://opencollective.com/komodo" },
            { label: "Demo", href: "https://demo.komo.do" },
          ],
        },
      ],
      copyright: "© 2026 Mogh Technologies Inc. Licensed under GPL-3.0",
    },
    prism: {
      theme: prismThemes.oneLight,
      darkTheme: prismThemes.oneDark,
      additionalLanguages: ["bash", "yaml", "toml", "rust", "json", "nginx"],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
