---
sidebar_label: Alerter
---

# Alerter

An Alerter routes alerts from Komodo to an external endpoint. It does not generate alerts by
itself. It decides which alerts to forward, where to send them, and when to suppress them.

Use an Alerter when Komodo should send notifications into Slack, Discord, ntfy, Pushover, or a
custom HTTP endpoint.

## What An Alerter Does

An Alerter combines:

- an endpoint type and destination
- optional filters for alert type
- optional resource include or exclude lists
- optional maintenance windows

That makes it the routing layer for alerts. The alert source stays with the resource or system
component that raised the alert.

## Endpoint Types

Komodo supports these endpoint types:

- `Custom`: POST the alert as JSON to an HTTP endpoint
- `Slack`: send to a Slack webhook URL
- `Discord`: send to a Discord webhook URL
- `Ntfy`: send to an ntfy topic URL, with optional email notifications
- `Pushover`: send to a Pushover endpoint URL with the app and user tokens in the query string

The default endpoint type is `Custom`, with the default URL `http://localhost:7000`.

## Filtering And Suppression

An Alerter can narrow what it sends:

- `enabled`: turn the alerter on or off
- `alert_types`: only send selected alert variants
- `resources`: only send alerts for selected resources
- `except_resources`: suppress alerts for selected resources
- `maintenance_windows`: suppress alerts during scheduled windows

If `alert_types` is empty, the Alerter accepts all alert types. If `resources` is empty, the
Alerter accepts alerts for all resources unless they are excluded by `except_resources`.

## Where Alerts Come From

Komodo raises alerts from several parts of the system. The alert types in source include:

- server unreachable, CPU, memory, disk, and version mismatch
- swarm unhealthy
- deployment state changes
- deployment image update available or auto-updated
- stack state changes
- build failures
- repo failures
- procedure and action failures
- scheduled run alerts and failures
- resource sync pending updates
- custom alerts and test alerts

That is why Alerters connect naturally to pages such as [Automatic Updates](./deploy/auto-update.md),
[Schedules](./automate/schedules.md), [Swarm](./swarm.md), and
[Resource Sync](./automate/sync-resources.md).

## Test And Custom Alerts

Komodo can send a test alert to confirm that an Alerter can reach its configured endpoint. The
Alerter must be enabled for that test to work.

Komodo can also send custom alerts through the same routing path. That is useful when Procedures,
Actions, or external automation should surface an operational event through the configured
notification channels.

## Example

```toml
[[alerter]]
name = "discord-alerts"
[alerter.config]
enabled = true
endpoint = { type = "Discord", params = { url = "https://discord.com/api/webhooks/..." } }
alert_types = ["ServerUnreachable", "DeploymentAutoUpdated"]
resources = []
except_resources = []
```

## Related Pages

- [Resources](./resources.md)
- [Automatic Updates](./deploy/auto-update.md)
- [Schedules](./automate/schedules.md)
- [Swarm](./swarm.md)
- [Resource Sync](./automate/sync-resources.md)
- [Community](./ecosystem/community.md)
