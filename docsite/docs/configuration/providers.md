# Providers

Providers are the stored git and registry credentials Komodo uses on behalf of resources such as
[Build](../build.md), [Repo](../repo.md), [Stack](../deploy/compose.md), and
[Resource Sync](../automate/sync-resources.md).

For task-oriented guidance on choosing between those git-backed resources, see
[Configure Git-Backed Workflows](../how-to/git-backed-workflows.md).

## Usage

When a resource references a private repository or registry, select the matching provider and
account in that resource's configuration. Komodo then uses the stored token for clone, pull, or
push.

The important constraint is location: the account has to be available where the work runs. For
git-backed resources on a Periphery-managed host, the host-side execution path has to be able to
use that account. For builds on a Builder, the Builder-side execution path has to be able to use
it. See [Periphery](../periphery.md), [Server](../server.md), and
[Configure Git-Backed Workflows](../how-to/git-backed-workflows.md).

## Managing Providers In The UI

Manage providers in the Komodo UI under `Settings > Providers`. From there you can:

- add new git provider or registry accounts
- view accounts loaded from the UI or config files
- edit or delete database-managed accounts

## Configuring Via Config Files

Providers can also be defined in config files:

- **Core config** (`core.config.toml`): accounts are defined centrally and appear in the UI for
  selection across Komodo resources. See [Mounted Config Files](../how-to/mounted-config-files.mdx).
- **Periphery config** (`periphery.config.toml`): accounts are only available to resources running
  on that specific server. See [Connect More Servers](../setup/connect-servers.mdx).

Accounts loaded from config files appear in the UI, but their tokens cannot be read back through
the API or UI.

## Git Providers

Komodo supports cloning over HTTP or HTTPS from any provider that accepts one of these forms:

```shell
git clone https://<Username>:<Token>@<domain>/<Owner>/<Repo>
```

```shell
git clone https://<Token>@<domain>/<Owner>/<Repo>
```

This includes GitHub, GitLab,
[Bitbucket](https://github.com/moghtech/komodo/issues/387#issuecomment-3240726344), Forgejo,
Gitea, and many other git providers.

From Komodo's point of view, GitHub, Forgejo, and Gitea all fit the same provider model: a
`domain`, an `https` setting, and one or more `{ username, token }` accounts. The main difference
is operational, not conceptual:

- public providers such as GitHub are already reachable and can be added directly
- self-hosted providers need local DNS, certificates, routing, and bootstrap planning to exist
  before other resources depend on them

If the self-hosted git service is itself being deployed by Komodo, read
[Bootstrap A Self-Hosted Git Provider](../how-to/bootstrap-self-hosted-git-provider.md) before
making that service depend on its own provider entry.

### Git Provider Fields

- `domain`
  - default: `github.com`
  - hostname of the git provider. Do not include `http://` or `https://`.
- `https`
  - default: `true`
  - clone over HTTPS. Set to `false` for HTTP in local or internal setups.
- `accounts`
  - default: `[]`
  - list of `{ username, token }` pairs. Each account provides access to the repos visible to that
    user.

### Git Provider Configuration

```toml
# in core.config.toml or periphery.config.toml

[[git_provider]]
domain = "github.com"
accounts = [
  { username = "my-user", token = "ghp_xxxxxxxxxxxx" },
]

[[git_provider]]
domain = "git.example.com" # self-hosted Gitea, GitLab, etc.
accounts = [
  { username = "my-user", token = "access_token" },
]

[[git_provider]]
domain = "localhost:3000"
https = false # clone over http://
accounts = [
  { username = "my-user", token = "access_token" },
]
```

## Docker Registries

Komodo supports pushing and pulling images from Docker-compatible registries, including Docker Hub,
GitHub Container Registry, and self-hosted registries.

### Docker Registry Fields

- `domain`
  - default: `docker.io`
  - hostname of the registry. It can include `http://` for insecure registries when the host
    runtime is configured for them.
- `accounts`
  - default: `[]`
  - list of `{ username, token }` pairs used for registry authentication.
- `organizations`
  - default: `[]`
  - optional organization or namespace names. Builds can publish under one of these instead of the
    account's namespace.

### Docker Registry Configuration

```toml
# in core.config.toml or periphery.config.toml

[[docker_registry]]
domain = "docker.io"
accounts = [
  { username = "my-user", token = "dckr_pat_xxxxxxxxxxxx" },
]
organizations = ["MyOrg"]

[[docker_registry]]
domain = "ghcr.io"
accounts = [
  { username = "my-user", token = "ghp_xxxxxxxxxxxx" },
]

[[docker_registry]]
domain = "registry.example.com" # self-hosted registry
accounts = [
  { username = "my-user", token = "access_token" },
]
organizations = ["MyTeam"]
```

:::note

GitHub tokens used for pushing images must include the `write:packages` permission. See the
[GitHub package registry docs](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry#authenticating-with-a-personal-access-token-classic).

:::

### Docker Registry Usage

When configuring a [Build](../build.md), select the registry domain and account to push images to.
If organizations are defined, you can choose to publish under one of those namespaces.

When a Build is connected to a [Deployment](../deploy/containers.md), the Deployment inherits the
registry configuration by default. If that account is not available to the Deployment's server,
choose another account in the Deployment config.
