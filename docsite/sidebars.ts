import type { SidebarsConfig } from "@docusaurus/plugin-content-docs";

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
        "setup/advanced",
        "setup/connect-servers",
        "setup/backup",
        "setup/version-upgrades",
      ],
    },
    {
      type: "category",
      label: "Deploy",
      collapsible: true,
      collapsed: false,
      items: [
        {
          type: "category",
          label: "Containers",
          link: {
            type: "doc",
            id: "deploy/containers/index",
          },
          items: [
            "deploy/containers/configuration",
            "deploy/containers/lifetime-management",
          ],
        },
        "deploy/docker-compose",
        "deploy/auto-update",
      ],
    },
    {
      type: "category",
      label: "Build Images",
      link: {
        type: "doc",
        id: "build-images/index",
      },
      items: [
        "build-images/configuration",
        "build-images/builders",
        "build-images/pre-build",
        "build-images/versioning",
      ],
    },
    {
      type: "category",
      label: "Automate",
      collapsible: true,
      collapsed: false,
      items: [
        "automate/procedures",
        "automate/sync-resources",
        "automate/webhooks",
      ],
    },
    {
      type: "category",
      label: "Configuration",
      collapsible: true,
      collapsed: false,
      items: [
        "resources/variables",
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
