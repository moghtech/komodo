---
slug: /setup
title: Setup
---

Start here with the main setup choices before the detailed installation pages.

One common shape is one Core instance, one database, one connected Server, and one first workload.

## Topology And Database

Single-host and multi-host are both normal. A single-host layout keeps Core, the database, and the
first managed server close together. A multi-host layout fits environments where machines are
already separated.

Komodo supports both MongoDB and FerretDB. MongoDB is the default database option and the one the
project documents most directly. FerretDB is a MongoDB-compatible layer backed by Postgres. Use it
when Postgres is the database you already operate or when MongoDB packaging is a poor fit for the
host. See [MongoDB](./mongo.mdx) and [FerretDB](./ferretdb.mdx).

## Periphery Deployment

Periphery can run under `systemd` or in a container. A host-managed Periphery runs in the same
filesystem and process space as the host runtime. A containerized Periphery runs through an extra
container boundary and therefore depends on mounts for the Docker socket, `/proc`, key storage, and
the configured root directory.

There are three common installation shapes:

- **Systemd, root install**: installs the binary to `/usr/local/bin`, the config to
  `/etc/komodo/periphery.config.toml`, and the service file to `/etc/systemd/system/periphery.service`.
  This is the direct host-level install.
- **Systemd, user install**: installs into the calling user's home directory, with config under
  `$HOME/.config/komodo` and the service file under `$HOME/.config/systemd/user`. This avoids a
  root-managed service, but the user must have access to Docker and write access to the configured
  root directory. To survive logout, it also needs `loginctl enable-linger`.
- **Container**: runs Periphery in Docker using the published container image. The configured root
  directory path must match inside and outside the container, or path and mount behavior diverges.

On macOS, the bundled containerized Periphery path is a local evaluation setup, but the
sample `/proc:/proc` bind mount does not apply there and should be removed.

Containerized Periphery adds another execution boundary between Komodo and the host. That changes
how filesystem layout, relative paths, and mounts are interpreted. See
[Periphery](../periphery.md), [Host Model](../host-model.md), and
[Use Containerized Periphery Without Path Surprises](../how-to/containerized-periphery-paths.md).

The bundled Compose path in [Install Komodo](./install-komodo.mdx) uses a containerized Periphery
by
default. If you switch to a host-managed Periphery, do not also run the bundled `periphery` service
from the sample Compose file.

[Connect More Servers](./connect-servers.mdx) covers the installation details and path differences.

## First Workload

After Core and Periphery are up, choose the first workload shape:

- [Docker Compose](../deploy/compose.md) for compose-based and multi-service workloads
- [Containers](../deploy/containers.md) for a single container managed directly
- [Build](../build.md) when Komodo should produce the image from source

## Configuration Source

UI-managed configuration keeps the first install focused on Core, Periphery, and host execution.
Git-backed configuration adds provider setup, repository placement, and webhook or polling
behavior. See
[Configure Git-Backed Workflows](../how-to/git-backed-workflows.md) and [Repo](../repo.md).

## Related Pages

- [Install Komodo](./install-komodo.mdx)
- [Connect More Servers](./connect-servers.mdx)
- [Onboarding Keys](./onboarding-keys.md)
- [After First Server](./after-first-server.md)
- [Docker Compose](../deploy/compose.md)
- [Containers](../deploy/containers.md)

```mdx-code-block
import DocCardList from '@theme/DocCardList';

<DocCardList />
```
