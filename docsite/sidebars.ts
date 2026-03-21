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
    "resources",
    {
      type: "category",
      label: "Deploy",
      collapsible: true,
      collapsed: false,
      items: [
        "deploy/containers",
        "deploy/compose",
        "deploy/auto-update",
      ],
    },
    "swarm",
    "build",
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
        "configuration/providers",
        "configuration/variables",
        "configuration/permissioning",
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
