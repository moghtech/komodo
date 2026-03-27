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
      ],
    },
    "resources",
    {
      type: "category",
      label: "Deploy",
      items: [
        "deploy/compose",
        "deploy/containers",
        "deploy/auto-update",
      ],
    },
    "swarm",
    "terminals",
    "build",
    {
      type: "category",
      label: "Automate",
      items: [
        "automate/procedures",
        "automate/schedules",
        "automate/sync-resources",
        "automate/webhooks",
      ],
    },
    {
      type: "category",
      label: "Configuration",
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
    {
      type: "category",
      label: "Releases",
      items: ["releases/v2.0.0"],
    },
  ],
};

export default sidebars;
