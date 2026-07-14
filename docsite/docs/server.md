# Server

A Server is the attachment point between Komodo's control plane and a real machine. It represents a
managed host with a connected [Periphery](./periphery.md) agent. Resources such as
[Deployment](./deploy/containers.md), [Stack](./deploy/compose.md), [Repo](./repo.md), and
[Builder](./build.md#builders) attach to a Server because they need a host where that work can run.

## What A Server Represents

A Server is more than an inventory record. It gives Core a path to host-side execution. When Core
stores a Stack or Deployment configuration, that does not by itself make the workload runnable. It
becomes runnable when the resource is attached to a Server that has a connected Periphery agent and
access to the target runtime.

## Server And Periphery

Each Server depends on a connected Periphery agent. Periphery is the process that actually executes
work on the machine, reads host state, and reports back to Core. A Server without a working
Periphery connection can still exist in configuration, but it cannot execute resource operations on
that host.

That relationship also explains why logs, mounts, and filesystem paths have to be reasoned about
from the host and Periphery side, not only from Core's stored configuration. See
[Host Model](./host-model.md) for the host-side path and mount model.

## Identity And Connection

A Server also anchors connection identity. On first connection, onboarding creates or updates the
server record and Core stores the public key it expects for that server. On later reconnects,
`connect_as` and the persisted Periphery key need to match that existing server identity. See
[Connection Model](./connection-model.md).

## What Attaches To A Server

A Server is the target for the resource types that need direct host access:

- [Deployment](./deploy/containers.md)
- [Stack](./deploy/compose.md)
- [Repo](./repo.md)
- [Builder](./build.md#builders) when backed by a connected server

Those resources may be defined in Core, but they operate through the host environment exposed by the
Server and its Periphery agent.

## Host State And Operations

Server configuration also ties together several host-side concerns:

- connection to Periphery
- alert thresholds and health signals
- logs and runtime state
- filesystem visibility
- the host-side location for builds, checkouts, and deployments

When a deployment fails, check the attached Server first, then follow the host-side path and mount
assumptions through [Host Model](./host-model.md).

## Related Pages

- [What Is Komodo](./intro.md)
- [Periphery](./periphery.md)
- [Connection Model](./connection-model.md)
- [Repo](./repo.md)
- [Host Model](./host-model.md)
- [Install Komodo](./setup/install-komodo.mdx)
- [Connect More Servers](./setup/connect-servers.mdx)
- [Resources](./resources.md#server)
- [Setup](./setup)
