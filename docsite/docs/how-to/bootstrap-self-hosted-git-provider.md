---
sidebar_label: Bootstrap Self-Hosted Git
---

# Bootstrap A Self-Hosted Git Provider

If you run Forgejo, Gitea, or another self-hosted git service in Komodo, that service cannot pull
its own repository until it is already up and serving git traffic. This is the bootstrap case for
`Files On Server` or another non-git-backed path first.

Treat the Git service as the thing that enables the rest of the git-backed workflow, not the first
thing that depends on it.

## Start With A Local Or Files-On-Server Stack

Bootstrap the Git provider using one of these paths:

- a Stack in `Files On Server` mode
- a local checkout or compose file copied to the host
- an external git provider that is already reachable

Get the Git service healthy once. At this stage, avoid making the service depend on its own git
endpoint.

## Bring The Git Service Up First

Once the provider stack is running:

1. confirm the web UI and git endpoint are reachable
2. create the repository or import the existing one
3. create the access token you want Komodo to use
4. add the provider account in [Providers](../configuration/providers.md)

Other [git-backed workflows](./git-backed-workflows.md) can use the new provider once the service is
up. The provider entry still uses the same fields in
[Providers](../configuration/providers.md): domain, transport choice, and one or more accounts.

## Move Other Resources To Git-Backed Mode

With the provider up, switch the resources that depend on it:

- [Repo](../repo.md) resources for host-side scripts or checkouts
- [Docker Compose](../deploy/compose.md) resources that pull compose files from git
- [Build](../build.md) resources that build images from git source
- [Resource Sync](../automate/sync-resources.md) for declarative resource definitions

That keeps the bootstrap dependency isolated to the provider itself.

## When To Move The Git Provider Itself

You can keep the Git provider stack in `Files On Server` mode indefinitely. That avoids a
self-reference during recovery or outage handling.

If you do want the provider stack itself to become git-backed later, make sure:

- the repository is already hosted and reachable
- the provider account is already configured in Komodo
- you understand what happens if that provider is down when Komodo needs to pull updates

Leaving the git provider on a host-managed or files-on-server path avoids a self-reference during
recovery and outage handling.

## Related Pages

- [Configure Git-Backed Workflows](./git-backed-workflows.md)
- [Repo](../repo.md)
- [Docker Compose](../deploy/compose.md)
- [Providers](../configuration/providers.md)
