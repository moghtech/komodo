---
sidebar_label: Use Encrypted Secrets
---

# Use Encrypted Secrets Workflows

Komodo does not decrypt SOPS files or manage an external secret store for you. To keep secrets
encrypted at rest, combine Komodo with a host-side workflow
that produces plain secret files only at execution time.

The right choice depends on where the secret is consumed:

- environment variables in Compose-based installs
- mounted config files for Core or Periphery
- host-side deploy workflows that already use SOPS or another secret tool

## Native File-Based Path

If Komodo only needs the secret value at runtime, prefer file-based secret loading over putting the
secret directly into `compose.env`.

That is what [Docker Secrets And `_FILE` Variables](./docker-secrets-files.md) covers. It is a good
first step because the secret remains in a file and Komodo reads it through the supported
`${VARIABLE}_FILE` pattern.

## Mounted Config Files

For larger or more structured configuration, move the settings into mounted config files and keep
the sensitive material outside the main Compose environment file.

That fits:

- provider accounts
- OIDC settings
- TLS paths
- Periphery connection settings

See [Mounted Config Files](./mounted-config-files.mdx).

## SOPS And Similar Tools

If you already use SOPS, a common pattern is:

1. keep the encrypted file in git
2. decrypt it on the host at execution time
3. point Komodo or Docker at the resulting plain file
4. avoid committing the decrypted output

For Compose-based workflows, a common SOPS pattern is:

```bash
sops exec-file --no-fifo .env "docker compose --env-file {} up -d"
```

Komodo does not currently replace that decryption step. Instead, you can keep using SOPS on the
host and feed the resulting file into Compose, mounted config files, or `_FILE` variables.

## Choosing A Pattern

Use `_FILE` variables when Komodo or Docker only needs a small set of secret values.

Use mounted config files when the configuration is larger, structured, or easier to manage as a
full file.

Use a host-side decryption step when your source of truth is already SOPS-encrypted and you want to
keep Komodo out of the decryption role.

## Related Pages

- [Docker Secrets And `_FILE` Variables](./docker-secrets-files.md)
- [Mounted Config Files](./mounted-config-files.mdx)
- [Variables and Secrets](../configuration/variables.md)
- [Install Komodo](../setup/install-komodo.mdx)
