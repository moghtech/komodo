# Build

Komodo builds Docker images by running `docker build` and pushing the result to a configured image registry.

## Dockerfile Sources

Komodo supports three ways of providing the Dockerfile and build context:

1. **Write in the UI** — define the Dockerfile contents directly in Komodo. Supports variable and secret interpolation.
2. **Files on host** — point to an existing Dockerfile and build context already present on the builder machine. Set `files_on_host = true` and use `build_path` / `dockerfile_path` to specify the paths.
3. **Git repo** — clone a repository containing the Dockerfile. This is the default mode. Configure `repo`, `branch`, and optionally `git_account` for private repos.

## Configuration

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

### Config fields

| Field | Description | Default |
|---|---|---|
| `builder` | The Builder resource to run the build on. | — |
| `version` | Current build version (`major.minor.patch`). | `0.0.0` |
| `auto_increment_version` | Auto-increment patch number on each build. | `true` |
| `image_name` | Alternate image name (uses build name if empty). | `""` |
| `image_tag` | Extra tag postfix, e.g. `aarch64` → `:1.2.3-aarch64`. | `""` |
| `include_latest_tag` | Push `:latest` / `:latest-{image_tag}` tags. | `true` |
| `include_version_tags` | Push semver tags (`:1.2.3`, `:1.2`, `:1`). | `true` |
| `include_commit_tag` | Push commit hash tag (`:a6v8h83`). | `true` |
| `linked_repo` | A Komodo Repo resource to source files from. | `""` |
| `git_provider` | Git provider domain. | `github.com` |
| `git_https` | Use HTTPS to clone (versus HTTP). | `true` |
| `git_account` | Git provider account for private repos. | `""` |
| `repo` | Repository in `owner/repo` format. | `""` |
| `branch` | Branch to clone. Webhooks only trigger on pushes to this branch. | `main` |
| `commit` | Pin to a specific commit hash. | `""` |
| `files_on_host` | Source Dockerfile and build context from files already on the builder. | `false` |
| `dockerfile` | UI-defined Dockerfile contents. Supports variable/secret interpolation. | `""` |
| `build_path` | Build context directory, relative to the repo root (or absolute path when `files_on_host`). | `.` |
| `dockerfile_path` | Dockerfile path, relative to the build directory. | `Dockerfile` |
| `image_registry` | Registry to push images to (domain + account + optional organization). | `[]` |
| `build_args` | Build arguments in `KEY=value` format. Visible in `docker history`. | `""` |
| `secret_args` | Build secrets in `KEY=value` format. Access via `RUN --mount=type=secret,id=KEY`. Not visible in image history. | `""` |
| `skip_secret_interp` | Skip secret interpolation in build_args. | `false` |
| `extra_args` | Additional flags passed to `docker build`. | `[]` |
| `use_buildx` | Use `docker buildx build` instead of `docker build`. | `false` |
| `pre_build` | Command to run after cloning but before `docker build`. | — |
| `labels` | Docker labels in `key=value` format. | `""` |
| `webhook_enabled` | Whether incoming webhooks trigger builds. | `true` |
| `webhook_secret` | Alternate webhook secret (uses default from config if empty). | `""` |
| `links` | Quick links displayed in the resource header. | `[]` |

## Image Versioning and Tagging

Komodo uses a `major.minor.patch` versioning scheme. By default, each build auto-increments the patch number. You can control exactly which tags are pushed using the following options:

### Tag types

| Option | Tags produced | Example |
|---|---|---|
| `include_version_tags` | Full semver, minor, and major | `:1.2.3`, `:1.2`, `:1` |
| `include_latest_tag` | Latest tag | `:latest` |
| `include_commit_tag` | Short commit hash | `:a6v8h83` |

All three are enabled by default. Disable any combination to control which tags are pushed.

### Image tag postfix

The `image_tag` field appends a postfix to all generated tags. This is useful for multi-platform or variant builds:

| `image_tag` | Version tag | Latest tag | Commit tag |
|---|---|---|---|
| _(empty)_ | `:1.2.3` | `:latest` | `:a6v8h83` |
| `aarch64` | `:1.2.3-aarch64` | `:latest-aarch64` | `:a6v8h83-aarch64` |

When `image_tag` is set, an additional pure tag is also pushed: `:aarch64`.

### Custom image name

By default, the build's name is used as the image name. Set `image_name` to override this, e.g. if the build name doesn't match the desired image name on the registry.

### Manual versioning

Set `auto_increment_version = false` to manage the `version` field yourself. The major and minor versions are always set manually — only the patch auto-increments.

## Image Registry

Komodo supports pushing to any Docker-compatible registry. Configure accounts in [Providers](configuration/providers.md).

A build can push to **multiple registries** simultaneously. The `image_registry` field accepts a list — each entry specifies a domain, account, and optional organization:

```toml
[build.config]
image_registry = [
  { domain = "ghcr.io", account = "my-user", organization = "my-org" },
  { domain = "docker.io", account = "my-user" },
]
```

The first registry in the list is used by default when a Deployment is connected to the Build.

:::note
GitHub access tokens must have the `write:packages` permission to push to GHCR.
See the [GitHub docs](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry#authenticating-with-a-personal-access-token-classic).
:::

When a Build is connected to a Deployment, the Deployment inherits the Build's registry credentials by default. If the builder's account isn't available to the Deployment's server, select a different account in the Deployment config.


## Multi-platform builds (Buildx)

To build for multiple platforms (e.g. ARM + x86), set up Docker Buildx on the builder:

```sh
docker buildx create --name builder --use --bootstrap
docker buildx install   # makes buildx the default for `docker build`
```

Then pass the target platforms in the Build's **Extra Args**:

```
--platform linux/amd64,linux/arm64
```

## Builders

A `Builder` resource defines **where** builds run. Any Server connected to Komodo can be used as a builder, but building on production servers is not recommended.

### Server builder

Point the builder at an existing Server running the Periphery agent.

### AWS EC2 builder

Komodo can launch a temporary EC2 instance for each build and shut it down when finished.

```toml
[[builder]]
name = "builder-01"
[builder.config]
type = "Aws"
params.region = "us-east-2"
params.instance_type = "c5.2xlarge"
params.ami_id = "ami-xxxxxxxxxxxxxxxxxx"
params.subnet_id = "subnet-xxxxxxxxxxxxxxxxxx"
params.key_pair_name = "ssh-key"
params.assign_public_ip = true ## Required for outbound internet access unless you have network gateway.
params.use_public_ip = true ## Setting 'false' uses the private IP (when Komodo Core is in same subnet).
params.security_group_ids = ["sg-xxxxxxxxxxxxxxxxxx"]
params.user_data = """
#!/bin/bash
curl -sSL \
   https://raw.githubusercontent.com/moghtech/komodo/main/scripts/setup-periphery.py | \
  HOME=/root python3 - --version=v2.X.X
"""
```

To create the AMI:

1. Launch an EC2 instance and install Docker:
   ```shell
   apt update && apt upgrade -y
   curl -fsSL https://get.docker.com | sh
   systemctl enable docker.service containerd.service
   ```
2. Create an AMI from the instance in the AWS console.
3. Create a security group and ensure it allows inbound access on port **8120** from Komodo Core.

The instance `user_data` will install the Periphery agent as the instance starts up,
and Komodo Core will then connect and build the image.



