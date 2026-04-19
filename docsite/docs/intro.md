---
slug: /intro
---

# What Is Komodo

Komodo is a control plane for managing container workloads across one or more servers. It does not
replace Docker or Podman. It sits above the container runtime and provides a web UI, API, and
resource model for describing what should run, where it should run, and how it should be updated.

## Core, Periphery, And The Runtime

The system has four main pieces. **Core** provides the web UI and API. The **database** stores
state, configuration, and history for Core. **Periphery** runs on each managed server and performs
host-side work on behalf of Core. The **container runtime** is Docker by default, with Podman
supported through the `podman` -> `docker` alias. It remains the thing that actually starts
containers, runs Compose projects, and builds images.

- [**Core**](./core.md) handles user interaction, state, orchestration, and policy.
- [**Database**](./database.md) stores state, configuration, and history for Core.
- [**Periphery**](./periphery.md) handles execution on the target host.
- [**Connection Model**](./connection-model.md) determines how Core and Periphery reach each other,
  how trust is bootstrapped, and which endpoint needs network exposure.
- [**Host Model**](./host-model.md) determines how host-side operations, paths, mounts, and
  container actions are interpreted.

When a Stack is deployed, Core sends the resolved configuration to Periphery on the attached
Server, and Periphery runs the corresponding Docker Compose commands on that machine. The same
execution boundary applies to logs, filesystem paths, git checkouts, and container lifecycle
operations. Komodo sees and performs those through Periphery, so where Periphery runs changes what
Komodo can access directly on the host.

The network side follows a similar split. Some installs have Periphery call Core. Others have Core
reach Periphery directly. Those choices affect onboarding, key persistence, reverse proxy setup,
and whether Periphery needs a public listener. See [Connection Model](./connection-model.md).

For installation, see [Install Komodo](./setup/install-komodo.mdx) and
[Connect More Servers](./setup/connect-servers.mdx). For the first workload, recovery steps, and
second-server setup, see [After First Server](./setup/after-first-server.md).

## Resource Model

Komodo exposes that execution model through **Resources**. A [**Server**](./server.md) represents a
real machine with a connected Periphery agent. It is the host-side attachment point for resources
that need to run work on a machine.

The primary workload resources are:

- [**Deployment**](./deploy/containers.md): a single container workload attached to a Server
- [**Stack**](./deploy/compose.md): a Docker Compose project for compose-based or multi-service
  workloads
- [**Build**](./build.md): a Docker image build from source rather than a deploy of an existing
  image from a registry

Supporting resources extend that model:

- [**Builder**](./build.md#builders): chooses where builds run
- [**Automation**](./automate/procedures.md): Procedures and Actions for multi-step workflows and
  scripted API work
- [**Resource Sync**](./automate/sync-resources.md): apply resource configuration declaratively from
  files in git
- [**Repo**](./repo.md): run git-backed scripts on a Server or Builder
- [**Alerter**](./alerter.md): route alerts to external systems

Across resource types, Komodo uses some common concepts: each resource has a name and id, resources
can be grouped with tags, and resources that need private git or registry access can use configured
[provider credentials](./configuration/providers.md). The full reference for resource types and
fields lives in [Resources](./resources.md).

## Topologies And Workflow Shape

Common topologies are:

- **Single host**: Core and the database run in one place, and Periphery runs on the same machine
  or on the first managed server.
- **Multi-host**: Core and the database run centrally while each managed machine runs its own
  Periphery.
- **Git-backed workflow**: compose files or application source live in git and are cloned onto the
  target host before deployment or build.

[Setup](./setup) compares those shapes against the database and Periphery installation choices.

## Further Reading

- [Setup](./setup) for installation shape and deployment choices.
- [Install Komodo](./setup/install-komodo.mdx) for the first-time install path.
- [After First Server](./setup/after-first-server.md) for the first workload,
  first recovery steps, and the jump to a second server.
- [Connection Model](./connection-model.md) for Core, Periphery, onboarding, and network shape.
- [Server](./server.md) for the host attachment model.
- [Repo](./repo.md) for git-backed checkouts, commands, and webhooks.
- [Alerter](./alerter.md) for alert routing and notification filters.
- [Host Model](./host-model.md) for paths, mounts, and host-side execution.
- [Resources](./resources.md) for the resource catalog.
