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
- [Mounted Config Files](./mounted-config-files.mdx)
- [Custom CA Certificates](./custom-ca-certificates.md)
