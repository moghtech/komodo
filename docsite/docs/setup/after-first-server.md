---
slug: /setup/after-first-server
sidebar_label: After First Server
---

# After First Server

Once a Server reaches `OK`, Komodo is installed but not yet proven in your environment. The next
steps validate the host model, the first workload path, and the recovery path.

## Confirm The Server Model

Use the attached [Server](../server.md) to confirm that the host looks the way you expect:

- the Server status is `OK`
- the expected Docker runtime is visible
- the root directory path is the one you intend to use
- logs and container state are visible from Komodo

If any of those do not line up, read [Host Model](../host-model.md) and
[Periphery](../periphery.md) before creating more resources. Misaligned paths, mounts, or host
state here turn into deployment and git-checkout failures later.

## Deploy One Small Workload First

Deploy one small workload before introducing more automation.

- Use [Docker Compose](../deploy/compose.md) if you already have compose files or expect a
  multi-service workload.
- Use [Containers](../deploy/containers.md) for one container.
- Use [Build](../build.md) only if the first useful test also needs Komodo to produce the image.

This verifies that Core, Periphery, the runtime, and the selected host agree on what should run and
where it should run.

## Check The Operational Loop

After the first workload is up, verify the basic loop:

1. inspect the resource status in the UI
2. read logs
3. restart or redeploy once
4. open a terminal if the resource type supports it

That confirms the host-side execution path, runtime visibility, and permissions.

See [Terminals](../terminals.md) and [Permissioning](../configuration/permissioning.md) if those
checks expose gaps.

## Decide When To Introduce Git

Git-backed workflows add another layer of host and provider configuration.

Keep the first few resources UI-managed if you are still learning:

- where files live on the host
- which Server or Builder runs the work
- which provider account has to exist on Core or Periphery

Once that model is stable, move to [Configure Git-Backed Workflows](../how-to/git-backed-workflows.md),
[Repo](../repo.md), or [Build](../build.md).

## Add The Second Server Deliberately

After the first Server and first workload behave as expected, add the second machine. This is the
point where topology and connection choices start to matter.

Use:

- [Add Another Server](../how-to/add-another-server.md) for the shortest path
- [Connect More Servers](./connect-servers.mdx) for the full install and authentication details
- [Connection Model](../connection-model.md) if Core and Periphery will not live on the same
  private path

## Set Up Recovery Early

Before the environment expands, make sure database recovery is in place.

Use [Back Up And Restore](./backup.md) to confirm that backup scheduling, storage, and restore
behavior match the environment you are building.

## Related Pages

- [Configure Git-Backed Workflows](../how-to/git-backed-workflows.md)
- [Use A Reverse Proxy With Core And Periphery](../how-to/reverse-proxy-and-periphery-access.md)
- [Back Up And Restore](./backup.md)
- [Automatic Updates](../deploy/auto-update.md)
