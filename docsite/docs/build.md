# Build

Komodo builds container images through Docker and pushes them to one or more configured image
registries.

By default, Komodo runs [Docker Build](https://docs.docker.com/build/concepts/overview/) with
`docker build`. When `use_buildx` is enabled, it runs
[`docker buildx build`](https://docs.docker.com/reference/cli/docker/buildx/build/).

Use a `Build` when Komodo should own the image build step. Use [Repo](./repo.md) when the goal is
to manage a checkout and run host-side commands without the image-build workflow.

## Builder And Source Model

Every Build has two location choices:

- **where the build runs**: the attached [`Builder`](./resources.md#builder)
- **where the source comes from**: UI-managed Dockerfile, files on the host, git, or a linked Repo

A Build combines:

- a machine that can run Docker builds
- a source tree or Dockerfile source
- one or more target registries
- a tag/versioning strategy

## Choose The Source Path

Builds support four source paths:

- **UI-managed Dockerfile**: store Dockerfile contents directly in Komodo.
- **Files on host**: point at an existing Dockerfile and build context already visible on the
  builder machine.
- **Direct git config**: configure `repo`, `branch`, and related git fields on the Build itself.
- **Linked Repo**: use an existing [Repo](./repo.md) resource as the source checkout.

## Example

```toml
[[build]]
name = "my-app"
[build.config]
builder = "builder-01"
repo = "myorg/my-app"
branch = "main"
git_account = "my-user"
image_registry = [
  { domain = "ghcr.io", account = "my-user", organization = "my-org" }
]
```

## Core Fields

| Field | Meaning | Default |
| --- | --- | --- |
| `builder` | Builder resource that runs the build. | — |
| `linked_repo` | Existing Komodo `Repo` resource used as the source checkout. | `""` |
| `repo` | Repository used as the build source. | `""` |
| `branch` | Branch to clone. | `main` |
| `commit` | Optional pinned commit hash. | `""` |
| `files_on_host` | Use files that already exist on the builder host. | `false` |
| `build_path` | Build context path. Defaults to the repository root. | `.` |
| `dockerfile_path` | Dockerfile path relative to the build context. | `Dockerfile` |
| `dockerfile` | UI-managed Dockerfile contents. | `""` |
| `links` | Quick links shown in the resource header. | `[]` |

## Git And Provider Fields

| Field | Meaning | Default |
| --- | --- | --- |
| `git_provider` | Git provider domain. | `github.com` |
| `git_https` | Use HTTPS instead of HTTP when cloning. | `true` |
| `git_account` | Provider account for private repository access. | `""` |
| `webhook_enabled` | Whether git webhooks trigger builds. | `true` |
| `webhook_secret` | Alternate webhook secret. | `""` |

If the Build clones a private repository directly, the token must be available where the build
runs. If the Build uses a linked Repo, the Repo owns the checkout and its provider configuration
matters instead.

See [Providers](./configuration/providers.md), [Repo](./repo.md), and
[Configure Git-Backed Workflows](./how-to/git-backed-workflows.md).

## Image Naming And Tagging

Builds produce image names from:

- the Build name, unless `image_name` overrides it
- the configured `image_registry` entries
- versioning and tag settings

By default, Komodo can push:

- semver tags such as `:1.2.3`, `:1.2`, and `:1`
- `:latest`
- a short commit-hash tag

`image_tag` appends a suffix such as `-aarch64` to those generated tags. That is useful when
multiple builds publish variants of the same image name.

## Versioning Fields

| Field | Meaning | Default |
| --- | --- | --- |
| `version` | Current build version. | `0.0.0` |
| `auto_increment_version` | Auto-increment the patch version on each build. | `true` |
| `image_name` | Override the pushed image name. | `""` |
| `image_tag` | Extra suffix appended to generated tags. | `""` |
| `include_latest_tag` | Push latest tags. | `true` |
| `include_version_tags` | Push semver tags. | `true` |
| `include_commit_tag` | Push short commit-hash tags. | `true` |

Use `image_name` or `image_tag` when several Builds should publish related image variants under one
registry namespace.

## Registry Configuration

`image_registry` is a list, not a single destination. A Build can push to multiple registries in
one run.

Each entry defines:

- registry domain
- account
- optional organization

The first registry in the list becomes the default registry context for attached
[Containers](./deploy/containers.md).

:::note
GitHub access tokens need `write:packages` to push to GHCR. See the
[GitHub container registry docs](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry#authenticating-with-a-personal-access-token-classic).
:::

## Build Behavior

| Field | Meaning | Default |
| --- | --- | --- |
| `build_args` | Docker build arguments. Visible in image history. | `""` |
| `secret_args` | Build secrets used through Docker secret mounts. | `""` |
| `skip_secret_interp` | Skip secret interpolation in `build_args`. | `false` |
| `extra_args` | Extra Docker build or Buildx arguments. | `[]` |
| `use_buildx` | Use `docker buildx build` instead of `docker build`. | `false` |
| `pre_build` | Command run after clone and before the build. | none |
| `labels` | Docker labels attached during build. | `""` |

`build_args` and `secret_args` are not interchangeable:

- use `build_args` for non-secret build-time values
- use `secret_args` for values that should not end up visible in image history

## Multi-Platform Builds

Use `use_buildx` when the builder host is configured for Buildx and the image should target more
than one platform.

For example, after setting up Buildx on the builder, pass the target platforms in `extra_args`:

```text
--platform linux/amd64,linux/arm64
```

That is a build-host concern. The builder machine must already support the requested Buildx flow.

## Builders

A `Builder` resource defines where builds run.

### Server Builder

A server builder points at an existing connected [Server](./server.md). Use it when the build
should run on a machine Core already manages.

### URL Builder

A URL builder points at a Periphery endpoint by address instead of by an existing Server record.
This is the direct remote-builder path.

The key fields are:

- `address`: the Periphery address
- `periphery_public_key`: expected public key for that Periphery
- `insecure_tls`: whether to skip TLS certificate validation
- `passkey`: deprecated older authentication field

Use this when the build runner should be addressed directly as a Periphery endpoint rather than
through a connected [Server](./server.md). The same trust and network model still applies. See
[Connection Model](./connection-model.md).

### AWS EC2 Builder

Komodo can also launch a temporary EC2 instance for each build and shut it down afterward. This is
useful when builds need isolation or more resources than a standing server should provide.

The builder configuration is what describes the instance to launch. The AMI used there must already
have Docker available, and the EC2 instance must still be able to run Periphery and reach the
target registries.

## Related Pages

- [Containers](./deploy/containers.md)
- [Repo](./repo.md)
- [Providers](./configuration/providers.md)
- [Connection Model](./connection-model.md)
- [Configure Git-Backed Workflows](./how-to/git-backed-workflows.md)
- [Write And Debug Actions](./how-to/write-and-debug-actions.md)
