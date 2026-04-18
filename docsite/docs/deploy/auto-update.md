# Automatic Updates

Komodo can check for newer Docker image digests and react when the image behind a tag changes.

Use this for images that intentionally move over time, such as `:latest` or another rolling tag.
Do not use it as the only update mechanism for pinned image versions in git-managed stacks.

## Update Modes

Both [Stack](./compose.md) and [Deployment](./containers.md) resources support two related modes:

- **Poll for Updates**: check for newer images with the same tag, show an update indicator in the
  UI, and send an alert if an Alerter is configured. This does not redeploy.
- **Auto Update**: perform the same check and automatically redeploy when a newer digest is found.
  This also sends an alert.

Use `Poll for Updates` when you want visibility before rollout. Use `Auto Update` when the tag is
intentionally rolling and redeploying on a new digest is acceptable.

This is registry-driven change detection. It answers "did the image behind this tag change?" It
does not answer "did the source repository change?" For that path, use git-backed
[Stacks](./compose.md), [Build](../build.md), [Repo](../repo.md), and
[Webhooks](../automate/webhooks.md).

:::note
Auto update depends on a moving image tag. For pinned versions in git-sourced stacks, use a
version-management workflow such as [Renovate](https://github.com/renovatebot/renovate) instead of
digest polling.
:::

## Global Auto Update Procedure

New installs include a `Global Auto Update` Procedure scheduled daily. It loops through all
resources with `poll_for_updates` or `auto_update` enabled and checks registries for newer digests.

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

This is only a default automation entry point. You can move the same execution into another
[Procedure](../automate/procedures.md) if update timing needs to line up with backup windows,
maintenance windows, or another operational sequence.

## Choose The Right Update Path

- Use Komodo auto update when the image tag is meant to float.
- Use git-based change management when the deployed version should be reviewed before rollout.
- Use [Webhooks](../automate/webhooks.md) or git-backed [Stacks](./compose.md) when the rollout
  should follow a git event rather than a registry digest change.
- Use [Build](../build.md) plus `redeploy_on_build` on [Containers](./containers.md) when Komodo
  owns the image build and the Deployment should follow that Build.

## Related Pages

- [Docker Compose](./compose.md)
- [Containers](./containers.md)
- [Build](../build.md)
- [Repo](../repo.md)
- [Procedures and Actions](../automate/procedures.md)
- [Back Up And Restore](../setup/backup.md)
