import type { SidebarsConfig } from "@docusaurus/plugin-content-docs";

/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */
const sidebars: SidebarsConfig = {
  docs: [
    "intro",
    {
      type: "category",
      label: "Setup",
      link: {
        type: "doc",
        id: "setup/index",
      },
      items: [
        "setup/mongo",
        "setup/ferretdb",
        "setup/connect-servers",
        "setup/backup",
        "setup/advanced",
        "setup/version-upgrades",
      ],
    },
    {
      type: "category",
      label: "Resources",
      link: {
        type: "doc",
        id: "resources/index",
      },
      items: [
        {
          type: "category",
          label: "Build Images",
          link: {
            type: "doc",
            id: "resources/build-images/index",
          },
          items: [
            "resources/build-images/configuration",
            "resources/build-images/pre-build",
            "resources/build-images/builders",
            "resources/build-images/versioning",
          ],
        },
        {
          type: "category",
          label: "Deploy Containers",
          link: {
            type: "doc",
            id: "resources/deploy-containers/index",
          },
          items: [
            "resources/deploy-containers/configuration",
            "resources/deploy-containers/lifetime-management",
          ],
        },
        "resources/docker-compose",
        "resources/auto-update",
        "resources/variables",
        "resources/procedures",
        "resources/sync-resources",
        "resources/webhooks",
        "resources/permissioning",
      ],
    },
    {
      type: "category",
      label: "Ecosystem",
      link: {
        type: "doc",
        id: "ecosystem/index",
      },
      items: [
        "ecosystem/cli",
        "ecosystem/api",
        "ecosystem/community",
        "ecosystem/development",
      ],
    },
  ],
};

export default sidebars;
