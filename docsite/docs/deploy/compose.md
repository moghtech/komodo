---
sidebar_label: Compose Stacks
---

# Docker Compose

Komodo deploys Compose projects through the `Stack` resource.

In Compose mode, a Stack runs [`docker compose`](https://docs.docker.com/reference/cli/docker/compose/)
on the attached [Server](../server.md) through [Periphery](../periphery.md). In Swarm mode, the
same resource runs `docker stack deploy` against a configured [Swarm](../swarm.md).

Use a Stack when the workload is naturally expressed as one or more Compose files. Use
[Containers](./containers.md) when the workload is a single container and you do not want Compose
as the deployment model.

## Choose The Stack Mode

A Stack has two execution modes:

- **Compose mode**: set `server`. Komodo runs `docker compose` on that host.
- **Swarm mode**: set `swarm`. If both `server` and `swarm` are set, `swarm` wins and the Stack is
  deployed as a Swarm stack.

## Choose Where The Compose Files Come From

Stacks support three source paths:

- **UI-managed files**: store the Compose file contents in the Stack resource itself.
- **Files on the host**: point at Compose files that already exist under the Periphery-visible root
  directory.
- **Git-backed files**: clone a repository onto the host and deploy from that checkout.

Those choices affect how Komodo decides what changed, where files live, and which host or provider
credentials are needed.

See [Repo](../repo.md), [Host Model](../host-model.md), and
[Configure Git-Backed Workflows](../how-to/git-backed-workflows.md) for the host and git model
behind those choices.

## What Komodo Actually Does

In Compose mode, Stack deployment is built around a working directory and one or more Compose files.
Komodo can:

- write UI-managed file contents into that directory
- clone or update a repository there
- write a generated env file such as `.env`
- add extra `--env-file` arguments
- optionally run pre-deploy or post-deploy commands
- run `docker compose up -d` with the configured extra arguments

The Stack is the host-side deployment context for that Compose project.

## Example

```toml
[[stack]]
name = "my-stack"
[stack.config]
server = "server-prod"
run_directory = "/opt/stacks/my-stack"
file_paths = ["compose.yaml"]
git_account = "my-user"
repo = "myorg/stacks"
environment = """
DB_HOST = db.example.com
LOG_LEVEL = info
"""
```

## Core Fields

| Field | Meaning | Default |
| --- | --- | --- |
| `server` | Server to deploy on in Compose mode. | — |
| `swarm` | Swarm to deploy on in Swarm mode. Overrides `server` when both are set. | — |
| `run_directory` | Working directory on the target host before the deploy command runs. | `""` |
| `file_paths` | Compose files relative to `run_directory`. Defaults to `compose.yaml`. | `[]` |
| `project_name` | Compose project name. If empty, Komodo uses the Stack name. | Stack name |
| `environment` | Environment variables written to the generated env file. | `""` |
| `env_file_path` | Relative path of the generated env file. | `.env` |
| `additional_env_files` | Additional env files passed with `--env-file`. | `[]` |
| `config_files` | Additional tracked files associated with the Stack. | `[]` |
| `links` | Quick links shown in the resource header. | `[]` |
| `send_alerts` | Send Stack state alerts. | `true` |

## File Source And Git Fields

| Field | Meaning | Default |
| --- | --- | --- |
| `files_on_host` | Use Compose files that already exist on the host. | `false` |
| `file_contents` | UI-managed Compose file contents written by Komodo. | `""` |
| `linked_repo` | Existing Komodo `Repo` resource used as the source checkout. | `""` |
| `git_provider` | Git provider domain for direct repository cloning. | `github.com` |
| `git_https` | Use HTTPS instead of HTTP when cloning. | `true` |
| `git_account` | Provider account used for private repository access. | `""` |
| `repo` | Repository in `owner/repo` form. | `""` |
| `branch` | Branch to clone. | `main` |
| `commit` | Optional pinned commit hash. | `""` |
| `clone_path` | Optional explicit checkout path. | `""` |
| `reclone` | Delete and reclone instead of `git pull` after the initial clone. | `false` |

The git-backed paths are:

- use `linked_repo` when another Komodo [Repo](../repo.md) resource already owns the checkout
- use `repo` and provider fields when the Stack should manage its own checkout directly

That distinction matters because the checkout location, credentials, and webhook behavior follow
the thing that owns the repository state.

## Deployment Behavior Fields

| Field | Meaning | Default |
| --- | --- | --- |
| `auto_pull` | Run `docker compose pull` before redeploying. Not used in Swarm mode. | `true` |
| `run_build` | Run `docker compose build` before deploy. Not used in Swarm mode. | `false` |
| `build_extra_args` | Extra arguments passed after `docker compose build`. | `[]` |
| `destroy_before_deploy` | Run `docker compose down` before `up`. | `false` |
| `extra_args` | Extra arguments passed to `docker compose up -d` or `docker stack deploy`. | `[]` |
| `ignore_services` | Services ignored when evaluating Stack health. | `[]` |
| `pre_deploy` | Command run before deployment. | none |
| `post_deploy` | Command run after deployment. | none |

These fields control rollout behavior, local build behavior, and service-health evaluation.

## Secrets And Wrapper Commands

Some setups need a wrapper around the Compose command so another tool can inject secrets or prepare
runtime state first.

Use these fields for that:

- `compose_cmd_wrapper`
- `compose_cmd_wrapper_include`
- `skip_secret_interp`

`compose_cmd_wrapper` wraps the generated Compose command and uses `[[COMPOSE_COMMAND]]` as the
placeholder for the real command. This is the field used for tools such as 1Password CLI or SOPS
when the secret-loading step should happen at deploy time rather than by committing plain values to
the host.

For the broader secret model, see [Use Docker Secrets And `_FILE` Variables](../how-to/docker-secrets-files.md),
[Use Encrypted Secrets](../how-to/encrypted-secrets-workflows.md), and
[Variables and Secrets](../configuration/variables.md).

## Updates And Webhooks

Stacks support both registry-driven and git-driven change detection.

Registry-driven:

- `poll_for_updates`
- `auto_update`
- `auto_update_all_services`

Git-driven:

- `webhook_enabled`
- `webhook_secret`
- `webhook_force_deploy`

Use update polling when the deployed images move behind a tag. Use webhooks when the repository
should trigger the deployment. The two are related, but they answer different questions:

- "Did the image digest change?"
- "Did the source repository change?"

See [Automatic Updates](./auto-update.md), [Webhooks](../automate/webhooks.md), and
[Repo](../repo.md).

## Import An Existing Compose Project

To import an existing Compose project, create a Stack that points at the same files and host
context already used by that project.

If the existing Compose project name does not match the Stack name, set `project_name` explicitly.
Komodo uses that name when matching the existing project on the host.

Run [`docker compose ls`](https://docs.docker.com/reference/cli/docker/compose/ls/) on the host if
you need to confirm the current project names.

## When A Stack Fits Better Than A Deployment

Choose a Stack when:

- there are multiple services
- the app already has Compose files
- you need Compose-native concepts such as multi-file layering
- the workload should stay close to normal Docker Compose workflows

Choose [Containers](./containers.md) when the workload is one container and Compose would only add
ceremony.

## Related Pages

- [Containers](./containers.md)
- [Swarm](../swarm.md)
- [Repo](../repo.md)
- [Host Model](../host-model.md)
- [Configure Git-Backed Workflows](../how-to/git-backed-workflows.md)
- [Automatic Updates](./auto-update.md)
