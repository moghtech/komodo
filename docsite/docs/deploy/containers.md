---
sidebar_label: Deploy Containers
---

# Containers

Komodo deploys single-container workloads through the `Deployment` resource.

In server mode, a Deployment turns its configuration into a
[`docker run`](https://docs.docker.com/reference/cli/docker/container/run/) invocation on the
target [Server](../server.md). In swarm mode, the same resource becomes a Swarm service instead.

Use a Deployment when the workload is naturally one container. Use [Docker Compose](./compose.md)
when the workload should be expressed as a Compose project with one or more services.

## Choose The Deployment Mode

A Deployment has two execution modes:

- **Server mode**: set `server`. Komodo creates and manages one container on that host.
- **Swarm mode**: set `swarm`. If both `server` and `swarm` are set, `swarm` wins and the
  Deployment is created as a Swarm service.

## Choose The Image Source

A Deployment can run either:

- a direct image string, such as `ghcr.io/myorg/my-app:latest`
- a [Build](../build.md) resource produced by Komodo

That choice matters because it changes where versioning comes from and whether the deployment
should follow a Build automatically.

## Example

```toml
[[deployment]]
name = "my-app"
[deployment.config]
server = "server-prod"
image.type = "Image"
image.params.image = "ghcr.io/myorg/my-app:latest"
network = "host"
restart = "on-failure"
environment = """
DB_HOST = db.example.com
LOG_LEVEL = info
"""
volumes = """
/data/my-app/data:/app/data
/data/my-app/config:/app/config
"""
```

## Core Fields

| Field | Meaning | Default |
| --- | --- | --- |
| `server` | Server to deploy on in server mode. | — |
| `swarm` | Swarm to deploy on in Swarm mode. Overrides `server` when both are set. | — |
| `image` | Either a direct image string or a Komodo Build reference. | — |
| `image_registry_account` | Registry account used for `docker login` before pull. | `""` |
| `network` | Docker network attached to the container. | `host` |
| `restart` | Restart policy. | `unless-stopped` |
| `ports` | Host-to-container port mappings. Ignored on `host` network. | `""` |
| `volumes` | Host-to-container bind mounts. | `""` |
| `environment` | Container environment variables. | `""` |
| `labels` | Docker labels attached to the container or service. | `""` |
| `links` | Quick links shown in the resource header. | `[]` |
| `send_alerts` | Send container state alerts. | `true` |

## Image And Registry Behavior

When `image.type = "Image"`, Komodo deploys the image string directly.

When `image.type = "Build"`, Komodo deploys the image produced by the attached Build. A version of
`0.0.0` means the latest image from that Build.

`image_registry_account` controls the registry login used before image pull:

- empty means "use the Build's account if there is one, otherwise none"
- a configured account means a token for that account must be available

See [Build](../build.md) and [Providers](../configuration/providers.md).

## Runtime And Lifecycle Fields

| Field | Meaning | Default |
| --- | --- | --- |
| `command` | Command appended after the image name, or used to override the image command. | `""` |
| `extra_args` | Extra `docker run` or `docker service create` arguments. | `[]` |
| `termination_signal` | Signal used to stop the deployment. | Docker default |
| `termination_timeout` | Time to wait before forceful termination. | `10` |
| `term_signal_labels` | Labels keyed by termination signal behavior. | `""` |
| `replicas` | Service replica count in Swarm mode. | `1` |

The `extra_args` field covers container options that do not have a dedicated top-level field. It
also makes runtime behavior less obvious in the UI, so check it carefully during review.

## Build-Driven And Registry-Driven Updates

Deployments support both Build-driven and registry-driven update flows.

Build-driven:

- `redeploy_on_build`

Registry-driven:

- `poll_for_updates`
- `auto_update`

Use `redeploy_on_build` when a Komodo Build is the source of the image and the Deployment should
follow that Build automatically.

Use `poll_for_updates` or `auto_update` when the image comes from a registry tag that may move over
time.

See [Automatic Updates](./auto-update.md) and [Build](../build.md).

## How Redeploy Differs From Start And Stop

The container lifecycle actions are not interchangeable:

- **Deploy / Redeploy**: replace the existing container or service with the current configuration
- **Start**: start a stopped container with its existing configuration
- **Stop**: stop the container but keep its current state and logs
- **Remove**: delete the container entirely

If configuration changes, use redeploy. A plain stop/start cycle does not apply the changed
configuration.

## Network, Ports, And Host Assumptions

The default network is `host`, which means:

- port mappings are ignored on the `host` network
- the container sees the host network namespace directly
- behavior differs from a bridge-network deployment

That can be convenient for local services, but it also changes how the app is exposed and how port
conflicts behave. If you need explicit Docker port publishing, choose a non-host network and set
`ports`.

See [Host Model](../host-model.md).

## When A Deployment Fits Better Than A Stack

Choose a Deployment when:

- the workload is one container
- you do not need Compose as the deployment model
- the desired shape maps closely to one `docker run`
- the container should optionally follow one Build resource

Choose [Docker Compose](./compose.md) when the workload is naturally a Compose project or already
described by Compose files.

## Related Pages

- [Docker Compose](./compose.md)
- [Build](../build.md)
- [Swarm](../swarm.md)
- [Host Model](../host-model.md)
- [Automatic Updates](./auto-update.md)
