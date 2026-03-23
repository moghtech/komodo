# Providers

Providers allow Komodo to authenticate against external git providers and docker registries on behalf of your resources. Once configured, provider accounts are available to attach to Builds, Repos, Stacks, and Resource Syncs.

### Usage

When configuring a Build, Repo, Stack, or Resource Sync that references a private repository, select the matching git provider and account in the resource's configuration. Komodo will use the token to authenticate the clone.

## Managing Providers in the UI

The easiest way to set up providers is through the Komodo UI. Navigate to **Settings > Providers** to manage your git and registry accounts. From this page you can:

- **Add** new git provider or docker registry accounts (domain, username, and access token).
- **View** accounts that were added via the UI as well as those loaded from config files.
- **Edit or delete** database-managed accounts.

## Configuring via Config Files

As an alternative to the UI, providers can be defined in config files:

- **Core config** (`core.config.toml`): Accounts are available globally to all resources. See [Advanced Configuration](../setup/advanced.mdx#mount-a-config-file).
- **Periphery config** (`periphery.config.toml`): Accounts are only available to resources running on that specific server. See [Connect Servers](../setup/connect-servers.mdx).

Accounts from config files also appear in the UI under **Settings > Providers**, but their tokens cannot be read back through the API or UI.

## Git Providers

Komodo supports cloning repos over HTTP/S from any provider that supports 

```shell
git clone https://<Username>:<Token>@<domain>/<Owner>/<Repo>
```

or 

```shell
git clone https://<Token>@<domain>/<Owner>/<Repo>
```

This includes GitHub, GitLab,
[Bitbucket](https://github.com/moghtech/komodo/issues/387#issuecomment-3240726344),
Forgejo, Gitea, and many other git providers.

### Fields

| Field | Default | Description |
|-------|---------|-------------|
| `domain` | `github.com` | The hostname of the git provider. Do not include the protocol (`http://` or `https://`). |
| `https` | `true` | Whether to clone over HTTPS. Set to `false` for HTTP (e.g. local development). |
| `accounts` | `[]` | A list of `{ username, token }` pairs. Each account provides access to repos visible to that user. |

### Configuration

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

Komodo supports pushing and pulling images from any Docker-compatible registry, including Docker Hub, GitHub Container Registry (GHCR), and self-hosted registries.

### Fields

| Field | Default | Description |
|-------|---------|-------------|
| `domain` | `docker.io` | The hostname of the registry. Can include `http://` for insecure registries, but this requires [insecure registries](https://docs.docker.com/reference/cli/dockerd/#insecure-registries) enabled on your hosts. |
| `accounts` | `[]` | A list of `{ username, token }` pairs for registry authentication. |
| `organizations` | `[]` | Optional list of organization/namespace names. When attached to a Build, images are published under the organization's namespace instead of the account's. |

### Configuration

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
Your GitHub access token must have the `write:packages` permission in order to push images.
For example, see the [GitHub docs on access tokens](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry#authenticating-with-a-personal-access-token-classic).
:::

### Usage

When configuring a Build, select the registry domain and account to push images to. If organizations are defined, you can choose to publish under an organization namespace.

When a Build is connected to a Deployment, the Deployment inherits the registry configuration by default. If that account is not available to the Deployment's server, you can choose a different account in the Deployment config.
