---
sidebar_label: Use Containerized Periphery
---

# Use Containerized Periphery Without Path Surprises

Containerized Periphery works, but it changes how Komodo sees the host. Periphery is now running
inside another filesystem boundary.

This matters most for:

- relative paths in [Docker Compose](https://docs.docker.com/compose/) files
- `Files On Server` stacks
- repo clone locations
- [bind mounts](https://docs.docker.com/engine/storage/bind-mounts/)
- any command that assumes the same path inside and outside the container

## Keep The Root Directory Path Identical

Keep the Periphery root directory mounted at the same path inside and outside the container.

If the host path is `/etc/komodo`, mount it as `/etc/komodo` inside the container as well. Do not
mount `/srv/komodo` on the host to `/etc/komodo` in the container and expect path-based behavior to
stay intuitive.

The sample Periphery compose file already models this:

```yaml
- ${PERIPHERY_ROOT_DIRECTORY:-/etc/komodo}:${PERIPHERY_ROOT_DIRECTORY:-/etc/komodo}
```

## Relative Paths Are The Main Trap

Relative paths that work when you run `docker compose up` directly on the host can behave
differently when Periphery runs inside a container. Maintainer guidance in the repo discussions is
clear on this point: absolute paths are more reliable with containerized Periphery, while
host-managed Periphery matches normal Docker Compose expectations more closely.

For reliable relative path behavior, prefer a host-managed Periphery. If Periphery stays
containerized, use absolute paths and matching root-directory mounts.

## Mount The Host Resources Periphery Needs

A containerized Periphery install needs these mounts:

- [`/var/run/docker.sock`](https://docs.docker.com/engine/security/protect-access/) for Docker
  access
- persistent key storage at `/config/keys`
- the Periphery root directory at the same path inside and outside the container
- `/proc` on Linux if you want host process visibility outside the container

On macOS, the sample `/proc:/proc` mount does not apply and should be removed.

## Files On Server And Stack Layout

When a Stack uses `Files On Server`, Komodo still resolves the compose path from the environment
visible to Periphery. If that environment is containerized, the compose file, relative mounts, and
root directory all need to make sense from inside the Periphery container.

That is why `Files On Server` mode is one of the first places where containerized Periphery feels
surprising.

## When To Switch To Host-Managed Periphery

Switch to a host-managed Periphery if:

- relative bind mounts keep behaving unexpectedly
- your compose files assume host-relative layouts
- repo clones need to land in normal host paths
- debugging path problems is taking more time than the containerized install saves

The host-managed path keeps the execution environment aligned with the host filesystem directly.

## Related Pages

- [Run Komodo Locally On macOS](./run-komodo-locally-on-macos.md)
- [Connect More Servers](../setup/connect-servers.mdx)
- [Periphery](../periphery.md)
- [Host Model](../host-model.md)
- [Docker Compose](../deploy/compose.md)
