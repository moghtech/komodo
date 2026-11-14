# External Secret Providers (Infisical)

Komodo supports resolving secrets at execution time from external secret managers via **Settings → Providers → Secret Providers**. **Infisical** (Cloud US/EU and self-hosted HTTPS) is the first official provider, built on a provider-agnostic adapter layer.

Komodo preserves its existing execution pipeline (`Core Interpolator → Periphery → write_env_file() / BuildKit --secret → Docker`) while enforcing encrypted credential storage, default-deny RBAC grants, verified WSS/TLS transport, and structured secret redaction.

---

## 1. Master Key Configuration & Key Rotation

External Secret Provider credentials (such as Infisical Universal Auth client secrets) are encrypted at rest in MongoDB (`secret_provider_credentials`) using **AES-256-GCM** with random 96-bit nonces and authenticated metadata (`provider_id`, `kind`, `schema_version`, `credential_revision`).

Configure a dedicated 32-byte master key on Komodo Core before creating Secret Providers:

```bash
openssl rand -base64 32 > /etc/komodo/keys/secret_provider_master.key
chmod 0600 /etc/komodo/keys/secret_provider_master.key
```

```toml
# core.config.toml
secret_provider_encryption_key = "file:/config/keys/secret_provider_master.key"
```

:::caution Back Up the Master Key Separately
`secret_provider_encryption_key` is **never** stored in MongoDB and must not reuse `jwt_secret`. Back up the key file in a separate secure vault or offline backup. If MongoDB is restored without the matching master key, stored Secret Provider credentials cannot be decrypted and must be re-entered. When the master key is absent, Komodo fails closed and never falls back to plaintext storage.
:::

### Zero-Downtime Key Rotation

1. Move your current key into `secret_provider_previous_encryption_keys` (`KOMODO_SECRET_PROVIDER_PREVIOUS_ENCRYPTION_KEYS`).
2. Set a newly generated 32-byte key (optionally prefixed with `<key_id>:`, e.g. `k2027:<base64>`) in `secret_provider_encryption_key` and restart Komodo Core.
3. Click **Rotate Encryption Keys** in **Settings → Providers → Secret Providers** (or call `RotateSecretProviderEncryptionKeys`).
4. Remove the retired key from `secret_provider_previous_encryption_keys` after your older database backups age out.

---

## 2. Setting Up Infisical (Universal Auth + Secrets API v4)

1. In Infisical, create a **Machine Identity** with **Universal Auth** enabled (`Client ID` + `Client Secret`).
2. Assign a **least-privilege Project Role** scoped only to the required environment(s) and secret folder(s):
   - Grant **Read Value** (`ReadValue`) and **Describe Secret** (`DescribeSecret`) on the target paths.
   - Note: Infisical v4 returns `secretValueHidden: true` when a role only has metadata permission (`DescribeSecret`) without `ReadValue`. Komodo's **Test Connection** tool checks authentication, metadata discovery, and actual value-read permission separately so hidden-value misconfigurations are caught before deploy.
3. In Komodo (**Settings → Providers → Secret Providers → New Secret Provider**):
   - Choose an immutable **Slug** (`[a-z0-9-]+`, no dots, e.g. `infisical-prod`). Dots are prohibited in slugs so `[[vault.<slug>.<secret.key.with.dots>]]` is unambiguous.
   - Select **Infisical Cloud (US)** (`https://app.infisical.com`), **Infisical Cloud (EU)** (`https://eu.infisical.com`), or enter your **Self-Hosted HTTPS** URL.
   - For self-hosted instances on internal networks or private PKI, enable **Allow Private / Internal IP Ranges** (cloud metadata `169.254.169.254` and link-local ranges remain unconditionally blocked) and paste your **Custom Root CA Certificate PEM**. Never disable TLS verification.
   - Click **Test Connection** to verify Stage 1 (`POST /api/v1/auth/universal-auth/login`), Stage 2 (`GET /api/v4/secrets?viewSecretValue=false`), and Stage 3 (`GET /api/v4/secrets` value read).

:::important Credential Target-Change Guard
Read APIs never return client secrets or ciphertexts. Editing a provider uses explicit **Keep / Replace / Clear** actions (`********` masks are rejected). If you change `endpoint`, `client_id`, `organization_slug`, `allow_private_ips`, or `ca_cert_pem`, Komodo blocks **Keep** and requires an explicit **Replace** or **Clear** so stored credentials are never silently sent to a different endpoint.
:::

---

## 3. Authorization Grants (`SecretProviderGrant`)

Access to every Secret Provider is **default-deny**, including for admins at runtime unless a matching enabled grant exists. Under **Grants**, bind each provider to:

- **Subject**:
  - `ResourceAuthorized`: Any enabled human/API-key user who holds the required Komodo permission (`Read` for metadata/preview, `Execute` for deploys/builds) on the target resource.
  - `User` / `UserGroup`: A specific user, service user, or group (`everyone` or named group).
  - `SystemActor`: Required for automated executions (`Procedure`, `Webhook`, `Schedule`, `AutoRedeploy`, `ResourceSync`, `Action`), optionally pinned to a specific trigger resource ID. Automated actors never inherit implicit access.
- **Allowed Resources**: `AllStacks`, `AllDeployments`, `AllBuilds`, or specific resource IDs.
- **Infisical Selector Scope**: Allowed `project_ids`, `environments`, normalized `path_prefixes` (path traversal and encoded `%2e` segments are rejected), plus explicit `allow_imports` and `allow_expand_references` toggles (both default to `false`).
- **Allowed Usages**: `StackEnv`, `StackComposeFileSecret`, `DeploymentEnv`, `BuildSecret`, `MetadataList`, `Preview`.

:::warning Imports & Secret Reference Expansion
When `include_imports` or `expand_references` is enabled, Infisical may read upstream folders or referenced secrets outside the initial `secret_path`. Because upstream responses do not enumerate transitive dependency graphs, enforce folder boundaries on the Infisical Machine Identity role itself.
:::

---

## 4. Literal Interpolation & Recommended Connection Strings

Komodo resolves inline external references using the reserved `vault.` namespace:

```ini
# Recommended: store the complete, URL-encoded connection string in Infisical
DATABASE_URL=[[vault.infisical-prod.DATABASE_URL]]

# Supported: mix plain text, Komodo [[VARIABLES]], and multiple [[vault...]] tokens
MIXED_URL=postgres://[[DB_USER]]:[[vault.infisical-prod.DB_PASSWORD]]@db:5432/app?ro=[[vault.infisical-eu.RO_PASSWORD]]
```

### Strict Literal Substitution Contract

- **Zero encoding or formatting**: If `DATABASE_URL` in Infisical is `postgres://user:p%40ss%23word%2F123@db:5432/app`, Komodo substitutes those exact bytes (`%40` is **not** double-encoded to `%2540`). Conversely, if `DB_PASSWORD` contains raw `@`, `#`, `%`, `$`, spaces, or quotes in a mixed URL, Komodo inserts those raw characters without URL-encoding them. Prefer storing full pre-encoded URLs in Infisical.
- **Single-pass, no rescanning**: Remote secret values containing `[[OTHER_KEY]]`, `${HOST_VAR}`, or `$DOLLAR` are never rescanned by Komodo's interpolator, and Periphery serializes `.env` values losslessly (escaping `$` as `$$` in single-quoted Compose dotenv entries where needed) so Docker Compose delivers the exact literal string into containers.
- **Per-execution snapshot & deduplication**: Repeated references to the same `(provider, selector, key)` within one execution are fetched once from Infisical and reused in memory for that execution only. Resolved snapshots are never persisted to MongoDB or cross-execution caches.

---

## 5. Resource Targets & Security Boundaries

### Docker Compose (`Stack`)

1. **Stack `.env` (`environment` inline refs & `ComposeEnv` bindings)**:
   - Written atomically (`0600`, symlink/traversal-safe) by Periphery's `write_env_file()` to the Stack deployment directory `.env` and passed to Docker Compose via `--env-file`.
   - If your `compose.yaml` already references `${DATABASE_URL}`, setting `DATABASE_URL=[[vault.infisical-prod.DATABASE_URL]]` in `stack.config.environment` works naturally without repeating service mappings.
   - **Accepted `.env` Risk Boundary**: Resolved values exist in plaintext inside the target server's `.env` file (`0600`). Anyone with `root` / Periphery user shell access or host disk/VM backups on that server can read `.env`. Komodo excludes `.env` and resolved values from `StackInfo`, `GetComposeContentsOnHost`, `Inspect*`, runtime logs, `Update` logs, `ResourceSync`, and Git commits.
2. **Service Environment (`ServiceEnv` binding)**:
   - Injects variables into explicit target `services` via a controlled `override.compose.yaml` (`0600`) + per-service `env_file` without exposing them to sibling services in the same Compose project.
3. **Compose File Secrets (`ComposeFileSecret` binding)**:
   - Writes immutable `0400` secret files under a Periphery-controlled runtime directory (`<stack_dir>/.komodo-secrets/<stack_id>/generations/<gen_id>/`) with a persistent `manifest.json`, mounting them into explicit target `services` at `/run/secrets/<secret_name>` (or custom `target_path`).
   - `StopStack`, `StartStack`, and `RestartStack` reuse the active deployed generation on disk without re-contacting Infisical. `DestroyStack` cleans up runtime secret directories.
4. **Change Detection & Rotation (`DeployStackIfChanged`)**:
   - Komodo stores keyed HMAC-SHA256 digests (`SecretDeploymentMetadata`, never plaintext or unkeyed hashes) of deployed secret bindings. Running `DeployStackIfChanged` fetches current values once, compares keyed digests against the active deployment, and redeploys when a secret rotates in Infisical. V1 does not background-poll Infisical for automatic rotation redeploys.

### Containers (`Deployment`)

- Supports inline `[[vault.<slug>.<key>]]` in `deployment.config.environment` and structured `DeploymentSecretBinding` entries, passed safely via shell-escaped `-e` arguments and masked as `<REDACTED>` in `InspectDeploymentContainer` / `InspectContainer`.

### Docker Images (`Build`)

- Supports inline `[[vault.<slug>.<key>]]` in `build.config.secret_args` and structured `BuildSecretBinding` entries **only** (never `build_args` or `dockerfile`, which leak into image history).
- Periphery writes each build secret (`0600`) into an isolated `0700` temporary directory **outside** the Git repo / Docker build context, passes `--secret id=<secret_id>,src=<temp_file>`, and deletes the directory immediately on completion, failure, timeout, or cancellation.
- Consume secrets in your `Dockerfile` with BuildKit secret mounts:
  ```dockerfile
  RUN --mount=type=secret,id=npm_token \
      NPM_TOKEN="$(cat /run/secrets/npm_token)" npm ci
  ```
- **Layer Cache Warning**: Never copy `/run/secrets/...` into a regular image file, `ENV`, or `ARG`. Enable `secret_cache_bust = true` on the Build if you want Komodo to pass a non-sensitive `KOMODO_SECRET_CACHE_NONCE` build arg so rotated secrets invalidate BuildKit's `RUN` cache.

### ResourceSync (`komodo.toml`) Example

`ResourceSync` exports and imports preserve symbolic `[[vault...]]` tokens and structured bindings without resolving secret values:

```toml
[[stack]]
name = "billing-api"
[stack.config]
server = "prod-server-1"
environment = """
DATABASE_URL=[[vault.infisical-prod.DATABASE_URL]]
"""

[[stack.config.secret_sources]]
name = "infisical-payments"
provider = "infisical-prod"
selector = { type = "Infisical", params = { project_id = "proj_123", environment = "prod", secret_path = "/payments" } }

[[stack.config.secret_bindings]]
source = "infisical-payments"
remote_key = "STRIPE_WEBHOOK_SECRET"
target = { type = "ComposeFileSecret", params = { secret_name = "stripe_webhook_secret", services = ["api"], mode = 256 } }
required = true
allow_empty = false
```

---

## 6. Confidential Core ↔ Periphery Transport Requirement

Before sending any external secret value to a Server or Builder, Komodo Core verifies that the active Core ↔ Periphery connection uses:

1. **End-to-end verified `wss://` TLS** (`tls_insecure = false`), for both inbound (`Core → Periphery`) and outbound (`Periphery → Core`) connection modes. Plaintext `ws://` (including reverse proxies terminating TLS into plaintext `ws://`) and unverified self-signed TLS (`tls_insecure = true`) are rejected before secret delivery.
   - For inbound Periphery with a private CA, configure `ca_cert_pem` on the Server / URL Builder config.
   - For outbound Periphery connecting to a Core with a private CA, configure `core_ca_cert_pem = "file:/etc/komodo/ssl/core-ca.pem"` (`PERIPHERY_CORE_CA_CERT_PEM_FILE`) in `periphery.config.toml`.
2. **Mutual Noise static key authentication** and a Periphery version advertising external secret lifecycle support.

---

## 7. Unsupported Scope in V1 & Migrating from `compose_cmd_wrapper`

- **Unsupported in V1**: Docker Swarm native secrets (`swarm_id` resources fail closed when external secrets are present), `Repo` / `Action` / `Alerter` fields, `Build.build_args` / `Build.dockerfile`, `Stack.file_contents` / `pre_deploy` / `post_deploy`, dynamic secret leases, and background polling auto-rotation.
- **Migrating from `compose_cmd_wrapper`**: V1 intentionally fails closed if a Stack configures both native external secrets and an active `compose_cmd_wrapper` (such as `infisical run --` or `sops exec-env`). To migrate:
  1. Create your Infisical Secret Provider and Grant under **Settings → Providers → Secret Providers**.
  2. Replace wrapper-injected variables with `[[vault.<slug>.<key>]]` in `stack.config.environment` or structured `secret_bindings`.
  3. Clear `compose_cmd_wrapper` (and `compose_cmd_wrapper_include`) on the Stack, then deploy.
