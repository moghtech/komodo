# Repo

A Repo is the resource Komodo uses when the checkout itself is the thing you want to manage. It
clones a repository onto a target [Server](./server.md) or [Builder](./build.md#builders), keeps
that checkout on the host, and can run commands after clone or pull.

Use Repo when the git working tree is part of the operational model: host-side scripts, generated
files, local build steps, or a shared checkout that other resources should reuse.

## What A Repo Does

A Repo tracks:

- which repository to clone
- which branch or commit to use
- which Server or Builder owns the checkout
- which commands should run after clone or pull

Komodo can clone, pull, or build a Repo resource. Those operations happen on the attached host
through [Periphery](./periphery.md), not in Core.

## Where A Repo Runs

Repo follows the same host model as other git-backed resources. The checkout lives on the target
machine in the environment visible to Periphery and the container runtime. That means path,
filesystem, and mount questions still need to be reasoned about from the host side. See
[Host Model](./host-model.md).

If a Repo is attached to a Server, clone and pull operations happen on that Server. If it is
attached to a Builder, build operations happen on that Builder.

## Git Access

Repo uses the same provider model as other git-backed resources. `git_provider` chooses the git
host, such as `github.com`, `git.example.com`, Forgejo, or Gitea. `git_account` selects the
configured account for private repository access. `repo`, `branch`, and `commit` choose what to
clone.

Create the provider account in the UI under `Settings > Providers`. See
[Providers](./configuration/providers.md).

Private repo access still follows the execution boundary: the token has to be available where the
clone or pull happens. If the Repo runs on a Periphery-managed host, the provider account must be
available to that host. If it runs on a Builder, the Builder-side environment has to be able to
use it.

That same rule applies to self-hosted git. Forgejo and Gitea are not special resource types in
Komodo. They are just git providers with a different domain and token. The one special case is
bootstrap: if Komodo is deploying the git service itself, bring that service up first before other
resources depend on it. See
[Bootstrap A Self-Hosted Git Provider](./how-to/bootstrap-self-hosted-git-provider.md).

## Commands And Environment

Repo can run commands after clone and after pull through `on_clone` and `on_pull`. Those commands
run relative to the checked-out repo path on the target machine. Repo also supports a
resource-specific environment file written into the checkout before those commands run. For shared
values and secrets, see [Variables and Secrets](./configuration/variables.md).

## Webhooks And Automation

Repo resources can be triggered by incoming webhooks from a git provider. The supported executions
are `clone`, `pull`, and `build`. That makes Repo useful for git-driven automation on a host. See
[Webhooks](./automate/webhooks.md) for the listener model and branch filtering.

## Repo And Other Resources

Repo overlaps with other git-backed resources, but the role is different:

- [Docker Compose](./deploy/compose.md) uses git to source compose files for a Stack.
- [Build](./build.md) uses git to source a Dockerfile and build context.
- [Resource Sync](./automate/sync-resources.md) uses git to source declarative resource files.

Repo is the right choice when you want the checkout itself to be a managed resource with clone,
pull, and host-side command execution, rather than just an input to a Stack or Build.

For the broader decision between Repo, git-backed Stacks, Builds, and Resource Sync, see
[Configure Git-Backed Workflows](./how-to/git-backed-workflows.md).

## Related Pages

- [What Is Komodo](./intro.md)
- [Server](./server.md)
- [Periphery](./periphery.md)
- [Host Model](./host-model.md)
- [Providers](./configuration/providers.md)
- [Configure Git-Backed Workflows](./how-to/git-backed-workflows.md)
- [Bootstrap A Self-Hosted Git Provider](./how-to/bootstrap-self-hosted-git-provider.md)
- [Webhooks](./automate/webhooks.md)
- [Build](./build.md)
- [Resources](./resources.md#repo)
