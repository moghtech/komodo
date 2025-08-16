import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

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
    "resources",
    {
      type: "category",
      label: "Setup Komodo Core",
      link: {
        type: "doc",
        id: "setup/index",
      },
      items: [
        "setup/mongo",
        "setup/ferretdb",
        "setup/backup",
        "setup/advanced",
      ],
    },
    "connect-servers",
    {
      type: "category",
      label: "Build Images",
      link: {
        type: "doc",
        id: "build-images/index",
      },
      items: [
        "build-images/configuration",
        "build-images/pre-build",
        "build-images/builders",
        "build-images/versioning",
      ],
    },
    {
      type: "category",
      label: "Deploy Containers",
      link: {
        type: "doc",
        id: "deploy-containers/index",
      },
      items: [
        "deploy-containers/configuration",
        "deploy-containers/lifetime-management",
        // "deploy-containers/choosing-builder",
        // "deploy-containers/versioning",
      ],
    },
    "docker-compose",
    "variables",
    "procedures",
    "permissioning",
    "sync-resources",
    "webhooks",
    "api",
    "cli",
    "version-upgrades",
    "development",
    "other-resources"
  ],
};

export default sidebars;
