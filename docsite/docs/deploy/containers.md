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
| `environment`      | Environment variables in `KEY=value` format. Supports [variable interpolation](../configuration/variables.md) and `[[vault.<provider-slug>.<secret-key>]]` [external secret references](../configuration/secret-providers.md). | `""`             |
| `secret_sources`   | Scoped external secret provider selectors (`project_id`, `environment`, `secret_path`, `include_imports`, `expand_references`) overriding provider defaults. | `[]`             |
| `secret_bindings`  | Structured external secret bindings injected directly into container environment variables.                    | `[]`             |
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

## External Secret Providers

`Deployment` containers can consume external secrets managed under **Settings → Providers → Secrets** (see [Secret Providers](../configuration/secret-providers.md)):

- **Inline `[[vault.<provider-slug>.<secret-key>]]` references** inside `environment` (recommended as full pre-encoded values such as `DATABASE_URL=[[vault.infisical-prod.DATABASE_URL]]`, while also supporting mixed literal strings such as `DATABASE_URL=postgres://[[DB_USER]]:[[vault.infisical-prod.DB_PASSWORD]]@db:5432/app`).
- **Structured `secret_bindings`** mapping a remote secret key directly to a container environment variable with explicit `required`, `allow_empty`, and `conflict_policy` (`Fail` or `ReplaceExisting`) semantics.

```toml
[[deployment]]
name = "api-service"
[deployment.config]
server = "server-prod"
image.type = "Image"
image.params.image = "ghcr.io/myorg/api:latest"
environment = """
DATABASE_URL = [[vault.infisical-prod.DATABASE_URL]]
"""

[[deployment.config.secret_bindings]]
source = "infisical-prod"
remote_key = "STRIPE_WEBHOOK_SECRET"
variable = "STRIPE_WEBHOOK_SECRET"
required = true
allow_empty = false
conflict_policy = "Fail"
```

Resolved values are substituted byte-for-byte in a single pass (never URL-encoded, trimmed, or re-interpolated), shell-escaped safely on the target Periphery host, and masked (`<REDACTED>`) in Komodo `Update` logs, `InspectDeploymentContainer`, and `GetDeploymentLog`.

:::warning
External secret references are only permitted in `environment` and `secret_bindings` on Linux Server-targeted Deployments over verified TLS (`insecure_tls = false`). Referencing `[[vault...]]` in `ports`, `volumes`, `labels`, `command`, `extra_args`, or Swarm-targeted Deployments fails closed before execution.
:::

## Deploying to a Swarm

Instead of targeting a single Server, a Deployment can target a **Swarm** to deploy the container as a Swarm service. You can attach Swarm configs and secrets to the service. See [Swarm](../swarm.md) for details.
