# Automatic Updates

Starting from **v1.19.0**, new Komodo installs will automatically create the
**Global Auto Update** [Procedure](../resources/procedures#procedures), scheduled daily.
If you don't have it, this is the Toml:

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
You are also able to integrate `GlobalAutoUpdate` into other Procedures
to coordinate the timing with other processes, such as backup. There is
nothing special about this Procedure, it was just created for
guidance / convenience.
:::

### How does it work?

Both Stacks and Deployments allow you to configure **Poll for Updates** or **Auto Update**.
When [**GlobalAutoUpdate**](https://docs.rs/komodo_client/latest/komodo_client/api/execute/struct.GlobalAutoUpdate.html)
is run, Komodo will loop through all the resources with either of these options enabled,
and run [**PullStack**](https://docs.rs/komodo_client/latest/komodo_client/api/execute/struct.PullStack.html) / [**PullDeployment**](https://docs.rs/komodo_client/latest/komodo_client/api/execute/struct.PullDeployment.html)
in order to pick up any newer images **at the same tag**.
Note that in order to work, it requires use of a "Rolling" image tag, such as `:latest`.

:::info
If you use git sources Stacks and want to automatically update image tags, check out
[Renovate](https://github.com/renovatebot/renovate?tab=readme-ov-file#what-is-the-mend-renovate-cli)
:::

For resources with **Poll for Updates** enabled and an Alerter configured, it will
send an alert that a newer image is available, and display the update available indicator in the UI

For resource with **Auto Update** enabled, it will go ahead and Redeploy *just the services* with
newer images (by default). If an Alerter is configured, it will also send an alert that this occured.