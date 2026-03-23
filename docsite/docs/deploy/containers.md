# Containers

Komodo can deploy Docker containers through the `Deployment` resource. It translates your configuration into a `docker run` command and executes it on the target Server via the Periphery agent.

## Configuration

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

### Config fields

| Field              | Description                                                                                                    | Default          |
| ------------------ | -------------------------------------------------------------------------------------------------------------- | ---------------- |
| `server`           | The Server to deploy on.                                                                                       | —                |
| `image`            | Docker image — either a custom image string or an attached Komodo Build.                                       | —                |
| `network`          | Docker network to connect to. `host` bypasses the virtual network layer.                                       | `host`           |
| `restart`          | Restart policy (`no`, `on-failure`, `always`, `unless-stopped`).                                               | `unless-stopped` |
| `ports`            | Port mappings when not using `host` network (e.g. `27018:27017`).                                              | `[]`             |
| `volumes`          | Bind mounts in `host_path:container_path` format.                                                              | `""`             |
| `environment`      | Environment variables in `KEY=value` format. Supports [variable interpolation](../configuration/variables.md). | `""`             |
| `labels`           | Docker labels in `key=value` format.                                                                           | `""`             |
| `command`          | Override the default image command. Appended after the image in `docker run`.                                  | `""`             |
| `extra_args`       | Additional flags passed directly to `docker run`.                                                              | `""`             |
| `send_alerts`      | Send alerts on container state changes.                                                                        | `true`           |
| `auto_update`      | Automatically redeploy when a newer image digest is available (same tag).                                      | `false`          |
| `poll_for_updates` | Check for newer images and show an update indicator (without auto-deploying).                                  | `false`          |
| `links`            | Quick links displayed in the resource header.                                                                  | `[]`             |

### Image source

There are two ways to specify the image:

- **Komodo Build** — attach a Build resource and Komodo deploys the latest (or a pinned) version. The registry credentials are inherited from the Build by default.
- **Custom image** — specify an image string directly, e.g. `mongo` or `ghcr.io/myorg/my-app:1.0.0`. Select a Docker registry account if the image is private.

## Container Lifecycle

Komodo tracks container state and provides actions to manage the lifecycle:

| Action                | Description                                                                                                                               |
| --------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| **Deploy / Redeploy** | Destroys the existing container (if any) and creates a new one with the current config. Config changes only take effect after a redeploy. |
| **Start**             | Starts a stopped container with its existing config.                                                                                      |
| **Stop**              | Stops the container but preserves its logs and state.                                                                                     |
| **Remove**            | Destroys the container entirely.                                                                                                          |

:::note
Stopping and starting a container does **not** apply config changes — you must redeploy for that.
:::

## Deploying to a Swarm

Instead of targeting a single Server, a Deployment can target a **Swarm** to deploy the container as a Swarm service. You can attach Swarm configs and secrets to the service. See [Swarm](../swarm.md) for details.
