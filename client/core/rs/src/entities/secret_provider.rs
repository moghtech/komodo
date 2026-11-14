use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumDiscriminants, EnumString};
use typeshare::typeshare;

use super::{I64, MongoId, ResourceTargetVariant, Usize};

/// Validate a SecretProvider slug:
/// - Lowercase ASCII letters, digits, and single hyphens (`[a-z0-9]+(?:-[a-z0-9]+)*`)
/// - Length 1..=64
/// - Explicitly forbids `.` so `[[vault.<provider-slug>.<secret-key>]]` has zero ambiguity
///   even when `<secret-key>` contains dots.
pub fn validate_provider_slug(slug: &str) -> anyhow::Result<()> {
  if slug.is_empty() {
    anyhow::bail!("Secret provider slug cannot be empty");
  }
  if slug.len() > 64 {
    anyhow::bail!(
      "Secret provider slug '{slug}' exceeds maximum length of 64 characters"
    );
  }
  if slug.contains('.') {
    anyhow::bail!(
      "Secret provider slug '{slug}' must not contain '.'"
    );
  }
  if slug.starts_with('-') || slug.ends_with('-') {
    anyhow::bail!(
      "Secret provider slug '{slug}' must not start or end with '-'"
    );
  }
  if slug.contains("--") {
    anyhow::bail!(
      "Secret provider slug '{slug}' must not contain consecutive '--'"
    );
  }
  if !slug
    .chars()
    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
  {
    anyhow::bail!(
      "Secret provider slug '{slug}' may only contain lowercase ASCII letters (a-z), digits (0-9), and hyphens (-)"
    );
  }
  Ok(())
}

/// Normalize and strictly validate an Infisical secret path.
/// Rejects directory traversal (`..`), percent-encoded traversal/slashes, backslashes,
/// NUL/control characters, query/fragment characters, and collapses duplicate `/`
/// only if canonical, returning a canonical path starting with `/` (without trailing `/` unless `"/"`).
pub fn normalize_secret_path(raw: &str) -> anyhow::Result<String> {
  let trimmed = raw.trim();
  if trimmed.is_empty() {
    return Ok(String::from("/"));
  }
  if trimmed.bytes().any(|b| b < 0x20 || b == 0x7f) {
    anyhow::bail!("Secret path contains control or NUL characters");
  }
  if trimmed.contains('\\')
    || trimmed.contains('?')
    || trimmed.contains('#')
  {
    anyhow::bail!(
      "Secret path '{trimmed}' must not contain '\\', '?', or '#'"
    );
  }
  let lower = trimmed.to_ascii_lowercase();
  if lower.contains("%2e")
    || lower.contains("%2f")
    || lower.contains("%5c")
    || lower.contains("%00")
  {
    anyhow::bail!(
      "Secret path '{trimmed}' must not contain percent-encoded path or traversal sequences"
    );
  }
  if !trimmed.starts_with('/') {
    anyhow::bail!("Secret path '{trimmed}' must start with '/'");
  }
  let mut segments = Vec::new();
  for seg in trimmed.split('/') {
    if seg.is_empty() {
      continue;
    }
    if seg == "." || seg == ".." {
      anyhow::bail!(
        "Secret path '{trimmed}' must not contain '.' or '..' traversal segments"
      );
    }
    segments.push(seg);
  }
  if segments.is_empty() {
    Ok(String::from("/"))
  } else {
    Ok(format!("/{}", segments.join("/")))
  }
}

/// Check whether `candidate_path` is within `allowed_prefix` using normalized path segments
/// (e.g. prefix `"/app"` matches `"/app"` and `"/app/db"`, NOT `"/app-other"`).
pub fn secret_path_matches_prefix(
  candidate_path: &str,
  allowed_prefix: &str,
) -> anyhow::Result<bool> {
  let candidate = normalize_secret_path(candidate_path)?;
  let prefix = normalize_secret_path(allowed_prefix)?;
  if prefix == "/" {
    return Ok(true);
  }
  if candidate == prefix {
    return Ok(true);
  }
  Ok(
    candidate
      .strip_prefix(&prefix)
      .is_some_and(|rest| rest.starts_with('/')),
  )
}

/// Parsed inline external secret reference from `[[vault.<provider-slug>.<secret-key>]]`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExternalSecretRef {
  pub provider_slug: String,
  pub secret_key: String,
}

impl ExternalSecretRef {
  pub fn canonical_token(&self) -> String {
    format!("vault.{}.{}", self.provider_slug, self.secret_key)
  }

  /// Parse the inner text of `[[...]]` if it targets the `vault` namespace.
  /// Returns:
  /// - `Ok(None)` if the token does NOT belong to the `vault` namespace.
  /// - `Ok(Some(ExternalSecretRef))` if valid `vault.<provider-slug>.<secret-key>`.
  /// - `Err(...)` (fail-closed) if it starts with/resembles `vault` but is malformed or uses unsupported filters.
  pub fn parse_bracket_token(
    token: &str,
  ) -> anyhow::Result<Option<Self>> {
    let trimmed = token.trim();
    if trimmed == "vault"
      || trimmed.starts_with("vault.")
      || trimmed.starts_with("vault:")
      || trimmed.starts_with("vault/")
    {
      if token != trimmed {
        anyhow::bail!(
          "Malformed external secret reference '[[{token}]]': unexpected whitespace"
        );
      }
      let Some(rest) = token.strip_prefix("vault.") else {
        anyhow::bail!(
          "Malformed external secret reference '[[{token}]]': expected syntax [[vault.<provider-slug>.<secret-key>]]"
        );
      };
      if rest.contains('|') {
        anyhow::bail!(
          "Unsupported filter syntax in '[[{token}]]': external secret references only perform raw literal replacement"
        );
      }
      let Some((slug, key)) = rest.split_once('.') else {
        anyhow::bail!(
          "Malformed external secret reference '[[{token}]]': missing '<secret-key>' after provider slug"
        );
      };
      validate_provider_slug(slug).map_err(|e| {
        anyhow::anyhow!(
          "Invalid provider slug in external secret reference '[[{token}]]': {e}"
        )
      })?;
      validate_inline_secret_key(key).map_err(|e| {
        anyhow::anyhow!(
          "Invalid secret key in external secret reference '[[{token}]]': {e}"
        )
      })?;
      return Ok(Some(Self {
        provider_slug: slug.to_string(),
        secret_key: key.to_string(),
      }));
    }
    Ok(None)
  }
}

/// Validate inline `[[vault.<slug>.<secret-key>]]` key segment (allows dots `.`, underscores, hyphens, ASCII alphanumerics).
/// Keys containing unusual special characters can still be mapped via structured `SecretBinding`.
pub fn validate_inline_secret_key(key: &str) -> anyhow::Result<()> {
  if key.is_empty() {
    anyhow::bail!("secret key cannot be empty");
  }
  if key.starts_with('.') || key.ends_with('.') {
    anyhow::bail!(
      "secret key '{key}' must not start or end with '.'"
    );
  }
  if !key.chars().all(|c| {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')
  }) {
    anyhow::bail!(
      "secret key '{key}' contains unsupported inline characters; use structured secret bindings for special-character keys"
    );
  }
  Ok(())
}

/// Public representation of an external Secret Provider managed at `Settings -> Providers -> Secrets`.
/// Never contains bootstrap credentials or encrypted credential envelopes.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, PartialEq,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(
  feature = "mongo",
  derive(mongo_indexed::derive::MongoIndexed)
)]
pub struct SecretProvider {
  #[serde(
    default,
    rename = "_id",
    skip_serializing_if = "String::is_empty",
    with = "bson::serde_helpers::hex_string_as_object_id"
  )]
  pub id: MongoId,
  /// Immutable, globally unique slug (`[a-z0-9-]+`, no dots) used in `[[vault.<slug>.<key>]]`.
  #[cfg_attr(feature = "mongo", unique_index)]
  #[serde(default)]
  pub slug: String,
  /// Mutable human-readable display name.
  #[cfg_attr(feature = "mongo", unique_index)]
  #[serde(default)]
  pub name: String,
  #[serde(default)]
  pub description: String,
  #[cfg_attr(feature = "mongo", index)]
  #[serde(default = "default_true")]
  pub enabled: bool,
  /// Optimistic concurrency revision (increments on every config/state/credential mutation).
  #[serde(default = "default_revision")]
  pub revision: I64,
  /// Credential revision (increments whenever stored credentials are replaced or cleared).
  #[serde(default)]
  pub credential_revision: I64,
  /// Whether an encrypted credential envelope is currently stored for this provider.
  #[serde(default)]
  pub has_credentials: bool,
  #[serde(default)]
  pub updated_at: I64,
  pub config: SecretProviderConfig,
}

fn default_true() -> bool {
  true
}

fn default_revision() -> I64 {
  1
}

fn default_infisical_endpoint() -> String {
  String::from("https://app.infisical.com")
}

fn default_secret_path() -> String {
  String::from("/")
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  PartialEq,
  Eq,
  Serialize,
  Deserialize,
  EnumDiscriminants,
)]
#[strum_discriminants(name(SecretProviderKind))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(
  not(feature = "utoipa"),
  strum_discriminants(derive(
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    AsRefStr
  ))
)]
#[cfg_attr(
  feature = "utoipa",
  strum_discriminants(derive(
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    AsRefStr,
    utoipa::ToSchema
  ))
)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(tag = "type", content = "params")]
pub enum SecretProviderConfig {
  Infisical(InfisicalProviderConfig),
}

impl Default for SecretProviderConfig {
  fn default() -> Self {
    Self::Infisical(InfisicalProviderConfig::default())
  }
}

impl SecretProviderConfig {
  /// Returns true if changing from `self` to `other` alters the authentication target or identity,
  /// requiring explicit credential replacement (`Replace`) or clearing (`Clear`) rather than `Keep`.
  pub fn target_or_identity_changed(&self, other: &Self) -> bool {
    match (self, other) {
      (
        SecretProviderConfig::Infisical(a),
        SecretProviderConfig::Infisical(b),
      ) => {
        a.endpoint.trim().trim_end_matches('/')
          != b.endpoint.trim().trim_end_matches('/')
          || a.client_id.trim() != b.client_id.trim()
          || a.organization_slug.trim() != b.organization_slug.trim()
          || a.ca_cert_pem.trim() != b.ca_cert_pem.trim()
          || a.allow_private_ips != b.allow_private_ips
      }
    }
  }
}

#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct InfisicalProviderConfig {
  /// Base HTTPS URL for Infisical Cloud (`https://app.infisical.com` or `https://eu.infisical.com`)
  /// or a self-hosted Infisical instance.
  #[serde(default = "default_infisical_endpoint")]
  pub endpoint: String,
  /// Machine Identity Universal Auth Client ID (non-secret identifier).
  #[serde(default)]
  pub client_id: String,
  /// Optional Organization slug for Universal Auth login scoping.
  #[serde(default)]
  pub organization_slug: String,
  /// Optional PEM-encoded root CA certificate(s) for verifying self-hosted TLS endpoints.
  #[serde(default)]
  pub ca_cert_pem: String,
  /// Admin-controlled flag allowing connections to private/internal IP ranges for self-hosted Infisical
  /// (cloud metadata and link-local ranges remain unconditionally blocked).
  #[serde(default)]
  pub allow_private_ips: bool,
  /// Optional default scope selector used when inline `[[vault.<slug>.<key>]]` references
  /// do not specify a resource-level `SecretSource` override.
  #[serde(default)]
  pub default_selector: Option<InfisicalSecretSelector>,
}

impl Default for InfisicalProviderConfig {
  fn default() -> Self {
    Self {
      endpoint: default_infisical_endpoint(),
      client_id: String::new(),
      organization_slug: String::new(),
      ca_cert_pem: String::new(),
      allow_private_ips: false,
      default_selector: None,
    }
  }
}

/// Capabilities reported by a SecretProvider adapter.
#[typeshare]
#[derive(
  Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SecretProviderCapabilities {
  pub supports_metadata_listing: bool,
  pub supports_versions: bool,
  pub supports_binary_values: bool,
  pub supports_json_selector: bool,
  pub supports_leases: bool,
}

/// Safe non-admin summary of a `SecretProvider` returned to resource pickers.
/// Never exposes endpoint credentials, private CAs, or internal auth IDs.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SecretProviderSummary {
  pub id: String,
  pub slug: String,
  pub name: String,
  #[serde(default)]
  pub description: String,
  pub kind: SecretProviderKind,
  pub enabled: bool,
  pub has_credentials: bool,
  pub capabilities: SecretProviderCapabilities,
  pub default_selector: Option<SecretSelector>,
  /// Whether the caller currently holds an active grant for the requested resource/metadata scope.
  pub granted: bool,
}

/// Write-only bootstrap credential input for a Secret Provider.
/// Implements custom `Debug` so request debug logs never print raw secrets.
#[typeshare]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", content = "params")]
pub enum SecretProviderCredentialInput {
  Infisical { client_secret: String },
}

impl std::fmt::Debug for SecretProviderCredentialInput {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Infisical { .. } => f
        .debug_struct("Infisical")
        .field("client_secret", &"[REDACTED]")
        .finish(),
    }
  }
}

/// Explicit credential mutation mode on provider updates (never uses `********` sentinel values).
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", content = "params")]
pub enum CredentialUpdate {
  #[default]
  Keep,
  Replace(SecretProviderCredentialInput),
  Clear,
}

/// Infisical v4 secret selector (`projectId`, `environment`, `secretPath`, `includeImports`, `expandSecretReferences`).
#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct InfisicalSecretSelector {
  pub project_id: String,
  pub environment: String,
  #[serde(default = "default_secret_path")]
  pub secret_path: String,
  /// Explicitly defaults to `false`; requires admin grant `allow_imports = true` when enabled.
  #[serde(default)]
  pub include_imports: bool,
  /// Explicitly defaults to `false`; requires admin grant `allow_expand_references = true` when enabled.
  #[serde(default)]
  pub expand_references: bool,
}

impl Default for InfisicalSecretSelector {
  fn default() -> Self {
    Self {
      project_id: String::new(),
      environment: String::new(),
      secret_path: default_secret_path(),
      include_imports: false,
      expand_references: false,
    }
  }
}

#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(tag = "type", content = "params")]
pub enum SecretSelector {
  Infisical(InfisicalSecretSelector),
}

impl Default for SecretSelector {
  fn default() -> Self {
    Self::Infisical(InfisicalSecretSelector::default())
  }
}

/// Named secret source configured on a Stack, Deployment, or Build.
/// Stores provider reference and selector metadata only — never secret values.
#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SecretSource {
  /// Local alias used by `SecretBinding.source` (if empty, matches `provider` slug).
  #[serde(default)]
  pub name: String,
  /// Immutable `SecretProvider.slug`.
  pub provider: String,
  pub selector: SecretSelector,
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  Default,
  PartialEq,
  Eq,
  Hash,
  Serialize,
  Deserialize,
  Display,
  EnumString,
  AsRefStr,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum SecretConflictPolicy {
  #[default]
  Fail,
  ReplaceExisting,
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  Default,
  PartialEq,
  Eq,
  Hash,
  Serialize,
  Deserialize,
  Display,
  EnumString,
  AsRefStr,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum SecretValueFormat {
  #[default]
  Text,
  Binary,
}

/// Explicit injection target for a Stack structured secret binding.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(tag = "type", content = "params")]
pub enum StackSecretTarget {
  /// Injects `variable` into the Stack's `.env` file (`write_env_file()`) for Compose `${VAR}` interpolation.
  ComposeEnv { variable: String },
  /// Injects `variable` directly into the container environment of the specified Compose `services`.
  ServiceEnv {
    variable: String,
    services: Vec<String>,
  },
  /// Mounts an immutable controlled file secret into the specified Compose `services` via Compose `secrets:`.
  ComposeFileSecret {
    secret_name: String,
    services: Vec<String>,
    #[serde(default)]
    target_path: String,
    #[serde(default)]
    uid: String,
    #[serde(default)]
    gid: String,
    #[serde(default)]
    mode: Option<u32>,
  },
}

impl Default for StackSecretTarget {
  fn default() -> Self {
    Self::ComposeEnv {
      variable: String::new(),
    }
  }
}

#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct StackSecretBinding {
  /// References `SecretSource.name` on this Stack, or a `SecretProvider.slug` directly.
  pub source: String,
  /// Exact remote secret key name (dots preserved literally).
  pub remote_key: String,
  pub target: StackSecretTarget,
  #[serde(default)]
  pub format: SecretValueFormat,
  /// Reserved extension point for future JSON field extraction; unsupported in Infisical v1 (must be empty).
  #[serde(default, skip_serializing_if = "String::is_empty")]
  pub json_field: String,
  #[serde(default = "default_true")]
  pub required: bool,
  #[serde(default = "default_true")]
  pub allow_empty: bool,
  #[serde(default)]
  pub conflict_policy: SecretConflictPolicy,
}

#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct DeploymentSecretBinding {
  /// References `SecretSource.name` on this Deployment, or a `SecretProvider.slug` directly.
  pub source: String,
  /// Exact remote secret key name.
  pub remote_key: String,
  /// Target container environment variable name.
  pub variable: String,
  #[serde(default, skip_serializing_if = "String::is_empty")]
  pub json_field: String,
  #[serde(default = "default_true")]
  pub required: bool,
  #[serde(default = "default_true")]
  pub allow_empty: bool,
  #[serde(default)]
  pub conflict_policy: SecretConflictPolicy,
}

#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct BuildSecretBinding {
  /// References `SecretSource.name` on this Build, or a `SecretProvider.slug` directly.
  pub source: String,
  /// Exact remote secret key name.
  pub remote_key: String,
  /// Target BuildKit `--secret id=<secret_id>` mount identifier.
  pub secret_id: String,
  #[serde(default)]
  pub format: SecretValueFormat,
  #[serde(default, skip_serializing_if = "String::is_empty")]
  pub json_field: String,
  #[serde(default = "default_true")]
  pub required: bool,
  #[serde(default = "default_true")]
  pub allow_empty: bool,
  #[serde(default)]
  pub conflict_policy: SecretConflictPolicy,
}

// ==========================================
// = Authorization Grants & Subject Scopes  =
// ==========================================

/// Automated / indirect execution subjects in Komodo that require explicit grant authorization
/// (never implicitly authorized via built-in `admin: true` service user flags).
#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  Hash,
  Serialize,
  Deserialize,
  Display,
  EnumString,
  AsRefStr,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum SecretExecutionActor {
  Procedure,
  Webhook,
  Schedule,
  AutoRedeploy,
  ResourceSync,
  Action,
}

#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", content = "params")]
pub enum SecretGrantSubject {
  /// Grants usage to a specific real Komodo User (human or service user / API key owner).
  User { user_id: String },
  /// Grants usage to members of a Komodo UserGroup (`everyone` or explicit membership).
  UserGroup { group_id: String },
  /// Grants usage to an explicit indirect execution actor (optionally restricted to a specific trigger resource ID, e.g. Procedure ID).
  SystemActor {
    actor: SecretExecutionActor,
    #[serde(default)]
    actor_resource_id: String,
  },
  /// Grants usage to any real authenticated & enabled user who holds the required Komodo permission
  /// (`Read`/`Write`/`Execute`) on one of `allowed_resources`. Does NOT match indirect system actors.
  ResourceAuthorized,
}

impl Default for SecretGrantSubject {
  fn default() -> Self {
    Self::ResourceAuthorized
  }
}

#[typeshare]
#[derive(
  Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", content = "params")]
pub enum SecretResourceScope {
  Stack { id: String },
  Deployment { id: String },
  Build { id: String },
  AllStacks,
  AllDeployments,
  AllBuilds,
}

#[typeshare]
#[derive(
  Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct InfisicalSelectorScope {
  /// Allowed Infisical `projectId`s (exact match).
  #[serde(default)]
  pub project_ids: Vec<String>,
  /// Allowed Infisical environment slugs (exact match, e.g. `["prod"]`).
  #[serde(default)]
  pub environments: Vec<String>,
  /// Allowed normalized secret path segment prefixes (e.g. `["/"]` or `["/api"]`).
  #[serde(default)]
  pub path_prefixes: Vec<String>,
  /// Whether resources under this grant may set `include_imports = true` (default `false`).
  #[serde(default)]
  pub allow_imports: bool,
  /// Whether resources under this grant may set `expand_references = true` (default `false`).
  #[serde(default)]
  pub allow_expand_references: bool,
}

#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(tag = "type", content = "params")]
pub enum SecretSelectorScope {
  Infisical(InfisicalSelectorScope),
}

impl Default for SecretSelectorScope {
  fn default() -> Self {
    Self::Infisical(InfisicalSelectorScope::default())
  }
}

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  Hash,
  Serialize,
  Deserialize,
  Display,
  EnumString,
  AsRefStr,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum SecretUsage {
  MetadataList,
  Preview,
  StackEnv,
  StackComposeFileSecret,
  DeploymentEnv,
  BuildSecret,
}

/// Admin-managed authorization grant binding a `SecretProvider` to subjects, resources,
/// remote selector scopes, and allowed usages. Default policy is deny-all when no grant matches.
#[typeshare]
#[derive(
  Serialize, Deserialize, Debug, Clone, Default, PartialEq,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(
  feature = "mongo",
  derive(mongo_indexed::derive::MongoIndexed)
)]
pub struct SecretProviderGrant {
  #[serde(
    default,
    rename = "_id",
    skip_serializing_if = "String::is_empty",
    with = "bson::serde_helpers::hex_string_as_object_id"
  )]
  pub id: MongoId,
  #[cfg_attr(feature = "mongo", index)]
  pub provider_id: String,
  #[serde(default)]
  pub name: String,
  #[serde(default)]
  pub description: String,
  #[cfg_attr(feature = "mongo", index)]
  #[serde(default = "default_true")]
  pub enabled: bool,
  #[serde(default = "default_revision")]
  pub revision: I64,
  pub subject: SecretGrantSubject,
  #[serde(default)]
  pub allowed_resources: Vec<SecretResourceScope>,
  pub selector_scope: SecretSelectorScope,
  #[serde(default)]
  #[cfg_attr(feature = "utoipa", schema(value_type = Vec<SecretUsage>))]
  pub allowed_usages: IndexSet<SecretUsage>,
  #[serde(default)]
  pub updated_at: I64,
}

// ==========================================
// = Non-Sensitive Discovery / Test / Usage =
// ==========================================

#[typeshare]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  Serialize,
  Deserialize,
  Display,
  EnumString,
  AsRefStr,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum SecretCheckStageStatus {
  Passed,
  Failed,
  Skipped,
}

#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SecretTestStageResult {
  pub status: SecretCheckStageStatus,
  pub message: String,
}

/// Structured 3-stage connection test result:
/// reports authentication, scope metadata discovery, and actual value-read permission separately.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TestSecretProviderResponse {
  pub ready_for_deploy: bool,
  pub authentication: SecretTestStageResult,
  pub metadata_access: SecretTestStageResult,
  pub value_read_access: SecretTestStageResult,
  pub key_count: Option<Usize>,
}

/// Metadata-only secret descriptor returned to UI selectors (never includes secret values).
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ExternalSecretMetadataItem {
  pub key: String,
  pub version: Option<I64>,
  #[serde(default)]
  pub comment: String,
  #[serde(default)]
  pub imported: bool,
  #[serde(default)]
  pub source_environment: String,
  #[serde(default)]
  pub source_path: String,
  #[serde(default)]
  pub value_hidden: bool,
}

/// Dependency summary showing which Stack/Deployment/Build references a `SecretProvider`.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SecretProviderResourceUsage {
  pub resource_type: ResourceTargetVariant,
  pub resource_id: String,
  pub resource_name: String,
  pub reference_kinds: Vec<String>,
  pub keys: Vec<String>,
}

/// Non-sensitive preview item showing target variable/secret name, referenced external keys, and status.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ResourceSecretPreviewItem {
  pub target: String,
  pub references: Vec<String>,
  pub status: String,
  pub ok: bool,
  pub version: Option<I64>,
}

#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PreviewResourceSecretsResponse {
  pub valid: bool,
  pub items: Vec<ResourceSecretPreviewItem>,
  pub errors: Vec<String>,
  pub last_deployed_at: Option<I64>,
  #[serde(default)]
  pub deployed_provenance: Vec<DeployedSecretProvenance>,
}

#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct RotateSecretProviderEncryptionKeysResponse {
  pub rotated_count: Usize,
  pub active_key_id: String,
}

/// Non-sensitive provenance record stored for an executed deployment (never stores plaintext or unkeyed hashes).
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct DeployedSecretProvenance {
  pub provider_slug: String,
  pub remote_key: String,
  pub target: String,
  pub version: Option<I64>,
}

// =========================================================
// = Internal DB-Only Envelopes (NOT exported to TS/API!)  =
// =========================================================

/// Encrypted bootstrap credential envelope stored in dedicated `SecretProviderCredential` collection.
/// Never returned by any API endpoint.
#[cfg(feature = "mongo")]
#[derive(
  Serialize, Deserialize, Clone, mongo_indexed::derive::MongoIndexed,
)]
pub struct StoredSecretProviderCredential {
  #[serde(
    default,
    rename = "_id",
    skip_serializing_if = "String::is_empty",
    with = "bson::serde_helpers::hex_string_as_object_id"
  )]
  pub id: MongoId,
  #[unique_index]
  pub provider_id: String,
  pub provider_kind: String,
  pub schema_version: u32,
  pub credential_revision: i64,
  pub key_id: String,
  pub algorithm: String,
  #[serde(with = "serde_bytes_hex_or_vec")]
  pub nonce: Vec<u8>,
  #[serde(with = "serde_bytes_hex_or_vec")]
  pub ciphertext: Vec<u8>,
  pub updated_at: i64,
}

#[cfg(feature = "mongo")]
impl std::fmt::Debug for StoredSecretProviderCredential {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("StoredSecretProviderCredential")
      .field("provider_id", &self.provider_id)
      .field("provider_kind", &self.provider_kind)
      .field("schema_version", &self.schema_version)
      .field("credential_revision", &self.credential_revision)
      .field("key_id", &self.key_id)
      .field("algorithm", &self.algorithm)
      .field("nonce", &"[REDACTED]")
      .field("ciphertext", &"[REDACTED]")
      .finish()
  }
}

/// Internal deployment metadata tracking secret env keys, file secret names, provenance,
/// and keyed HMAC-SHA256 fingerprint for `DeployStackIfChanged` & structured `Inspect` redaction.
/// Never contains plaintext values or unkeyed hashes.
#[cfg(feature = "mongo")]
#[derive(
  Serialize,
  Deserialize,
  Debug,
  Clone,
  Default,
  mongo_indexed::derive::MongoIndexed,
)]
#[unique_doc_index({ "resource_type": 1, "resource_id": 1 })]
pub struct SecretDeploymentMetadata {
  #[serde(
    default,
    rename = "_id",
    skip_serializing_if = "String::is_empty",
    with = "bson::serde_helpers::hex_string_as_object_id"
  )]
  pub id: MongoId,
  #[index]
  pub resource_type: String,
  #[index]
  pub resource_id: String,
  #[index]
  pub server_id: String,
  /// Compose project name (for Stack) or container name (for Deployment), enabling reverse lookup on Server-level `InspectContainer`.
  #[index]
  #[serde(default)]
  pub runtime_project_or_container: String,
  /// All environment variable keys carrying external secrets (covering both whole-value and mixed-string vars).
  #[serde(default)]
  pub secret_env_keys: Vec<String>,
  /// Compose file secret names mounted in this deployment.
  #[serde(default)]
  pub file_secret_names: Vec<String>,
  /// Active runtime secret generation ID on Periphery (when `ComposeFileSecret` bindings are used).
  #[serde(default)]
  pub active_generation: Option<String>,
  /// Keyed HMAC-SHA256 hex digest (computed using master-key-derived HMAC key) for `DeployStackIfChanged` comparison.
  #[serde(default)]
  pub keyed_fingerprint_hmac: String,
  /// Key ID of the master key used to compute `keyed_fingerprint_hmac`.
  #[serde(default)]
  pub fingerprint_key_id: String,
  /// Non-sensitive provenance records (provider slug, remote key, target, version).
  #[serde(default)]
  pub provenance: Vec<DeployedSecretProvenance>,
  pub deployed_at: i64,
}

#[cfg(feature = "mongo")]
mod serde_bytes_hex_or_vec {
  use serde::{Deserialize, Deserializer, Serializer};

  pub fn serialize<S>(
    bytes: &[u8],
    serializer: S,
  ) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let hex_str: String =
      bytes.iter().map(|b| format!("{b:02x}")).collect();
    serializer.serialize_str(&hex_str)
  }

  pub fn deserialize<'de, D>(
    deserializer: D,
  ) -> Result<Vec<u8>, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Repr {
      Hex(String),
      Bytes(Vec<u8>),
    }
    match Repr::deserialize(deserializer)? {
      Repr::Hex(s) => {
        if s.len() % 2 != 0 {
          return Err(serde::de::Error::custom(
            "invalid hex byte length",
          ));
        }
        (0..s.len())
          .step_by(2)
          .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
              .map_err(serde::de::Error::custom)
          })
          .collect()
      }
      Repr::Bytes(v) => Ok(v),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn slug_validation_rules() {
    assert!(validate_provider_slug("infisical-prod").is_ok());
    assert!(validate_provider_slug("a1").is_ok());
    assert!(validate_provider_slug("").is_err());
    assert!(validate_provider_slug("infisical.prod").is_err());
    assert!(validate_provider_slug("Infisical-Prod").is_err());
    assert!(validate_provider_slug("-infisical").is_err());
    assert!(validate_provider_slug("infisical-").is_err());
    assert!(validate_provider_slug("infisical--prod").is_err());
  }

  #[test]
  fn path_normalization_and_prefix_matching() {
    assert_eq!(normalize_secret_path("/").unwrap(), "/");
    assert_eq!(
      normalize_secret_path("  /api/db/ ").unwrap(),
      "/api/db"
    );
    assert!(normalize_secret_path("api/db").is_err());
    assert!(normalize_secret_path("/api/../etc").is_err());
    assert!(normalize_secret_path("/api/%2e%2e/etc").is_err());
    assert!(normalize_secret_path("/api/%2fsecret").is_err());
    assert!(normalize_secret_path("/api\\secret").is_err());

    assert!(secret_path_matches_prefix("/api/db", "/").unwrap());
    assert!(secret_path_matches_prefix("/api/db", "/api").unwrap());
    assert!(secret_path_matches_prefix("/api", "/api").unwrap());
    assert!(!secret_path_matches_prefix("/api-v2", "/api").unwrap());
  }

  #[test]
  fn vault_ref_parsing_allows_dots_in_key_and_rejects_filters_or_malformed()
   {
    assert_eq!(
      ExternalSecretRef::parse_bracket_token("EXISTING_VAR").unwrap(),
      None
    );
    assert_eq!(
      ExternalSecretRef::parse_bracket_token(
        "vault.infisical-prod.DATABASE_URL"
      )
      .unwrap(),
      Some(ExternalSecretRef {
        provider_slug: "infisical-prod".into(),
        secret_key: "DATABASE_URL".into(),
      })
    );
    assert_eq!(
      ExternalSecretRef::parse_bracket_token(
        "vault.infisical-prod.service.db.password"
      )
      .unwrap(),
      Some(ExternalSecretRef {
        provider_slug: "infisical-prod".into(),
        secret_key: "service.db.password".into(),
      })
    );
    // Reject filters, missing key, whitespace, invalid slug
    assert!(
      ExternalSecretRef::parse_bracket_token(
        "vault.infisical-prod.DB_PASSWORD|urlencode"
      )
      .is_err()
    );
    assert!(ExternalSecretRef::parse_bracket_token("vault").is_err());
    assert!(
      ExternalSecretRef::parse_bracket_token("vault.infisical-prod")
        .is_err()
    );
    assert!(
      ExternalSecretRef::parse_bracket_token("vault.infisical-prod.")
        .is_err()
    );
    assert!(
      ExternalSecretRef::parse_bracket_token(
        " vault.infisical-prod.DB_PASSWORD "
      )
      .is_err()
    );
  }
}
