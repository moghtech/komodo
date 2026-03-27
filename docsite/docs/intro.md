---
slug: /intro
---

# What is Komodo?

Komodo is a web application for managing servers, builds, deployments, and automated procedures.

- **Connect servers**. Monitor CPU, memory, and disk usage with alerts. Connect to shell sessions.
- **Deploy containers**. Create, start, stop, and redeploy Docker containers. View status, logs, and exec into shells.
- **Deploy compose stacks**. Define compose files in the UI, on the host, or in a git repo with auto-deploy on push.
- **Manage Docker Swarms**. Connect swarm managers and deploy services and stacks across your cluster.
- **Build images**. Define the dockerfile in UI or clone a git repo. Supports AWS EC2 spot instances for scalable build capacity.
- **Run automation**. Orchestrate multi-step workflows with Procedures and Actions. Schedule automations to run regularly.
- **Manage configuration**. shared variable and secret with interpolation across all resources.
- **Full audit trail**. every change is recorded, with who made it and when.

There is no limit to the number of servers you can connect, and there never will be.

## Architecture

Komodo is composed of two components: **Core** and **Periphery**.

| Component | Role |
|---|---|
| **Core** | Web server hosting the API and browser UI. All user interaction flows through Core. |
| **Periphery** | Small, stateless agent running on each connected server. Called by Core to perform actions, report system usage, and retrieve container logs. |

## API

Core exposes a REST and WebSocket API for programmatic access. Client libraries are available:

- [**Komodo CLI**](./ecosystem/cli.mdx)
- [**Rust crate**](https://crates.io/crates/komodo_client)
- [**NPM package**](https://www.npmjs.com/package/komodo_client)
- [**curl examples**](https://docs.rs/komodo_client/latest/komodo_client/api/index.html#curl-example)

## Permissioning

Komodo has a granular, role-based permissioning system so teams of developers, operators, and administrators can collaborate safely. See [Permissioning](/docs/configuration/permissioning) for details.

User sign-on supports **username/password** and **OAuth (GitHub, Google, and generic OIDC)**. See [Core Setup](./setup/index.mdx).

## Docker

Komodo uses [Docker](https://docs.docker.com/) as the container engine for building and deploying.

:::info
[Podman](https://podman.io/) is also supported via the `podman` → `docker` alias.
:::
