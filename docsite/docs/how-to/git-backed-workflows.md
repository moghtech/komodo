# Configure Git-Backed Workflows

Komodo uses git in several different ways. The setup is easier if you decide first what the git
repository is meant to provide:

- a checkout to run commands from
- a Compose file or stack definition
- a Docker build context
- declarative resource definitions

Those are different workflows even though they all need provider access and a host that can reach
the repository.

## Choose The Git-Backed Resource

Use [Repo](../repo.md) when the checkout itself is the managed thing. Repo clones onto a
[Server](../server.md) or [Builder](../build.md#builders), can pull updates, and can run host-side
commands after clone or pull.

Use [Docker Compose](../deploy/compose.md) when the repository is the source of a Stack's compose
files.

Use [Build](../build.md) when the repository provides a Dockerfile and build context and Komodo
should produce an image from that source.

Use [Resource Sync](../automate/sync-resources.md) when the repository holds declarative Komodo
resource files that should be applied back into Core.

## Configure Provider Access

Configure git access in the Komodo UI under `Settings > Providers`. See
[Providers](../configuration/providers.md) for the provider model.

Providers can also be loaded from config files:

- [Core config](../how-to/mounted-config-files.mdx) makes accounts available globally.
- [Periphery config](../setup/connect-servers.mdx) makes accounts available only on that specific
  server.

This matters because git operations happen where the resource runs. If a private checkout happens on
Periphery, the token needs to be available in the environment visible to that Periphery agent.
See [Periphery](../periphery.md), [Server](../server.md), and [Host Model](../host-model.md) for
the execution boundary behind that rule.

GitHub, Forgejo, and Gitea all use the same basic model here:

- set the provider `domain`
- decide whether it uses HTTPS or HTTP
- add an account token that works on that provider

The provider does not become a different kind of resource just because it is self-hosted. The
special case is bootstrap, not normal day-two usage.

## Decide Where The Checkout Runs

Git-backed resources do not clone into Core's filesystem unless the resource actually runs there.
They clone where [Periphery](../periphery.md) or a [Builder](../build.md#builders) performs the
work.

That affects:

- filesystem paths
- repo visibility on the host
- compose relative paths
- build context paths
- which provider account is available

If the path and mount model is still unclear, read [Host Model](../host-model.md) and
[Use Containerized Periphery Without Path Surprises](./containerized-periphery-paths.md) before
building more git automation around it.

## Common Setup Pattern

One small git-backed setup looks like this:

1. configure the git provider account in `Settings > Providers`
2. verify the target host or Builder can reach the repository
3. deploy one git-backed resource
4. add webhooks or resource sync only after the basic checkout and execution path works

That keeps provider, filesystem, and execution problems separate.

## Private Self-Hosted Git

Self-hosted Git providers such as Forgejo or Gitea use the same provider model. Configure the
provider `domain`, decide whether it uses HTTPS or HTTP, and attach the correct account token.

If the self-hosted provider is itself being deployed by Komodo, read
[Bootstrap A Self-Hosted Git Provider](./bootstrap-self-hosted-git-provider.md). That is the one
common case where the normal git-backed pattern needs a bootstrap step first.

If the provider is already running and reachable, treat it like any other private git host. Check:

- can the host that runs the checkout reach it?
- does the certificate or CA chain validate there?
- does the account token have the repository access you expect?

## Webhooks And Automation

Git-backed resources become more useful once they are connected to incoming webhooks or scheduled
automation:

- [Webhooks](../automate/webhooks.md) for push-triggered clone, pull, build, and deploy operations
- [Procedures and Actions](../automate/procedures.md) for multi-step workflows
- [Write And Debug Actions](./write-and-debug-actions.md) when you need scripted API work around a
  git-backed flow

## Related Pages

- [Repo](../repo.md)
- [Providers](../configuration/providers.md)
- [Docker Compose](../deploy/compose.md)
- [Build](../build.md)
- [Resource Sync](../automate/sync-resources.md)
- [Bootstrap A Self-Hosted Git Provider](./bootstrap-self-hosted-git-provider.md)
