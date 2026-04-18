---
sidebar_label: Run Komodo Locally on macOS
---

# Run Komodo Locally On macOS

macOS works well for local evaluation, UI work, and configuration editing before workloads move to
Linux servers.

Some host details in the sample Compose files and Periphery behavior assume a Linux host.

## What Works On macOS

The bundled Compose install from [Install Komodo](../setup/install-komodo.mdx) gives you:

- Core in Docker
- one database backend in Docker
- the bundled Periphery container in the same stack

That is enough to explore the UI, create resources, and understand how Core, Periphery, and the
resource model fit together.

## What Is Different On macOS

The bundled Periphery container sample includes the bind mount `- /proc:/proc`. That mount is there
so Periphery can inspect host processes outside the container on Linux.

On macOS, remove or comment out that line before `docker compose up`. Komodo can run without it.

You should also expect some host metrics and filesystem details to be less representative than on a
real Linux target. The bundled local install is useful for evaluation, but it is not a perfect
stand-in for a Linux server running Docker directly.

## Periphery Choices On macOS

The `systemd` path does not apply on macOS. To run Periphery outside the bundled Compose stack, run
the binary directly under a macOS process manager such as `launchd`, or keep Periphery in a
container while you evaluate Komodo locally.

## Local Evaluation Pattern

A macOS evaluation flow looks like this:

1. Run [Install Komodo](../setup/install-komodo.mdx) locally with the bundled Periphery container.
2. Remove the sample `/proc:/proc` bind mount before startup.
3. Learn the UI, resources, and git or workflow model locally.
4. Add one or more Linux servers when you want real host behavior for builds, deployments, or
   filesystem-sensitive workloads.

This keeps the first install small while moving real work onto the Linux hosts Komodo targets most
directly.

## When To Move Work Off macOS

Move real workloads onto Linux servers when you need:

- host-accurate Docker and filesystem behavior
- `systemd`-managed Periphery
- more representative container metrics
- path and mount behavior that matches production hosts

At that point, local macOS can remain the place where you access the Komodo UI, while Linux servers
become the places where Periphery executes builds and deployments.

## Related Pages

- [Install Komodo](../setup/install-komodo.mdx)
- [Setup](../setup/index.md)
- [Connect More Servers](../setup/connect-servers.mdx)
- [Use Containerized Periphery Without Path Surprises](./containerized-periphery-paths.md)
- [Add Another Server](./add-another-server.md)
