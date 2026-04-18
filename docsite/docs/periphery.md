# Periphery

Periphery is the host-side agent in Komodo. A managed [Server](./server.md) runs Periphery, and
[Core](./core.md) sends work to it. If Core is the control plane, Periphery is the execution
boundary between Komodo and the target host.

## What Periphery Does

Periphery is responsible for work that has to happen on a specific machine:

- running container and Compose operations
- performing image builds
- cloning git repos onto the target host
- reading logs and host state
- interacting with files, paths, and mounts on that machine

This is why a [Server](./server.md) matters in Komodo's model. A Server is not only a
label for a machine. It represents a host with a connected Periphery agent that can execute
resource operations.

## The Execution Boundary

Komodo does not execute workloads directly from Core. It goes through Periphery on the target host.
That boundary affects more than deployment commands. It also affects which filesystem paths exist,
where git repositories are checked out, which mounts are valid, and how logs are collected.

In Komodo's model, host-side work runs where Periphery can execute it.

## Connection And Trust

Periphery also carries the host-side identity in the Core↔Periphery connection. New servers use an
[Onboarding Key](./setup/onboarding-keys.md) to bootstrap trust. Existing servers reconnect with
persisted key material and a matching server identity. See
[Connection Model](./connection-model.md) for the network and trust model behind those flows.

## Periphery And Resources

Several resource types depend on Periphery because they need host access:

- [Stack](./deploy/compose.md)
- [Deployment](./deploy/containers.md)
- [Build](./build.md)
- [Repo](./repo.md)
- [Builder](./build.md#builders) when backed by a connected server

Those resources may be configured in Core, but they are executed from the host environment attached
to the relevant Server or Builder.

## Periphery Placement

A host-managed Periphery runs in the host environment. Host filesystem paths, Docker mounts, and
checked-out files are all interpreted directly on that host.

Containerized Periphery adds another layer between Komodo and the host. That changes what "local
path" means from Komodo's point of view and can make mounts, relative paths, and checked-out files
harder to reason about.

See [Use Containerized Periphery Without Path Surprises](./how-to/containerized-periphery-paths.md)
for matching root-directory paths, relative mounts, and Linux-specific host mounts.

## Network And Topology Role

Each managed machine needs its own Periphery if Komodo is expected to run host-side work there. In a
single-host installation, Core and Periphery may run near each other. In a multi-host installation,
Core stays central while each managed server runs its own Periphery.

Without Periphery, Core can store configuration, but it cannot execute work on that host.

## Related Pages

- [What Is Komodo](./intro.md)
- [Core](./core.md)
- [Server](./server.md)
- [Connection Model](./connection-model.md)
- [Repo](./repo.md)
- [Host Model](./host-model.md)
- [Connect More Servers](./setup/connect-servers.mdx)
- [Use Containerized Periphery Without Path Surprises](./how-to/containerized-periphery-paths.md)
- [Setup](./setup)
