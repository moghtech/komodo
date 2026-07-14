# Resources

Komodo models managed objects as resources. `Server`, `Deployment`, and `Stack` are resource types.
For the execution model behind them, see [What Is Komodo](./intro.md).

All resources have a type-local `name` and `id`. Resources can also be grouped with `tags`.

:::note

Some resources need access to git repositories or container registries. Resources that work with
private repos or registries can use configured
[provider credentials](./configuration/providers.md).

:::

## [Server](./server.md)

- Configure the connection to periphery agents.
- Set alerting thresholds.
- Used by **Deployments**, **Stacks**, **Repos**, and **Builders**.

See [Server](./server.md) for the concept page and
[Connection Model](./connection-model.md) and [Connect More Servers](./setup/connect-servers.mdx)
for identity and installation.

## [Swarm](./swarm.md)

- Configure the manager nodes to control the Swarm through.
- Manage swarm nodes, stacks, services, tasks, configs, and secrets.
- Can be attached to by **Deployments** and **Stacks**.

## [Deployment](./deploy/containers.md)

- Deploy one container on an attached Server, or one service in Swarm mode.
- Use this when the workload maps cleanly to one container rather than a Compose project.
- Supports direct-image deploys, [Build](./build.md)-based deploys, lifecycle actions, and
  registry-driven updates through [Automatic Updates](./deploy/auto-update.md).

## [Stack](./deploy/compose.md)

- Deploy a Compose project on a Server, or a stack in Swarm mode.
- Source files from the UI, files already on the host, a direct git repository, or a linked
  [Repo](./repo.md).
- Supports multi-file Compose deploys, generated env files, wrapper commands, and git or
  registry-driven updates through [Webhooks](./automate/webhooks.md) and
  [Automatic Updates](./deploy/auto-update.md).

## [Repo](./repo.md)

- Clone a git repository onto a Server or Builder and keep that checkout managed in Komodo.
- Run host-side commands after clone or pull.
- Support webhook-driven clone, pull, and build execution.
- Use configured provider credentials for private git access.

See [Repo](./repo.md), [Providers](./configuration/providers.md), and
[Webhooks](./automate/webhooks.md).

## [Build](./build.md)

- Build container images on an attached Builder and push them to one or more registries.
- Source the Dockerfile and build context from the UI, files on the host, a direct git repo, or a
  linked [Repo](./repo.md).
- Controls image naming, tagging, build args, secret args, and Buildx usage.
- Feeds [Deployment](./deploy/containers.md) resources directly and can also support Compose-based
  workflows that build on the host.

## [Builder](./build.md#builders)

- Either points to a connected Server, a direct Periphery URL, or holds configuration to launch a
  single-use AWS instance for builds.
- Can be attached to [Build](./build.md) and [Repo](./repo.md) resources.

## [Procedure](./automate/procedures.md#procedures)

- Compose built-in executions such as `RunBuild`, `DeployStack`, or `PullRepo` into staged
  automation.
- Run manually, on a [schedule](./automate/schedules.md), or through a
  [webhook](./automate/webhooks.md).

## [Action](./automate/procedures.md#actions)

- Write TypeScript scripts that call the Komodo API from inside Komodo.
- Use a pre-initialized Komodo client in the script, with no separate API-key setup.
- Type-aware in the UI editor.
- The TypeScript client is also [published on NPM](https://www.npmjs.com/package/komodo_client).

See [Write And Debug Actions](./how-to/write-and-debug-actions.md) for the scripting and
debugging path.

## [Resource Sync](./automate/sync-resources.md)

- Orchestrate resource configuration declaratively from `toml` files, often in git.
- Can deploy **Deployments** and **Stacks** if changes are suggested.
- Specify deploy ordering with `after` across resources.

See [Configure Git-Backed Workflows](./how-to/git-backed-workflows.md) for where Resource Sync fits
relative to [Repo](./repo.md), [Build](./build.md), and git-backed Stacks.

## [Alerter](./alerter.md)

- Route alerts to external endpoints such as Slack, Discord, ntfy, Pushover, or a custom HTTP
  endpoint.
- Filter by alert type, resource include list, resource exclude list, and maintenance windows.
- Used by alert-producing features such as [Automatic Updates](./deploy/auto-update.md),
  [Schedules](./automate/schedules.md), and [Swarm](./swarm.md).
