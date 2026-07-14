---
slug: /how-to
---

# How-To

These guides cover operational tasks that sit outside the default install path or need a more
specific setup shape.

## Common Operational Tasks

- [Backup and Restore](../setup/backup.md): schedule database backups and restore from snapshots.
- [After First Server](../setup/after-first-server.md): validate the host
  model, deploy one small workload, and decide when to introduce git-backed workflows.
- [Add Another Server](./add-another-server.md): create an onboarding key, install Periphery on
  the new host, and verify the new Server.
- [Use A Reverse Proxy With Core And Periphery](./reverse-proxy-and-periphery-access.md): prefer
  private networks, or lock down the public path when Periphery must be reachable from outside.
- [Configure Git-Backed Workflows](./git-backed-workflows.md): choose between Repo, Stack, Build,
  and Resource Sync and configure provider access where the work runs.
- [Install And Use The Komodo CLI](./komodo-cli.md): install `km`, configure it, and run backup,
  deploy, and terminal commands.

## Auth, Config, And Secrets

- [Configure OIDC / OAuth2 Login](./oidc-oauth2.md): configure external login providers and
  redirect URIs.
- [Trust Custom CA Certificates](./custom-ca-certificates.md): trust internal certificate
  authorities in Core or Periphery.
- [Use Mounted Config Files](./mounted-config-files.mdx): move Core or Periphery configuration out
  of environment variables and into config files.
- [Use Docker Secrets And `_FILE` Variables](./docker-secrets-files.md): load secret values from
  files instead of storing them directly in `compose.env`.
- [Use Encrypted Secrets Workflows](./encrypted-secrets-workflows.md): combine Komodo with SOPS or
  another host-side secret path instead of storing plain secrets in git.

## Specialized And Environment-Specific Tasks

- [Use Containerized Periphery Without Path Surprises](./containerized-periphery-paths.md): keep
  root-directory paths, mounts, and relative paths understandable.
- [Connect Periphery Without An Onboarding Key](./connect-periphery-without-onboarding-key.md):
  reconnect an existing server or manage the keys explicitly.
- [Bootstrap A Self-Hosted Git Provider](./bootstrap-self-hosted-git-provider.md): bring up
  Forgejo, Gitea, or another internal git service before the rest of the environment depends on it.
- [Write And Debug Actions](./write-and-debug-actions.md): find the Action API surface and build up
  from small scripts.
- [Run Komodo Locally On macOS](./run-komodo-locally-on-macos.md): evaluate Komodo locally on
  macOS and handle the Linux-specific `/proc` mount in the sample stack.

## Related Pages

- [Setup](../setup/index.md)
- [Connection Model](../connection-model.md)
- [Install Komodo](../setup/install-komodo.mdx)
- [Connect More Servers](../setup/connect-servers.mdx)
