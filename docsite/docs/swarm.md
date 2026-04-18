# Swarm

Komodo can manage Docker Swarm clusters through the `Swarm` resource. A Swarm resource points at
one or more manager nodes that already exist as connected [Server](./server.md) resources.

Use this when the target environment is an existing Docker Swarm rather than a single-host Docker
or Docker Compose deployment.

## Swarm Configuration

A Swarm resource points to one or more manager nodes. If one manager is unreachable, Komodo tries
the next one in the list.

```toml
[[swarm]]
name = "production-swarm"
[swarm.config]
server_ids = ["manager-01", "manager-02", "manager-03"]
send_unhealthy_alerts = true
```

### Config Fields

| Field | Description | Default |
| --- | --- | --- |
| `servers` | Swarm manager Server names or IDs. Tried sequentially on failure. | `[]` |
| `send_unhealthy_alerts` | Send alerts when nodes or tasks are unhealthy. | `true` |
| `maintenance_windows` | Scheduled windows during which alerts are suppressed. | `[]` |
| `links` | Quick links displayed in the resource header. | `[]` |

## Get Started

To use Docker Swarm with Komodo, initialize a swarm on one connected server first.

### Initialize The Swarm

Use a shell on the host, or a Komodo terminal, and run
[`docker swarm init`](https://docs.docker.com/reference/cli/docker/swarm/init/):

```bash
docker swarm init --advertise-addr <SERVER_IP>
```

### Create The Swarm Resource In Komodo

Create a Swarm resource and add the server as a manager node. Once created, Komodo detects the
swarm and shows its nodes.

### Join Additional Nodes

To add more servers, open the Swarm resource's node list in the UI and click **Join**. Komodo will
show the correct [`docker swarm join`](https://docs.docker.com/reference/cli/docker/swarm/join/)
command and token for the selected role.

There are separate join commands for worker and manager nodes.

:::tip
After a node joins, add it as a Server in Komodo and, for manager nodes, include it in the Swarm
resource's `servers` list for redundancy.
:::

## What Komodo Manages

### Nodes

Komodo shows node role, availability, status, and related cluster information.

### Services

Swarm services can be managed directly, or created from [Containers](./deploy/containers.md) by
configuring `swarm` instead of `server` in the Deployment config.

- view running, desired, and completed task counts
- inspect service configuration and attached configs or secrets
- view and search service logs

### Stacks

Compose-based stacks can be deployed to the swarm with `docker stack deploy`.
[Docker Compose](./deploy/compose.md) resources can target a Swarm by configuring `swarm` instead
of `server`.

- define compose files in the UI, on the host, or in git
- view the services and tasks that make up a stack

### Configs And Secrets

Komodo can manage Docker Swarm configs and secrets directly.

- create configs and secrets with labels and an optional template driver
- rotate them even though Docker treats them as immutable
- remove them when no longer needed

:::note
Docker Swarm configs and secrets have a maximum size of 500 KB.
:::

## Health Monitoring

Komodo tracks overall Swarm health:

| State | Meaning |
| --- | --- |
| **Healthy** | All nodes and tasks are OK. |
| **Unhealthy** | Some nodes or tasks do not match the desired state. |
| **Down** | All nodes or tasks are down. |
| **Unknown** | Status cannot be determined. |

When `send_unhealthy_alerts` is enabled, Komodo routes alerts through configured
[Alerters](./alerter.md).

## Related Pages

- [Server](./server.md)
- [Containers](./deploy/containers.md)
- [Docker Compose](./deploy/compose.md)
- [Alerter](./alerter.md)
- [Terminals](./terminals.md)
