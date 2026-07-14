---
sidebar_label: Use Docker Secrets and _FILE
---

# Docker Secrets And `_FILE` Variables

Komodo supports the `${VARIABLE}_FILE` pattern for many secret settings. This lets Docker or
Compose mount a secret as a file, while Core or Periphery reads the value from that file instead of
from a plain environment variable.

This is useful when you want to keep passwords, tokens, or client secrets out of `compose.env`.

## How The Pattern Works

Instead of:

```env
KOMODO_JWT_SECRET=super-secret-value
```

use:

```env
KOMODO_JWT_SECRET_FILE=/run/secrets/komodo_jwt_secret
```

Komodo then reads the file contents and uses that value as the secret.

## Compose Example

With Docker Compose, define a secret and mount it into the Core service:

```yaml
services:
  core:
    secrets:
      - komodo_jwt_secret
    environment:
      KOMODO_JWT_SECRET_FILE: /run/secrets/komodo_jwt_secret

secrets:
  komodo_jwt_secret:
    file: ./secrets/komodo_jwt_secret
```

The same pattern works for other supported variables. The main change is the environment variable
name and the secret filename.

## Common Variables

Examples called out in the shipped `compose.env` include:

- `KOMODO_WEBHOOK_SECRET_FILE`
- `KOMODO_JWT_SECRET_FILE`
- `KOMODO_OIDC_CLIENT_ID_FILE`
- `KOMODO_OIDC_CLIENT_SECRET_FILE`
- `KOMODO_GITHUB_OAUTH_ID_FILE`
- `KOMODO_GITHUB_OAUTH_SECRET_FILE`
- `KOMODO_GOOGLE_OAUTH_ID_FILE`
- `KOMODO_GOOGLE_OAUTH_SECRET_FILE`
- `KOMODO_AWS_ACCESS_KEY_ID_FILE`
- `KOMODO_AWS_SECRET_ACCESS_KEY_FILE`

The config files also show `_FILE` support for database credentials, initial admin credentials, TLS
material, and other secret values. When in doubt, check the comments in
`core.config.toml`, `periphery.config.toml`, or `compose.env`.

## When To Use This

Use `_FILE` variables when:

- you are already using Docker or Compose secrets
- you want to keep secret values out of `compose.env`
- a mounted config file would be too broad for the change you need

Mounted config files are better when you want to move a larger block of configuration at once. See
[Mounted Config Files](./mounted-config-files.mdx).

## Related Pages

- [Install Komodo](../setup/install-komodo.mdx)
- [Mounted Config Files](./mounted-config-files.mdx)
- [Variables and Secrets](../configuration/variables.md)
- [Providers](../configuration/providers.md)
