# Swarm

Komodo can manage Docker Swarm clusters through the `Swarm` resource. Connect your swarm manager nodes and manage nodes, services, stacks, configs, and secrets from a single interface.

## Configuration

A Swarm resource points to one or more **manager nodes** — these are Servers that already have Periphery installed and are part of a Docker Swarm as managers. If one manager is unreachable, Komodo will try the next one in the list.

```toml
[[swarm]]
name = "production-swarm"
[swarm.config]
server_ids = ["manager-01", "manager-02", "manager-03"]
send_unhealthy_alerts = true
```

### Config fields

| Field | Description | Default |
|---|---|---|
| `servers` | List of Swarm manager Server names/IDs. Tries each sequentially on failure. | `[]` |
| `send_unhealthy_alerts` | Send alerts when nodes or tasks are unhealthy. | `true` |
| `maintenance_windows` | Scheduled windows during which alerts are suppressed. | `[]` |
| `links` | Quick links displayed in the resource header. | `[]` |

## Nodes

View all nodes in the swarm with their role, availability, status, platform, and resource usage.

## Services

Swarm services can be managed directly, or created from 
**Deployment** resources by configuring `swarm` instead of `server` in the Deployment config.

- View running, desired, and completed task counts.
- Inspect service configuration and attached configs/secrets.
- View and search service logs with grep support.

## Stacks

Deploy compose-based stacks to the swarm using `docker stack deploy`.
**Stack** resources can target a Swarm by configuring `swarm` instead of `server` in the Stack config.

- Define compose files in the UI, on the host, or in a git repo (same as regular Stacks).
- View the services and tasks that make up a stack.

## Configs and Secrets

Manage Docker Swarm configs and secrets directly from Komodo.

- **Create** configs and secrets with labels and an optional template driver.
- **Rotate** configs and secrets — since they are immutable in Docker, Komodo handles the rotation pattern automatically:
  1. Creates a temporary replacement.
  2. Updates all referencing services to use the temporary version.
  3. Removes and recreates the original with the new data.
  4. Updates services back to the original name.
- **Remove** configs and secrets.

:::note
Configs and secrets have a maximum size of 500KB.
:::

## Health Monitoring

Komodo tracks the overall health of each Swarm:

| State | Meaning |
|---|---|
| **Healthy** | All nodes and tasks are OK. |
| **Unhealthy** | Some nodes or tasks don't match the desired state. |
| **Down** | All nodes or tasks are down. |
| **Unknown** | Status cannot be determined. |

When `send_unhealthy_alerts` is enabled, Komodo will route alerts through your configured [Alerters](resources#alerter).

## Deploying to a Swarm

Both **Deployments** and **Stacks** can target a Swarm instead of a single Server:

- On a **Deployment**, set `swarm` to deploy the container as a Swarm service. You can attach swarm configs and secrets to the service.
- On a **Stack**, set `swarm` to deploy via `docker stack deploy` instead of `docker compose up`.
