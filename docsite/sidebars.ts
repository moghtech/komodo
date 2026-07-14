import type { SidebarsConfig } from "@docusaurus/plugin-content-docs";

const sidebars: SidebarsConfig = {
  docs: [
    {
      type: "category",
      label: "Get Started",
      link: {
        type: "doc",
        id: "index",
      },
      items: [
        "intro",
      ],
    },
    {
      type: "category",
      label: "Setup",
      link: {
        type: "doc",
        id: "setup/index",
      },
      items: [
        "setup/install-komodo",
        "setup/connect-servers",
        "setup/after-first-server",
        "setup/onboarding-keys",
        "setup/mongo",
        "setup/ferretdb",
      ],
    },
    {
      type: "category",
      label: "Concepts",
      items: [
        "core",
        "database",
        "periphery",
        "server",
        "connection-model",
        "repo",
        "host-model",
        "alerter",
        "resources",
      ],
    },
    {
      type: "category",
      label: "Deploy And Build",
      items: [
        "deploy/compose",
        "deploy/containers",
        "build",
        "swarm",
        "terminals",
        "deploy/auto-update"
      ],
    },
    {
      type: "category",
      label: "Automate",
      items: [
        "automate/procedures",
        "automate/schedules",
        "automate/sync-resources",
        "automate/webhooks"
      ],
    },
    {
      type: "category",
      label: "Configuration",
      items: [
        "configuration/providers",
        "configuration/variables",
        "configuration/permissioning"
      ],
    },
    {
      type: "category",
      label: "How-To",
      link: {
        type: "doc",
        id: "how-to/index",
      },
      items: [
        "setup/backup",
        "how-to/add-another-server",
        "how-to/reverse-proxy-and-periphery-access",
        "how-to/git-backed-workflows",
        "how-to/komodo-cli",
        "how-to/oidc-oauth2",
        "how-to/custom-ca-certificates",
        "how-to/mounted-config-files",
        "how-to/docker-secrets-files",
        "how-to/encrypted-secrets-workflows",
        "how-to/containerized-periphery-paths",
        "how-to/connect-periphery-without-onboarding-key",
        "how-to/bootstrap-self-hosted-git-provider",
        "how-to/write-and-debug-actions",
        "how-to/run-komodo-locally-on-macos",
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
        "ecosystem/writing-guidelines",
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
