# Host Model

Komodo does not replace the container runtime. It sits above it. [Core](./core.md) stores
configuration and dispatches work. [Periphery](./periphery.md) executes that work on the target
[Server](./server.md). The container runtime then performs the actual container, Compose, and build
operations.

## Container Runtime

Komodo assumes Docker by default. Podman is also supported through the `podman` -> `docker` alias.
From Komodo's point of view, the runtime lives on the managed host and is accessed through
Periphery, not through Core directly. [Install Komodo](./setup/install-komodo.mdx) covers the
bundled containerized path. [Connect More Servers](./setup/connect-servers.mdx) covers host-managed
Periphery installs.

## Linux Assumptions And Local Evaluation

Most host-integration examples assume a Linux host. That shows up in details such as `/proc`,
systemd service paths, Docker socket access, and direct host filesystem semantics.

Local evaluation on macOS still works, but it is not a full stand-in for a Linux host running
Docker directly. See [Run Komodo Locally On macOS](./how-to/run-komodo-locally-on-macos.md).

## Where Work Actually Happens

Several operations that may look like "Komodo operations" are really host-side operations:

- `docker run`
- `docker compose up`
- `docker build`
- git checkouts for repo-backed resources
- reading container logs
- resolving host filesystem paths and mounts

Core records and orchestrates those actions, but the host and its Periphery agent are the place
where they actually occur. The workload-facing pages are [Docker Compose](./deploy/compose.md),
[Containers](./deploy/containers.md), and [Build](./build.md).

## Paths, Mounts, And Visibility

A path in a Stack, Deployment, Build, or [Repo](./repo.md) is interpreted from the host-side
environment where Periphery runs.

If Periphery runs directly on the host, Komodo sees the host filesystem more directly. If Periphery
runs in a container, there is another layer between Komodo and the host. That changes what a local
path means, which mounts are visible, and where checked-out files actually live. See
[Periphery](./periphery.md), [Server](./server.md), and
[Connect More Servers](./setup/connect-servers.mdx).

## Git Checkouts And Build Context

Git-backed resources, build contexts, and generated runtime files follow the same model. Repos are
checked out on the target machine in the environment visible to Periphery and the runtime. Build
contexts and Compose files are interpreted from that same host-side environment. For resource-level
details, see [Repo](./repo.md), [Build](./build.md), [Docker Compose](./deploy/compose.md), and the
[Resources](./resources.md) reference. For git and registry credentials, see
[Providers](./configuration/providers.md).

## Operational Effects

This model affects several parts of Komodo at once:

- deployment behavior
- build behavior
- log collection
- path troubleshooting
- mount troubleshooting
- server selection

When a workload behaves differently than expected, check what the target host, Periphery, and
runtime can actually see.

## Related Pages

- [What Is Komodo](./intro.md)
- [Core](./core.md)
- [Periphery](./periphery.md)
- [Connection Model](./connection-model.md)
- [Server](./server.md)
- [Repo](./repo.md)
- [Setup](./setup)
- [Install Komodo](./setup/install-komodo.mdx)
- [Connect More Servers](./setup/connect-servers.mdx)
- [Docker Compose](./deploy/compose.md)
- [Build](./build.md)
- [Providers](./configuration/providers.md)
