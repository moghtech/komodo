# Build

Komodo builds Docker images by cloning a source repository, running `docker build`, and pushing the result to a configured image registry. Any repo containing a Dockerfile is buildable.

## Configuration

```toml
[[build]]
name = "my-app"
[build.config]
builder = "builder-01"
repo = "myorg/my-app"
branch = "main"
git_account = "my-user"
image_registry.type = "Standard"
image_registry.params.domain = "ghcr.io"
image_registry.params.account = "my-user"
```

### Config fields

| Field | Description | Default |
|---|---|---|
| `builder` | The Builder resource to run the build on. | — |
| `repo` | Repository in `owner/repo` format. | — |
| `branch` | Branch to clone. Webhooks only trigger on pushes to this branch. | `main` |
| `git_account` | Git provider account for private repos. | — |
| `git_provider` | Git provider domain. | `github.com` |
| `build_path` | Directory to build from, relative to the repo root. | `.` |
| `dockerfile_path` | Dockerfile path, relative to the build directory. | `Dockerfile` |
| `image_registry` | Registry to push images to (domain + account + optional organization). | — |
| `build_args` | Build arguments in `KEY=value` format. Visible in `docker history`. | `""` |
| `secret_args` | Build secrets in `KEY=value` format. Access via `RUN --mount=type=secret,id=KEY`. Not visible in image history. | `""` |
| `extra_args` | Additional flags passed to `docker build`. | `""` |
| `pre_build` | Command to run after cloning but before `docker build`. | — |
| `labels` | Docker labels in `key=value` format. | `""` |
| `links` | Quick links displayed in the resource header. | `[]` |

### Pre-build command

If you need to run a command before `docker build` (e.g. code generation, fetching dependencies), configure `pre_build`:

```toml
[build.config]
pre_build.path = "."          # working directory, relative to repo root
pre_build.command = "sh scripts/generate.sh"
```

## Image Versioning

Komodo uses a `major.minor.patch` versioning scheme. By default, each build auto-increments the patch number and pushes images with three tags:

| Tag | Example |
|---|---|
| Version | `my-app:1.2.3` |
| Commit hash | `my-app:h3c87c` |
| `latest` | `my-app:latest` |

You can turn off auto-increment and manage versions manually. An optional **version tag** postfixes a custom label, e.g. `my-app:1.2.3-dev`.

## Builders

A `Builder` resource defines **where** builds run. Any Server connected to Komodo can be used as a builder, but building on production servers is not recommended.

### Server builder

Point the builder at an existing Server running the Periphery agent.

### AWS EC2 builder

Komodo can launch a temporary EC2 instance for each build and shut it down when finished.

```toml
[[builder]]
name = "builder-01"
config.type = "Aws"
[builder.config.params]
region = "us-east-2"
ami_id = "ami-0e9bd154667944680"
subnet_id = "subnet-xxxxxxxxxxxxxxxxxx"
key_pair_name = "my-key"
assign_public_ip = true
use_public_ip = true
security_group_ids = ["sg-xxxxxxxxxxxxxxxxxx"]
```

To create the AMI:

1. Launch an EC2 instance and install Docker + Periphery:
   ```sh
   apt update && apt upgrade -y
   curl -fsSL https://get.docker.com | sh
   systemctl enable docker.service containerd.service
   curl -sSL https://raw.githubusercontent.com/moghtech/komodo/main/scripts/setup-periphery.py | HOME=/root python3
   systemctl enable periphery.service
   ```
2. Confirm Periphery is running: `systemctl status periphery.service`
3. Create an AMI from the instance in the AWS console.
4. Ensure the security group allows inbound access on port **8120** from Komodo Core.

:::note
AWS "user data" can run the install script automatically at launch for a hands-free setup.
:::

### Multi-platform builds (Buildx)

To build for multiple platforms (e.g. ARM + x86), set up Docker Buildx on the builder:

```sh
docker buildx create --name builder --use --bootstrap
docker buildx install   # makes buildx the default for `docker build`
```

Then pass the target platforms in the Build's **Extra Args**:

```
--platform linux/amd64,linux/arm64
```

## Image Registry

Komodo supports pushing to any Docker-compatible registry. Configure accounts in [Providers](configuration/providers.md).

:::note
GitHub access tokens must have the `write:packages` permission to push to GHCR.
See the [GitHub docs](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry#authenticating-with-a-personal-access-token-classic).
:::

When a Build is connected to a Deployment, the Deployment inherits the Build's registry credentials by default. If the builder's account isn't available to the Deployment's server, select a different account in the Deployment config.
