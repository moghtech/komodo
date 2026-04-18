# Core

Core is the central control plane for a Komodo installation. It provides the web UI and API, stores
and reads state from the [Database](./database.md), evaluates configuration, and sends work to
[Periphery](./periphery.md) on the target [Server](./server.md). Users interact with Komodo through
Core. Resources are created, updated, tagged, queried, and audited there.

## What Core Does

Core is responsible for the parts of the system that need a global view:

- user access and authentication
- resource definitions and history
- orchestration and policy
- provider, variable, and permission management
- initiating work on attached servers

That makes Core the place where Komodo decides what should happen. It is not the component that
directly runs `docker compose`, `docker run`, or `docker build` on managed hosts. That work happens
through [Periphery](./periphery.md).

## Core And The Database

Core depends on a database for persistent state. The database holds configuration, users,
permissions, resource definitions, and history. If Core is the control plane, the database is the
persistent store behind it.

For installation, Komodo supports both [MongoDB](./setup/mongo.mdx) and
[FerretDB](./setup/ferretdb.mdx). The database choice changes how Core is deployed, but it does not
change the Core / Periphery execution model. See [Database](./database.md) for the database role,
setup choices, and connection model.

## Authentication And Access

Core is also where Komodo's user-facing identity model lives. Local username and password login,
initial admin creation, [OIDC / OAuth2](./how-to/oidc-oauth2.md), and
[permissions](./configuration/permissioning.md) are all Core concerns.

That makes `KOMODO_HOST` more than a convenience setting. It affects callback URLs, webhook URL
suggestions, and reverse-proxy behavior whenever Core is exposed behind a stable hostname.

## Core And Resources

Resources are defined in Core and then executed through the target runtime model. A
[Stack](./deploy/compose.md), [Deployment](./deploy/containers.md), or [Build](./build.md) is
configured in Core, attached to a target such as a Server or Builder, and then dispatched to the
appropriate host-side agent.

Core tracks which resources exist, how they relate to one another, which credentials they can use,
and where they should run.

## What Core Does Not Do Directly

Core does not replace the container runtime, and it does not execute host-side operations by
itself. It does not have direct access to every managed server's filesystem just because the UI can
display those resources. The host boundary still matters. Core instructs; [Periphery](./periphery.md)
executes.

This distinction matters when reasoning about logs, git clones, mounts, and filesystem paths. If a
path is needed for deployment, build, or runtime behavior, the relevant question is what the target
host and its Periphery can actually access. See [Host Model](./host-model.md) for the host-side
view of those operations.

## Operational Role

In small installations, Core may run close to the first managed server. In larger installations,
Core runs centrally while multiple Periphery agents connect from managed hosts. In either case,
Core is the shared control point for the installation, so its persistence and availability matter
more than any individual workload resource.

## Related Pages

- [What Is Komodo](./intro.md)
- [Database](./database.md)
- [Periphery](./periphery.md)
- [Connection Model](./connection-model.md)
- [Host Model](./host-model.md)
- [Install Komodo](./setup/install-komodo.mdx)
- [Providers](./configuration/providers.md), [Variables](./configuration/variables.md), and
  [Permissioning](./configuration/permissioning.md)
