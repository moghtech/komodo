---
sidebar_label: Configure OIDC / OAuth2 Login
---

# Configure OIDC / OAuth2 Login

Komodo supports external login through GitHub, Google, and OIDC-compatible providers such as
Authentik, Gitea, and Keycloak.

Komodo uses the web application login flow. The redirect URI depends on the provider type:

- GitHub: `<KOMODO_HOST>/auth/github/callback`
- Google: `<KOMODO_HOST>/auth/google/callback`
- OIDC: `<KOMODO_HOST>/auth/oidc/callback`

## Core Settings

For generic OIDC providers, configure these Core settings:

- `KOMODO_OIDC_ENABLED=true`
- `KOMODO_OIDC_PROVIDER=https://your-provider.example.com/...`
- `KOMODO_OIDC_CLIENT_ID=...`
- `KOMODO_OIDC_CLIENT_SECRET=...`

GitHub and Google use their provider-specific settings in `compose.env` or `core.config.toml`.

## Registration And Redirect Behavior

Core can control registration separately for local and OIDC logins:

- `KOMODO_DISABLE_USER_REGISTRATION=true`
  - blocks new-user registration for every login path except creation of the first user
- `KOMODO_DISABLE_LOCAL_USER_REGISTRATION=true`
  - hides the local `Sign Up` path and blocks username/password signups
- `KOMODO_DISABLE_OIDC_USER_REGISTRATION=true`
  - blocks creation of new users through OIDC while leaving local signups unchanged

If the split settings are unset, they fall back to `KOMODO_DISABLE_USER_REGISTRATION`.

OIDC also supports automatic redirect to the identity provider:

- `KOMODO_OIDC_AUTO_REDIRECT=true`

When enabled, unauthenticated users are sent directly to the configured OIDC provider instead of
seeing the Komodo login page first.

## Provider Notes

Komodo documents or supports several common providers:

- [Authentik integration docs](https://integrations.goauthentik.io/infrastructure/komodo/)
- [Gitea OAuth2 provider](https://docs.gitea.com/development/oauth2-provider)
- [Keycloak](https://www.keycloak.org)

## Keycloak Example

To configure Keycloak:

1. Create an [OIDC client](https://www.keycloak.org/docs/latest/server_admin/index.html#proc-creating-oidc-client_server_administration_guide).
2. Set `Valid Redirect URIs` to `<KOMODO_HOST>/auth/oidc/callback`.
3. Enable `Client authentication`.
4. Copy the `Client ID` and `Client Secret`.
5. Set the Komodo Core OIDC variables to match that client.

## Related Pages

- [Install Komodo](../setup/install-komodo.mdx)
- [Permissioning](../configuration/permissioning.md)
- [Mounted Config Files](./mounted-config-files.mdx)
- [Custom CA Certificates](./custom-ca-certificates.md)
