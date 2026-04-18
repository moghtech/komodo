---
sidebar_label: Install the Komodo CLI
---

# Install And Use The Komodo CLI

The Komodo CLI, `km`, is useful for backup and restore, resource updates, running executions, and
opening server or container terminals without going through the web UI.

Install `km`, configure one profile, and try a few common commands here. Use
[Komodo CLI](../ecosystem/cli.mdx) as the fuller command reference.

## Choose An Install Method

Use the packaged binary when possible:

- Linux: install with the provided `install-cli.py` script
- macOS: install with Homebrew

If you already run the Core container, `km` is bundled inside the Core image, so you can invoke it
with `docker exec` against the running Core container.

## First Configuration

`km` needs to know which Core to talk to and, for many commands, which API credentials to use. The
CLI supports:

- config files such as `komodo.cli.toml`
- environment variables
- command-line flags

It also supports multiple profiles, which is useful when you manage more than one Core.

To inspect the loaded CLI configuration, run:

```bash
km config
```

That prints the effective configuration after profile selection and overrides.

## Basic Commands

Start with these commands:

```bash
km --help
km config
km database backup
```

Then move to resource-specific commands such as:

```bash
km deploy stack my-stack
km run action my-action -y
km ssh my-server
```

The resource names and execution model behind those commands are covered in
[What Is Komodo](../intro.md), [Server](../server.md), and [Resources](../resources.md).

## Terminal Workflows

Server and container terminals are a common CLI workflow:

- `km ssh` / `km connect` for server terminals
- `km exec` for container shells
- `km attach` for container stdio

These commands complement the browser terminals described on [Terminals](../terminals.md).

## Troubleshooting

If a standalone CLI image or older package fails with shared-library errors, use one of the
supported install paths on [Komodo CLI](../ecosystem/cli.mdx) instead of relying on an arbitrary
container image. The Core container path uses the same packaged CLI that ships with Core itself.

If commands fail because the CLI cannot authenticate, inspect the active profile with `km config`
and verify that the expected host and API key settings are actually loaded.

## Related Pages

- [Komodo CLI](../ecosystem/cli.mdx)
- [Komodo API](../ecosystem/api.md)
- [Terminals](../terminals.md)
- [Backup and Restore](../setup/backup.md)
