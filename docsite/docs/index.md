---
slug: /
---

# Docs Home

Komodo is a web application for managing servers, container workloads, builds, and automation. It
combines host management, deployment workflows, monitoring, configuration, and change history
behind a web UI and API.

## Learn The System

- [What Is Komodo](./intro.md)
  - Core, Periphery, the runtime, the resource model, and common topologies.
- [Core](./core.md)
  - The central control plane, database relationship, and administrative role.
- [Database](./database.md)
  - The persistence layer, setup choices, and MongoDB versus FerretDB.
- [Periphery](./periphery.md)
  - The host-side agent, execution boundary, and placement tradeoffs.
- [Connection Model](./connection-model.md)
  - How Core and Periphery reach each other, how trust is bootstrapped, and when network exposure
    is needed.
- [Server](./server.md)
  - The host attachment point for workloads and host-side execution.
- [Host Model](./host-model.md)
  - How Docker, Podman, paths, mounts, and host visibility fit into Komodo.
- [Repo](./repo.md)
  - Git-backed checkouts, host-side commands, provider-backed access, and webhook-driven execution.
- [Alerter](./alerter.md)
  - Alert routing, notification endpoints, and filtering rules.
- [Resources](./resources.md)
  - Full reference for Server, Stack, Deployment, Build, Repo, Procedure, and related resource
    types.

## Set It Up

- [Setup](./setup/index.md)
  - Single host or multi-host, MongoDB or FerretDB, `systemd` or containerized Periphery, Stack or
    Deployment.
- [Install Komodo](./setup/install-komodo.mdx)
  - Main first-time install path, including the database-specific setup paths.
- [Connect More Servers](./setup/connect-servers.mdx)
  - Periphery installation, onboarding keys, and server connection.
- [Onboarding Keys](./setup/onboarding-keys.md)
  - Bootstrap trust, key options, and reconnect behavior for new servers.
- [After First Server](./setup/after-first-server.md)
  - Validate the host model, deploy a first workload, and decide when to add git-backed workflows.

## Run Workloads

- [Docker Compose](./deploy/compose.md)
  - Stack modes, file sources, deployment behavior, and Compose-specific rollout options.
- [Containers](./deploy/containers.md)
  - Deployment modes, image sources, lifecycle behavior, and single-container runtime options.
- [Build](./build.md)
  - Builders, source paths, image tagging, registries, and build behavior.
- [Swarm](./swarm.md)
  - Swarm managers, stacks, services, configs, and secrets.

## Automate And Integrate

- [Procedures](./automate/procedures.md)
  - Multi-step workflows and actions.
- [Schedules](./automate/schedules.md)
  - Time-based automation.
- [Webhooks](./automate/webhooks.md)
  - Triggering work from external systems.
- [Komodo CLI](./ecosystem/cli.mdx)
  - Typed command-line client.
- [API](./ecosystem/api.md)
  - REST and WebSocket API, Rust crate, TypeScript package, and curl examples.

## Configure And Administer

- [Providers](./configuration/providers.md)
  - Git and registry credentials.
- [Variables](./configuration/variables.md)
  - Shared variables and secrets.
- [Permissioning](./configuration/permissioning.md)
  - Role-based permissions, users, and access with username/password and OAuth/OIDC login support.
- [How-To](./how-to/index.md)
  - Focused guides for host setup, git-backed workflows, auth, secrets, and backup or restore.

Komodo uses [Docker](https://docs.docker.com/) as the container engine for building and deploying.
[Podman](https://podman.io/) is also supported via the `podman` -> `docker` alias.

## Ecosystem And Community

- [Community](./ecosystem/community.md)
  - Community tools, guides, and alerter integrations.
- [Development](./ecosystem/development.md)
  - Running Komodo from source and working on the docs or product locally.
- [Writing Guidelines](./ecosystem/writing-guidelines.md)
  - Baseline docs guidance for avoiding low-signal AI-generated text.
