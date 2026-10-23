# Automatic Updates

Komodo can automatically check for newer Docker image digests and redeploy resources when updates are found.

## Configuration

Both **Stacks** and **Deployments** support two update modes:

| Mode | Behavior |
|---|---|
| **Poll for Updates** | Checks for newer images with the same tag. Displays an update indicator in the UI and sends an alert (if an Alerter is configured). Does not redeploy. |
| **Auto Update** | Same check, but automatically redeploys services with newer images. Also sends an alert. |

:::note
Auto-update requires a "rolling" image tag like `:latest`. For pinned tags in git-sourced stacks, consider [Renovate](https://github.com/renovatebot/renovate).
:::

**Auto Update** also supports a **Minimum Update Age**, the number of hours an image must have existed in the registry before it will be deployed. This gives the community a window to catch and pull compromised or broken releases before they reach you. It only delays **Auto Update**, not the update indicator / alert. `0` disables the check. [Docker recommends a 3 day (72 hour) cooldown](https://www.docker.com/blog/defending-your-software-supply-chain-what-every-engineering-team-should-do-now/) as a sensible default.

For public **Docker Hub** images, the age comes from when the tag was pushed, which the publisher of the image cannot backdate. Everything else falls back to the image build time.

:::warning
The build time is written into the image by whoever built it, so an attacker able to publish to the registry can set it to any date. Outside of Docker Hub, **Minimum Update Age** guards against a broken release, not a compromised one.
:::

## Global Auto Update Procedure

New installs include a **Global Auto Update** Procedure, scheduled daily. It loops through all resources with either mode enabled and checks registries for newer digests.

```toml
[[procedure]]
name = "Global Auto Update"
description = "Pulls and auto updates Stacks and Deployments using 'poll_for_updates' or 'auto_update'."
tags = ["system"]
config.schedule = "Every day at 03:00"

[[procedure.config.stage]]
name = "Stage 1"
enabled = true
executions = [
  { execution.type = "GlobalAutoUpdate", execution.params = {}, enabled = true }
]
```

:::info
`GlobalAutoUpdate` can be integrated into other Procedures to coordinate timing with processes like backups. There is nothing special about the default Procedure — it is created for convenience.
:::
