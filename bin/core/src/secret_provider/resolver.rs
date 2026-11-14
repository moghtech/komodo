use std::collections::{BTreeMap, BTreeSet, HashMap};

use anyhow::{Context, bail};
use indexmap::IndexMap;
use interpolate::Interpolator;
use komodo_client::{
  entities::{
    EnvironmentVar, ResourceTarget,
    action::Action,
    alerter::{Alerter, AlerterEndpoint},
    build::Build,
    deployment::Deployment,
    permission::PermissionLevel,
    repo::Repo,
    secret_provider::{
      DeployedSecretProvenance, ExternalSecretRef,
      PreviewResourceSecretsResponse, ResourceSecretPreviewItem,
      SecretConflictPolicy, SecretProvider, SecretSelector,
      SecretSource, SecretUsage, SecretValueFormat,
      StackSecretTarget,
    },
    stack::Stack,
    user::User,
  },
  parsers::parse_key_value_list,
};
use tokio_util::sync::CancellationToken;

use super::{
  adapter::{
    AdapterResolveItemRequest, ResolvedSecretValue, get_adapter,
  },
  authz::{
    authorize_secret_access, collect_inline_vault_refs_in_str,
    find_secret_provider, resolve_selector_for_provider,
  },
  crypto::SecretProviderKeyring,
  load_provider_credential,
};

/// Validate a POSIX environment variable name (`[A-Za-z_][A-Za-z0-9_]*`).
pub fn validate_env_var_name(name: &str) -> anyhow::Result<()> {
  if name.is_empty() || name.len() > 256 {
    bail!(
      "Invalid environment variable name '{name}': must be 1..=256 characters"
    );
  }
  let mut chars = name.chars();
  let first = chars.next().unwrap();
  if !(first.is_ascii_alphabetic() || first == '_') {
    bail!(
      "Invalid environment variable name '{name}': must start with an ASCII letter or '_'"
    );
  }
  if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
    bail!(
      "Invalid environment variable name '{name}': must contain only ASCII letters, digits, and '_'"
    );
  }
  Ok(())
}

/// Validate a Docker Compose service identifier (`[A-Za-z0-9._-]+`).
pub fn validate_compose_service_name(
  name: &str,
) -> anyhow::Result<()> {
  let trimmed = name.trim();
  if trimmed.is_empty()
    || trimmed.len() > 128
    || trimmed != name
    || name == "."
    || name == ".."
    || !name.chars().all(|c| {
      c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')
    })
  {
    bail!(
      "Invalid Compose service name '{name}': must match [A-Za-z0-9._-]+"
    );
  }
  Ok(())
}

/// Validate a Docker Compose secret identifier (`[A-Za-z0-9._-]+`, no path separators or traversal).
pub fn validate_compose_secret_name(
  name: &str,
) -> anyhow::Result<()> {
  let trimmed = name.trim();
  if trimmed.is_empty()
    || trimmed.len() > 128
    || trimmed != name
    || name == "."
    || name == ".."
    || !name.chars().all(|c| {
      c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')
    })
  {
    bail!(
      "Invalid Compose file secret name '{name}': must match [A-Za-z0-9._-]+"
    );
  }
  Ok(())
}

/// Validate a BuildKit `--secret id=<secret_id>` identifier (prevents `,` or `=` option injection and path traversal).
pub fn validate_buildkit_secret_id(id: &str) -> anyhow::Result<()> {
  let trimmed = id.trim();
  if trimmed.is_empty()
    || trimmed.len() > 128
    || trimmed != id
    || id == "."
    || id == ".."
    || !id.chars().all(|c| {
      c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')
    })
  {
    bail!(
      "Invalid BuildKit secret id '{id}': must match [A-Za-z0-9._-]+ without commas, '=', or path separators"
    );
  }
  Ok(())
}

/// Validate an optional container target path for a Compose file secret.
pub fn validate_container_secret_target_path(
  target_path: &str,
) -> anyhow::Result<()> {
  let trimmed = target_path.trim();
  if trimmed.is_empty() {
    return Ok(());
  }
  if trimmed != target_path
    || trimmed.bytes().any(|b| b < 0x20 || b == 0x7f)
    || trimmed.contains('\\')
  {
    bail!(
      "Invalid Compose file secret target_path '{target_path}': contains whitespace, control characters, or '\\'"
    );
  }
  for seg in trimmed.split('/') {
    if seg == ".." {
      bail!(
        "Invalid Compose file secret target_path '{target_path}': '..' traversal is forbidden"
      );
    }
  }
  Ok(())
}

fn ensure_no_nul_bytes(
  value: &str,
  target_desc: &str,
) -> anyhow::Result<()> {
  if value.as_bytes().contains(&0) {
    bail!(
      "Resolved external secret for {target_desc} contains a NUL (\\0) byte, which cannot be represented in environment variables"
    );
  }
  Ok(())
}

/// Escape inner string content matching Docker Compose double-quoted `.env` serialization,
/// used both for `.env` formatting and for registering escaped variants in `secret_replacers`.
pub fn escape_compose_dotenv_double_quoted_inner(
  raw: &str,
) -> String {
  let mut out = String::with_capacity(raw.len());
  for ch in raw.chars() {
    match ch {
      '\\' => out.push_str("\\\\"),
      '"' => out.push_str("\\\""),
      '$' => out.push_str("\\$"),
      '\n' => out.push_str("\\n"),
      '\r' => out.push_str("\\r"),
      '\t' => out.push_str("\\t"),
      other => out.push(other),
    }
  }
  out
}

/// Build a deduplicated, length-descending `Vec<(String, String)>` of secret replacers.
/// Never includes empty `""` strings, and includes raw, dotenv-escaped, and Compose `$$`-escaped variants
/// so multi-stage log sanitization masks both whole secrets and mixed connection URLs completely.
pub fn finalize_secret_replacers(
  raw_pairs: impl IntoIterator<Item = (String, String)>,
) -> Vec<(String, String)> {
  let mut by_needle: HashMap<String, String> = HashMap::new();

  let mut insert_variant = |needle: String, label: &str| {
    if needle.is_empty() {
      return;
    }
    by_needle.entry(needle).or_insert_with(|| label.to_string());
  };

  for (val, label) in raw_pairs {
    if val.is_empty() {
      continue;
    }
    let dotenv_esc = escape_compose_dotenv_double_quoted_inner(&val);
    let compose_dollar = val.replace('$', "$$");
    insert_variant(dotenv_esc, &label);
    insert_variant(compose_dollar, &label);
    insert_variant(val, &label);
  }

  let mut sorted: Vec<(String, String)> =
    by_needle.into_iter().collect();
  sorted.sort_by(|(a_val, a_lbl), (b_val, b_lbl)| {
    b_val
      .len()
      .cmp(&a_val.len())
      .then_with(|| a_val.cmp(b_val))
      .then_with(|| a_lbl.cmp(b_lbl))
  });
  sorted
}

/// Scan an arbitrary string field and fail closed if it contains any `[[vault...]]` reference
/// (including malformed `[[vault:...]]` or filter attempts).
pub fn ensure_no_vault_refs_in_unsupported_field(
  value: &str,
  resource_kind: &str,
  field_name: &str,
  remediation: &str,
) -> anyhow::Result<()> {
  let refs = collect_inline_vault_refs_in_str(value).map_err(|e| {
    anyhow::anyhow!(
      "External secret syntax rejected in unsupported {resource_kind} field '{field_name}': {e} ({remediation})"
    )
  })?;
  if let Some(first) = refs.first() {
    bail!(
      "External secret reference '[[{}]]' is not supported in {resource_kind} field '{field_name}'. {remediation}",
      first.canonical_token()
    );
  }
  Ok(())
}

/// Enforce unsupported-field fail-closed checks on `Stack`.
pub fn validate_stack_unsupported_vault_fields(
  stack: &Stack,
) -> anyhow::Result<()> {
  let cfg = &stack.config;
  let hint = "Place [[vault.<provider>.<key>]] in Stack 'environment' or configure structured 'secret_bindings' instead.";
  ensure_no_vault_refs_in_unsupported_field(
    &cfg.file_contents,
    "Stack",
    "file_contents",
    "Reference ${VAR} in compose.yaml and bind VAR via Stack 'environment' or 'secret_bindings' instead.",
  )?;
  ensure_no_vault_refs_in_unsupported_field(
    &cfg.pre_deploy.command,
    "Stack",
    "pre_deploy.command",
    hint,
  )?;
  ensure_no_vault_refs_in_unsupported_field(
    &cfg.pre_deploy.path,
    "Stack",
    "pre_deploy.path",
    hint,
  )?;
  ensure_no_vault_refs_in_unsupported_field(
    &cfg.post_deploy.command,
    "Stack",
    "post_deploy.command",
    hint,
  )?;
  ensure_no_vault_refs_in_unsupported_field(
    &cfg.post_deploy.path,
    "Stack",
    "post_deploy.path",
    hint,
  )?;
  ensure_no_vault_refs_in_unsupported_field(
    &cfg.compose_cmd_wrapper,
    "Stack",
    "compose_cmd_wrapper",
    hint,
  )?;
  ensure_no_vault_refs_in_unsupported_field(
    &cfg.env_file_path,
    "Stack",
    "env_file_path",
    hint,
  )?;
  for arg in &cfg.extra_args {
    ensure_no_vault_refs_in_unsupported_field(
      arg,
      "Stack",
      "extra_args",
      hint,
    )?;
  }
  for arg in &cfg.build_extra_args {
    ensure_no_vault_refs_in_unsupported_field(
      arg,
      "Stack",
      "build_extra_args",
      hint,
    )?;
  }
  Ok(())
}

/// Enforce unsupported-field fail-closed checks on `Deployment`.
pub fn validate_deployment_unsupported_vault_fields(
  deployment: &Deployment,
) -> anyhow::Result<()> {
  let cfg = &deployment.config;
  let hint = "Place [[vault.<provider>.<key>]] in Deployment 'environment' or configure structured 'secret_bindings' instead.";
  ensure_no_vault_refs_in_unsupported_field(
    &cfg.ports,
    "Deployment",
    "ports",
    hint,
  )?;
  ensure_no_vault_refs_in_unsupported_field(
    &cfg.volumes,
    "Deployment",
    "volumes",
    hint,
  )?;
  ensure_no_vault_refs_in_unsupported_field(
    &cfg.labels,
    "Deployment",
    "labels",
    hint,
  )?;
  ensure_no_vault_refs_in_unsupported_field(
    &cfg.command,
    "Deployment",
    "command",
    hint,
  )?;
  ensure_no_vault_refs_in_unsupported_field(
    &cfg.term_signal_labels,
    "Deployment",
    "term_signal_labels",
    hint,
  )?;
  for arg in &cfg.extra_args {
    ensure_no_vault_refs_in_unsupported_field(
      arg,
      "Deployment",
      "extra_args",
      hint,
    )?;
  }
  Ok(())
}

/// Enforce unsupported-field fail-closed checks on `Build` (especially rejecting `build_args` so secrets never enter image layers!).
pub fn validate_build_unsupported_vault_fields(
  build: &Build,
) -> anyhow::Result<()> {
  let cfg = &build.config;
  ensure_no_vault_refs_in_unsupported_field(
    &cfg.build_args,
    "Build",
    "build_args",
    "Docker persists ARG values in image history/layers; use Build 'secret_args' or structured BuildKit 'secret_bindings' instead.",
  )?;
  let hint = "Use Build 'secret_args' or structured BuildKit 'secret_bindings' instead.";
  ensure_no_vault_refs_in_unsupported_field(
    &cfg.labels,
    "Build",
    "labels",
    hint,
  )?;
  ensure_no_vault_refs_in_unsupported_field(
    &cfg.dockerfile,
    "Build",
    "dockerfile",
    hint,
  )?;
  ensure_no_vault_refs_in_unsupported_field(
    &cfg.pre_build.command,
    "Build",
    "pre_build.command",
    hint,
  )?;
  ensure_no_vault_refs_in_unsupported_field(
    &cfg.pre_build.path,
    "Build",
    "pre_build.path",
    hint,
  )?;
  for arg in &cfg.extra_args {
    ensure_no_vault_refs_in_unsupported_field(
      arg,
      "Build",
      "extra_args",
      hint,
    )?;
  }
  Ok(())
}

/// Enforce unsupported-resource fail-closed checks on `Repo`, `Action`, and `Alerter`.
pub fn validate_repo_no_vault_refs(
  repo: &Repo,
) -> anyhow::Result<()> {
  let hint = "External secret providers in V1 are supported on Stack, Deployment, and Build only.";
  ensure_no_vault_refs_in_unsupported_field(
    &repo.config.environment,
    "Repo",
    "environment",
    hint,
  )?;
  ensure_no_vault_refs_in_unsupported_field(
    &repo.config.on_clone.command,
    "Repo",
    "on_clone.command",
    hint,
  )?;
  ensure_no_vault_refs_in_unsupported_field(
    &repo.config.on_pull.command,
    "Repo",
    "on_pull.command",
    hint,
  )?;
  Ok(())
}

pub fn validate_action_no_vault_refs(
  action: &Action,
) -> anyhow::Result<()> {
  let hint = "External secret providers in V1 are supported on Stack, Deployment, and Build only.";
  ensure_no_vault_refs_in_unsupported_field(
    &action.config.file_contents,
    "Action",
    "file_contents",
    hint,
  )?;
  ensure_no_vault_refs_in_unsupported_field(
    &action.config.arguments,
    "Action",
    "arguments",
    hint,
  )?;
  Ok(())
}

pub fn validate_alerter_no_vault_refs(
  alerter: &Alerter,
) -> anyhow::Result<()> {
  let hint = "External secret providers in V1 are supported on Stack, Deployment, and Build only.";
  match &alerter.config.endpoint {
    AlerterEndpoint::Custom(c) => {
      ensure_no_vault_refs_in_unsupported_field(
        &c.url,
        "Alerter",
        "endpoint.url",
        hint,
      )?;
    }
    AlerterEndpoint::Slack(s) => {
      ensure_no_vault_refs_in_unsupported_field(
        &s.url,
        "Alerter",
        "endpoint.url",
        hint,
      )?;
    }
    AlerterEndpoint::Discord(d) => {
      ensure_no_vault_refs_in_unsupported_field(
        &d.url,
        "Alerter",
        "endpoint.url",
        hint,
      )?;
    }
    AlerterEndpoint::Ntfy(n) => {
      ensure_no_vault_refs_in_unsupported_field(
        &n.url,
        "Alerter",
        "endpoint.url",
        hint,
      )?;
      if let Some(email) = &n.email {
        ensure_no_vault_refs_in_unsupported_field(
          email,
          "Alerter",
          "endpoint.email",
          hint,
        )?;
      }
    }
    AlerterEndpoint::Pushover(p) => {
      ensure_no_vault_refs_in_unsupported_field(
        &p.url,
        "Alerter",
        "endpoint.url",
        hint,
      )?;
    }
  }
  Ok(())
}

/// Returns whether a Stack has any inline `[[vault...]]` references or structured `secret_bindings`.
pub fn stack_uses_external_secrets(
  stack: &Stack,
) -> anyhow::Result<bool> {
  validate_stack_unsupported_vault_fields(stack)?;
  if !stack.config.secret_bindings.is_empty() {
    return Ok(true);
  }
  let refs =
    collect_inline_vault_refs_in_str(&stack.config.environment)?;
  Ok(!refs.is_empty())
}

/// Returns whether a Deployment has any inline `[[vault...]]` references or structured `secret_bindings`.
pub fn deployment_uses_external_secrets(
  deployment: &Deployment,
) -> anyhow::Result<bool> {
  validate_deployment_unsupported_vault_fields(deployment)?;
  if !deployment.config.secret_bindings.is_empty() {
    return Ok(true);
  }
  let refs =
    collect_inline_vault_refs_in_str(&deployment.config.environment)?;
  Ok(!refs.is_empty())
}

/// Returns whether a Build has any inline `[[vault...]]` references or structured `secret_bindings`.
pub fn build_uses_external_secrets(
  build: &Build,
) -> anyhow::Result<bool> {
  validate_build_unsupported_vault_fields(build)?;
  if !build.config.secret_bindings.is_empty() {
    return Ok(true);
  }
  let refs =
    collect_inline_vault_refs_in_str(&build.config.secret_args)?;
  Ok(!refs.is_empty())
}

fn strip_matching_outer_template_quotes(template: &str) -> &str {
  if template.len() >= 2 {
    let bytes = template.as_bytes();
    let first = bytes[0];
    let last = bytes[bytes.len() - 1];
    if (first == b'"' && last == b'"')
      || (first == b'\'' && last == b'\'')
    {
      return &template[1..template.len() - 1];
    }
  }
  template
}

/// Single-pass segment substitution on a single value string:
/// - Segments outside `[[...]]` are emitted verbatim.
/// - `[[vault.<slug>.<key>]]` tokens are replaced literally byte-for-byte from `inline_values`
///   and are NEVER passed through Komodo's `Interpolator` or re-scanned.
/// - Other `[[...]]` tokens (Komodo `Variable` / local `Secret`) are passed individually to `interpolator`.
pub fn substitute_value_single_pass(
  template: &str,
  inline_values: &HashMap<ExternalSecretRef, ResolvedSecretValue>,
  interpolator: &mut Interpolator<'_>,
) -> anyhow::Result<(String, Vec<ExternalSecretRef>)> {
  let bytes = template.as_bytes();
  let mut out = String::with_capacity(template.len());
  let mut used_refs = Vec::new();
  let mut i = 0;
  let mut literal_start = 0;

  while i + 1 < bytes.len() {
    if bytes[i] == b'[' && bytes[i + 1] == b'[' {
      let inner_start = i + 2;
      let mut j = inner_start;
      let mut found_close = None;
      while j + 1 < bytes.len() {
        if bytes[j] == b']' && bytes[j + 1] == b']' {
          found_close = Some(j);
          break;
        }
        if bytes[j] == b'[' && bytes[j + 1] == b'[' {
          break;
        }
        j += 1;
      }

      if let Some(close_idx) = found_close {
        if literal_start < i {
          out.push_str(&template[literal_start..i]);
        }
        let inner = &template[inner_start..close_idx];
        if let Some(ext_ref) =
          ExternalSecretRef::parse_bracket_token(inner)?
        {
          let resolved = inline_values.get(&ext_ref).ok_or_else(|| {
            anyhow::anyhow!(
              "Internal error: missing resolved snapshot value for [[{}]]",
              ext_ref.canonical_token()
            )
          })?;
          let text = resolved.as_text()?;
          out.push_str(text);
          used_refs.push(ext_ref);
        } else {
          let mut local_token =
            template[i..close_idx + 2].to_string();
          interpolator.interpolate_string(&mut local_token)?;
          out.push_str(&local_token);
        }
        i = close_idx + 2;
        literal_start = i;
        continue;
      }
    }
    i += 1;
  }

  if literal_start < template.len() {
    out.push_str(&template[literal_start..]);
  }

  Ok((out, used_refs))
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BatchSelectorGroupKey {
  provider_id: String,
  selector: SecretSelector,
}

#[derive(Debug, Clone)]
struct MergedKeyRequirement {
  format: SecretValueFormat,
  json_field: String,
  required: bool,
  allow_empty: bool,
  usages: BTreeSet<SecretUsage>,
}

/// Resolved external service-scoped environment variable for a Stack.
#[derive(Clone, PartialEq, Eq)]
pub struct ResolvedStackServiceEnvBinding {
  pub variable: String,
  pub value: String,
  pub services: Vec<String>,
}

impl std::fmt::Debug for ResolvedStackServiceEnvBinding {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ResolvedStackServiceEnvBinding")
      .field("variable", &self.variable)
      .field("value", &"[REDACTED]")
      .field("services", &self.services)
      .finish()
  }
}

/// Resolved external Compose file secret for a Stack.
#[derive(Clone, PartialEq, Eq)]
pub struct ResolvedStackFileSecretBinding {
  pub secret_name: String,
  pub bytes: Vec<u8>,
  pub services: Vec<String>,
  pub target_path: String,
  pub uid: String,
  pub gid: String,
  pub mode: Option<u32>,
}

impl std::fmt::Debug for ResolvedStackFileSecretBinding {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ResolvedStackFileSecretBinding")
      .field("secret_name", &self.secret_name)
      .field("bytes_len", &self.bytes.len())
      .field("services", &self.services)
      .field("target_path", &self.target_path)
      .finish()
  }
}

/// Frozen, single-execution resolved secret bundle for a `Stack`.
/// Never serialized to MongoDB, Update history, or ResourceSync!
#[derive(Debug, Clone, Default)]
pub struct ResolvedStackSecrets {
  /// Cleaned `stack.config.environment` containing ONLY non-external variables
  /// (already interpolated for Komodo variables & local secrets).
  pub remaining_standard_environment: String,
  /// External-secret-bearing `.env` variables (both inline `[[vault...]]` and `ComposeEnv` bindings),
  /// passed separately to Periphery so Periphery's Stage-2 `Interpolator` and `parse_key_value_list`
  /// never re-scan or truncate remote secret values.
  pub external_compose_env: Vec<EnvironmentVar>,
  /// Direct service-scoped environment variables (`ServiceEnv` bindings).
  pub service_env_bindings: Vec<ResolvedStackServiceEnvBinding>,
  /// Controlled host file secrets (`ComposeFileSecret` bindings).
  pub file_secret_bindings: Vec<ResolvedStackFileSecretBinding>,
  /// All environment variable names carrying external secrets (for structured `Inspect` redaction).
  pub secret_env_keys: Vec<String>,
  /// All Compose file secret names mounted in this deployment.
  pub file_secret_names: Vec<String>,
  /// Length-descending `(to_replace, label)` replacers covering raw secrets, full mixed values, and escaped forms.
  pub replacers: Vec<(String, String)>,
  /// Non-sensitive provenance records.
  pub provenance: Vec<DeployedSecretProvenance>,
  /// Keyed HMAC-SHA256 hex digest + key_id for `DeployStackIfChanged`.
  pub keyed_fingerprint_hmac: String,
  pub fingerprint_key_id: String,
}

/// Frozen, single-execution resolved secret bundle for a `Deployment`.
#[derive(Debug, Clone, Default)]
pub struct ResolvedDeploymentSecrets {
  /// Non-external `EnvironmentVar`s (interpolated for Komodo variables & local secrets).
  pub standard_env_vars: Vec<EnvironmentVar>,
  /// External-secret-bearing `EnvironmentVar`s (passed via dedicated field so Periphery skips Stage-2 interpolation).
  pub external_env_vars: Vec<EnvironmentVar>,
  pub secret_env_keys: Vec<String>,
  pub replacers: Vec<(String, String)>,
  pub provenance: Vec<DeployedSecretProvenance>,
  pub keyed_fingerprint_hmac: String,
  pub fingerprint_key_id: String,
}

/// Frozen, single-execution resolved secret bundle for a `Build`.
#[derive(Debug, Clone, Default)]
pub struct ResolvedBuildSecrets {
  /// Non-external `secret_args` `EnvironmentVar`s.
  pub standard_secret_args: Vec<EnvironmentVar>,
  /// External BuildKit `--secret` values (`id` -> literal secret bytes/string).
  pub external_build_secrets: Vec<EnvironmentVar>,
  pub replacers: Vec<(String, String)>,
  pub provenance: Vec<DeployedSecretProvenance>,
}

struct BatchFetchEngine {
  providers_by_slug: HashMap<String, SecretProvider>,
  groups: IndexMap<
    BatchSelectorGroupKey,
    IndexMap<String, MergedKeyRequirement>,
  >,
}

impl BatchFetchEngine {
  fn new() -> Self {
    Self {
      providers_by_slug: HashMap::new(),
      groups: IndexMap::new(),
    }
  }

  async fn load_enabled_provider(
    &mut self,
    slug: &str,
  ) -> anyhow::Result<SecretProvider> {
    if let Some(p) = self.providers_by_slug.get(slug) {
      return Ok(p.clone());
    }
    let provider = find_secret_provider(slug).await?;
    if !provider.enabled {
      bail!(
        "Secret provider '{}' ({}) is disabled",
        provider.name,
        provider.slug
      );
    }
    if !provider.has_credentials {
      bail!(
        "Secret provider '{}' ({}) has no configured credentials",
        provider.name,
        provider.slug
      );
    }
    self
      .providers_by_slug
      .insert(provider.slug.clone(), provider.clone());
    Ok(provider)
  }

  fn resolve_source<'a>(
    sources: &'a [SecretSource],
    source_or_slug: &'a str,
  ) -> anyhow::Result<(&'a str, Option<&'a SecretSelector>)> {
    let trimmed = source_or_slug.trim();
    if trimmed.is_empty() {
      bail!("Secret binding 'source' cannot be empty");
    }
    // Match explicit named SecretSource first, then provider slug fallback on SecretSource
    if let Some(src) = sources.iter().find(|s| {
      (!s.name.trim().is_empty() && s.name.trim() == trimmed)
        || (s.name.trim().is_empty() && s.provider.trim() == trimmed)
    }) {
      return Ok((src.provider.trim(), Some(&src.selector)));
    }
    Ok((trimmed, None))
  }

  #[allow(clippy::too_many_arguments)]
  async fn register_request(
    &mut self,
    sources: &[SecretSource],
    source_or_slug: &str,
    remote_key: &str,
    format: SecretValueFormat,
    json_field: &str,
    required: bool,
    allow_empty: bool,
    usage: SecretUsage,
  ) -> anyhow::Result<(SecretProvider, SecretSelector)> {
    let (provider_slug, explicit_sel) =
      Self::resolve_source(sources, source_or_slug)?;
    let provider = self.load_enabled_provider(provider_slug).await?;

    let normalized_selector = resolve_selector_for_provider(
      &provider,
      sources,
      source_or_slug,
      explicit_sel,
    )?;
    let adapter = get_adapter((&provider.config).into());
    adapter
      .validate_selector(&provider.config, &normalized_selector)?;

    let group_key = BatchSelectorGroupKey {
      provider_id: provider.id.clone(),
      selector: normalized_selector.clone(),
    };
    let items = self.groups.entry(group_key).or_default();
    if let Some(existing) = items.get_mut(remote_key) {
      if existing.format != format {
        bail!(
          "Conflicting secret formats requested for key '{}' on provider '{}'",
          remote_key,
          provider.slug
        );
      }
      if existing.json_field != json_field {
        bail!(
          "Conflicting json_field selectors requested for key '{}' on provider '{}'",
          remote_key,
          provider.slug
        );
      }
      existing.required |= required;
      existing.allow_empty &= allow_empty;
      existing.usages.insert(usage);
    } else {
      let mut usages = BTreeSet::new();
      usages.insert(usage);
      items.insert(
        remote_key.to_string(),
        MergedKeyRequirement {
          format,
          json_field: json_field.to_string(),
          required,
          allow_empty,
          usages,
        },
      );
    }

    Ok((provider, normalized_selector))
  }

  /// Authorize every `(provider, selector, usage)` against live DB grants, decrypt each provider's
  /// bootstrap credential once, and resolve each `(provider, selector)` batch exactly once!
  async fn execute_all(
    &self,
    user: &User,
    override_resource_level: Option<PermissionLevel>,
    resource_target: &ResourceTarget,
    cancel: &CancellationToken,
  ) -> anyhow::Result<
    HashMap<
      (String, SecretSelector, String),
      Option<ResolvedSecretValue>,
    >,
  > {
    let providers_by_id: HashMap<&str, &SecretProvider> = self
      .providers_by_slug
      .values()
      .map(|p| (p.id.as_str(), p))
      .collect();

    // 1. Live authorization check across every group & usage BEFORE decrypting credentials or making HTTP calls!
    for (group_key, items) in &self.groups {
      let provider = providers_by_id
        .get(group_key.provider_id.as_str())
        .copied()
        .context("Missing provider in batch engine")?;
      let all_usages: BTreeSet<SecretUsage> = items
        .values()
        .flat_map(|req| req.usages.iter().copied())
        .collect();
      for usage in all_usages {
        authorize_secret_access(
          user,
          &provider.id,
          resource_target,
          &group_key.selector,
          usage,
          override_resource_level,
        )
        .await?;
      }
    }

    // 2. Decrypt bootstrap credential once per distinct provider
    let mut credentials_by_provider_id = HashMap::new();
    for provider in self.providers_by_slug.values() {
      let cred = load_provider_credential(provider).await?;
      credentials_by_provider_id.insert(provider.id.clone(), cred);
    }

    // 3. Execute one deduplicated `resolve_batch` per `(provider_id, selector)`
    let mut results = HashMap::new();
    for (group_key, items) in &self.groups {
      let provider = providers_by_id[group_key.provider_id.as_str()];
      let cred = &credentials_by_provider_id[&provider.id];
      let adapter = get_adapter((&provider.config).into());

      let batch_reqs: Vec<AdapterResolveItemRequest> = items
        .iter()
        .map(|(remote_key, req)| AdapterResolveItemRequest {
          remote_key: remote_key.clone(),
          format: req.format,
          json_field: req.json_field.clone(),
          required: req.required,
          allow_empty: req.allow_empty,
        })
        .collect();

      let resolved_map = adapter
        .resolve_batch(
          provider,
          cred,
          &group_key.selector,
          &batch_reqs,
          cancel,
        )
        .await?;

      for (remote_key, maybe_val) in resolved_map {
        results.insert(
          (
            group_key.provider_id.clone(),
            group_key.selector.clone(),
            remote_key,
          ),
          maybe_val,
        );
      }
    }

    Ok(results)
  }
}

/// Resolve all inline `[[vault.<slug>.<key>]]` references and structured `StackSecretBinding`s
/// for a Stack execution in a single deduplicated, authorized preflight pass.
pub async fn resolve_stack_secrets(
  stack: &Stack,
  user: &User,
  override_resource_level: Option<PermissionLevel>,
  is_swarm: bool,
  interpolator: &mut Interpolator<'_>,
  cancel: &CancellationToken,
) -> anyhow::Result<Option<ResolvedStackSecrets>> {
  if !stack_uses_external_secrets(stack)? {
    return Ok(None);
  }

  if stack.config.skip_secret_interp {
    bail!(
      "Stack '{}' references external secrets, which is incompatible with 'skip_secret_interp = true' (refusing to write unresolved [[vault...]] references or skip secret injection)",
      stack.name
    );
  }
  if is_swarm {
    bail!(
      "Stack '{}' targets a Swarm cluster, which is not supported for external Secret Providers in V1 (Docker Compose on a Server is required)",
      stack.name
    );
  }
  if !stack.config.compose_cmd_wrapper.trim().is_empty() {
    bail!(
      "Stack '{}' configures both native external secrets and 'compose_cmd_wrapper'; remove 'compose_cmd_wrapper' or migrate wrapper secrets to native Secret Providers to avoid ambiguous secret precedence",
      stack.name
    );
  }

  let keyring = SecretProviderKeyring::from_core_config()?;
  let parsed_env_lines =
    parse_key_value_list(&stack.config.environment).context(
      "Failed to parse Stack 'environment' key-value definitions",
    )?;

  // Verify duplicate keys inside `stack.config.environment` when external secrets are involved
  let mut engine = BatchFetchEngine::new();
  let mut inline_lookup_keys: HashMap<
    ExternalSecretRef,
    (String, SecretSelector),
  > = HashMap::new();

  for (kv_var, kv_val) in &parsed_env_lines {
    let refs = collect_inline_vault_refs_in_str(kv_val)?;
    if !refs.is_empty() {
      validate_env_var_name(kv_var)?;
      for ext_ref in refs {
        let (prov, sel) = engine
          .register_request(
            &stack.config.secret_sources,
            &ext_ref.provider_slug,
            &ext_ref.secret_key,
            SecretValueFormat::Text,
            "",
            true,
            true,
            SecretUsage::StackEnv,
          )
          .await?;
        inline_lookup_keys.insert(ext_ref, (prov.id, sel));
      }
    }
  }

  // Validate & register structured StackSecretBindings
  let mut binding_selectors: Vec<(SecretProvider, SecretSelector)> =
    Vec::with_capacity(stack.config.secret_bindings.len());
  for binding in &stack.config.secret_bindings {
    let usage = match &binding.target {
      StackSecretTarget::ComposeEnv { variable } => {
        validate_env_var_name(variable)?;
        if binding.format != SecretValueFormat::Text {
          bail!(
            "Stack ComposeEnv binding for variable '{variable}' must use format=Text"
          );
        }
        SecretUsage::StackEnv
      }
      StackSecretTarget::ServiceEnv { variable, services } => {
        validate_env_var_name(variable)?;
        if services.is_empty() {
          bail!(
            "Stack ServiceEnv binding for variable '{variable}' must specify at least one target Compose service"
          );
        }
        for svc in services {
          validate_compose_service_name(svc)?;
        }
        if binding.format != SecretValueFormat::Text {
          bail!(
            "Stack ServiceEnv binding for variable '{variable}' must use format=Text"
          );
        }
        SecretUsage::StackEnv
      }
      StackSecretTarget::ComposeFileSecret {
        secret_name,
        services,
        target_path,
        mode,
        ..
      } => {
        validate_compose_secret_name(secret_name)?;
        if services.is_empty() {
          bail!(
            "Stack ComposeFileSecret binding '{secret_name}' must specify at least one target Compose service"
          );
        }
        for svc in services {
          validate_compose_service_name(svc)?;
        }
        validate_container_secret_target_path(target_path)?;
        if let Some(m) = mode
          && *m > 0o777
        {
          bail!(
            "Invalid file mode {m:#o} on ComposeFileSecret '{secret_name}' (expected <= 0o777)"
          );
        }
        SecretUsage::StackComposeFileSecret
      }
    };

    let (prov, sel) = engine
      .register_request(
        &stack.config.secret_sources,
        &binding.source,
        &binding.remote_key,
        binding.format,
        &binding.json_field,
        binding.required,
        binding.allow_empty,
        usage,
      )
      .await?;
    binding_selectors.push((prov, sel));
  }

  let resource_target = ResourceTarget::Stack(stack.id.clone());
  let batch_values = engine
    .execute_all(
      user,
      override_resource_level,
      &resource_target,
      cancel,
    )
    .await?;

  let mut inline_values: HashMap<
    ExternalSecretRef,
    ResolvedSecretValue,
  > = HashMap::new();
  for (ext_ref, (prov_id, sel)) in &inline_lookup_keys {
    let val = batch_values
      .get(&(
        prov_id.clone(),
        sel.clone(),
        ext_ref.secret_key.clone(),
      ))
      .and_then(|v| v.clone())
      .ok_or_else(|| {
        anyhow::anyhow!(
          "Missing resolved value for inline reference [[{}]]",
          ext_ref.canonical_token()
        )
      })?;
    inline_values.insert(ext_ref.clone(), val);
  }

  let mut standard_env_vars: IndexMap<String, String> =
    IndexMap::new();
  let mut external_compose_map: IndexMap<String, String> =
    IndexMap::new();
  let mut secret_env_key_set: BTreeSet<String> = BTreeSet::new();
  let mut raw_replacers: Vec<(String, String)> = Vec::new();
  let mut provenance: Vec<DeployedSecretProvenance> = Vec::new();

  // Register every individual resolved inline secret in replacers
  for (ext_ref, val) in &inline_values {
    if let Ok(txt) = val.as_text() {
      raw_replacers
        .push((txt.to_string(), ext_ref.canonical_token()));
    }
  }

  for (kv_var, kv_val) in parsed_env_lines {
    let refs = collect_inline_vault_refs_in_str(&kv_val)?;
    if refs.is_empty() {
      let mut interpolated_val = kv_val;
      interpolator.interpolate_string(&mut interpolated_val)?;
      standard_env_vars.insert(kv_var, interpolated_val);
    } else {
      let unquoted_tpl =
        strip_matching_outer_template_quotes(&kv_val);
      let (final_val, used_refs) = substitute_value_single_pass(
        unquoted_tpl,
        &inline_values,
        interpolator,
      )?;
      ensure_no_nul_bytes(
        &final_val,
        &format!("Stack environment variable '{kv_var}'"),
      )?;
      // Mask the ENTIRE resolved env value (including mixed connection URLs) as well as individual secrets!
      raw_replacers
        .push((final_val.clone(), format!("env:{kv_var}")));
      secret_env_key_set.insert(kv_var.clone());
      for used in used_refs {
        let ver = inline_values.get(&used).and_then(|v| v.version);
        provenance.push(DeployedSecretProvenance {
          provider_slug: used.provider_slug,
          remote_key: used.secret_key,
          target: format!("env:{kv_var}"),
          version: ver,
        });
      }
      standard_env_vars.shift_remove(&kv_var);
      external_compose_map.insert(kv_var, final_val);
    }
  }

  let mut service_env_map: IndexMap<(String, String), String> =
    IndexMap::new();
  let mut file_secrets_map: IndexMap<
    String,
    ResolvedStackFileSecretBinding,
  > = IndexMap::new();

  for (binding, (prov, sel)) in stack
    .config
    .secret_bindings
    .iter()
    .zip(binding_selectors.into_iter())
  {
    let maybe_val = batch_values
      .get(&(prov.id.clone(), sel, binding.remote_key.clone()))
      .and_then(|v| v.clone());
    let Some(resolved) = maybe_val else {
      // Optional (`required = false`) and missing -> skip cleanly without error.
      continue;
    };

    if let Ok(txt) = resolved.as_text() {
      raw_replacers.push((
        txt.to_string(),
        format!("vault.{}.{}", prov.slug, binding.remote_key),
      ));
    }

    match &binding.target {
      StackSecretTarget::ComposeEnv { variable } => {
        let txt = resolved.as_text()?.to_string();
        ensure_no_nul_bytes(
          &txt,
          &format!("Stack ComposeEnv binding '{variable}'"),
        )?;
        let already_exists = standard_env_vars.contains_key(variable)
          || external_compose_map.contains_key(variable);
        if already_exists
          && binding.conflict_policy == SecretConflictPolicy::Fail
        {
          bail!(
            "Conflict on Stack Compose environment variable '{variable}': already defined and binding conflict_policy=Fail"
          );
        }
        standard_env_vars.shift_remove(variable);
        raw_replacers.push((txt.clone(), format!("env:{variable}")));
        secret_env_key_set.insert(variable.clone());
        external_compose_map.insert(variable.clone(), txt);
        provenance.push(DeployedSecretProvenance {
          provider_slug: prov.slug,
          remote_key: binding.remote_key.clone(),
          target: format!("compose_env:{variable}"),
          version: resolved.version,
        });
      }
      StackSecretTarget::ServiceEnv { variable, services } => {
        let txt = resolved.as_text()?.to_string();
        ensure_no_nul_bytes(
          &txt,
          &format!("Stack ServiceEnv binding '{variable}'"),
        )?;
        for svc in services {
          let pair = (svc.clone(), variable.clone());
          if service_env_map.contains_key(&pair)
            && binding.conflict_policy == SecretConflictPolicy::Fail
          {
            bail!(
              "Duplicate ServiceEnv binding for variable '{variable}' on service '{svc}' with conflict_policy=Fail"
            );
          }
          service_env_map.insert(pair, txt.clone());
        }
        raw_replacers.push((txt, format!("service_env:{variable}")));
        secret_env_key_set.insert(variable.clone());
        provenance.push(DeployedSecretProvenance {
          provider_slug: prov.slug,
          remote_key: binding.remote_key.clone(),
          target: format!(
            "service_env:{}:{variable}",
            services.join(",")
          ),
          version: resolved.version,
        });
      }
      StackSecretTarget::ComposeFileSecret {
        secret_name,
        services,
        target_path,
        uid,
        gid,
        mode,
      } => {
        if file_secrets_map.contains_key(secret_name)
          && binding.conflict_policy == SecretConflictPolicy::Fail
        {
          bail!(
            "Duplicate ComposeFileSecret binding '{secret_name}' with conflict_policy=Fail"
          );
        }
        file_secrets_map.insert(
          secret_name.clone(),
          ResolvedStackFileSecretBinding {
            secret_name: secret_name.clone(),
            bytes: resolved.bytes.clone(),
            services: services.clone(),
            target_path: target_path.clone(),
            uid: uid.clone(),
            gid: gid.clone(),
            mode: *mode,
          },
        );
        provenance.push(DeployedSecretProvenance {
          provider_slug: prov.slug,
          remote_key: binding.remote_key.clone(),
          target: format!("file_secret:{secret_name}"),
          version: resolved.version,
        });
      }
    }
  }

  // Reconstruct grouped `ResolvedStackServiceEnvBinding`s deterministically
  let mut grouped_service_envs: IndexMap<
    (String, String),
    Vec<String>,
  > = IndexMap::new();
  for ((svc, var), val) in service_env_map {
    grouped_service_envs
      .entry((var, val))
      .or_default()
      .push(svc);
  }
  let service_env_bindings: Vec<ResolvedStackServiceEnvBinding> =
    grouped_service_envs
      .into_iter()
      .map(|((variable, value), services)| {
        ResolvedStackServiceEnvBinding {
          variable,
          value,
          services,
        }
      })
      .collect();

  let file_secret_names: Vec<String> =
    file_secrets_map.keys().cloned().collect();
  let file_secret_bindings: Vec<ResolvedStackFileSecretBinding> =
    file_secrets_map.into_values().collect();

  let external_compose_env: Vec<EnvironmentVar> =
    external_compose_map
      .into_iter()
      .map(|(variable, value)| EnvironmentVar { variable, value })
      .collect();

  let remaining_standard_environment = standard_env_vars
    .iter()
    .map(|(k, v)| format!("{k}={v}"))
    .collect::<Vec<_>>()
    .join("\n");

  // Compute deterministic keyed HMAC-SHA256 fingerprint across all resolved external targets & bytes
  let mut fp_entries: BTreeMap<String, Vec<u8>> = BTreeMap::new();
  for ev in &external_compose_env {
    fp_entries.insert(
      format!("compose_env:{}", ev.variable),
      ev.value.as_bytes().to_vec(),
    );
  }
  for sev in &service_env_bindings {
    for svc in &sev.services {
      fp_entries.insert(
        format!("service_env:{svc}:{}", sev.variable),
        sev.value.as_bytes().to_vec(),
      );
    }
  }
  for fs in &file_secret_bindings {
    let meta_prefix = format!(
      "services={};target={};uid={};gid={};mode={:?}\n",
      fs.services.join(","),
      fs.target_path,
      fs.uid,
      fs.gid,
      fs.mode
    );
    let mut payload = meta_prefix.into_bytes();
    payload.extend_from_slice(&fs.bytes);
    fp_entries
      .insert(format!("file_secret:{}", fs.secret_name), payload);
  }
  let fp_vec: Vec<(String, Vec<u8>)> =
    fp_entries.into_iter().collect();
  let (keyed_fingerprint_hmac, fingerprint_key_id) = keyring
    .compute_deployment_fingerprint("Stack", &stack.id, &fp_vec);

  let replacers = finalize_secret_replacers(raw_replacers);

  Ok(Some(ResolvedStackSecrets {
    remaining_standard_environment,
    external_compose_env,
    service_env_bindings,
    file_secret_bindings,
    secret_env_keys: secret_env_key_set.into_iter().collect(),
    file_secret_names,
    replacers,
    provenance,
    keyed_fingerprint_hmac,
    fingerprint_key_id,
  }))
}

/// Resolve all inline `[[vault.<slug>.<key>]]` references and structured `DeploymentSecretBinding`s
/// for a Deployment execution.
pub async fn resolve_deployment_secrets(
  deployment: &Deployment,
  user: &User,
  override_resource_level: Option<PermissionLevel>,
  is_swarm: bool,
  interpolator: &mut Interpolator<'_>,
  cancel: &CancellationToken,
) -> anyhow::Result<Option<ResolvedDeploymentSecrets>> {
  if !deployment_uses_external_secrets(deployment)? {
    return Ok(None);
  }
  if deployment.config.skip_secret_interp {
    bail!(
      "Deployment '{}' references external secrets, which is incompatible with 'skip_secret_interp = true'",
      deployment.name
    );
  }
  if is_swarm {
    bail!(
      "Deployment '{}' targets a Swarm cluster, which is not supported for external Secret Providers in V1 (Server container mode is required)",
      deployment.name
    );
  }

  let keyring = SecretProviderKeyring::from_core_config()?;
  let parsed_env_lines = parse_key_value_list(
    &deployment.config.environment,
  )
  .context(
    "Failed to parse Deployment 'environment' key-value definitions",
  )?;

  let mut engine = BatchFetchEngine::new();
  let mut inline_lookup_keys: HashMap<
    ExternalSecretRef,
    (String, SecretSelector),
  > = HashMap::new();

  for (kv_var, kv_val) in &parsed_env_lines {
    let refs = collect_inline_vault_refs_in_str(kv_val)?;
    if !refs.is_empty() {
      validate_env_var_name(kv_var)?;
      for ext_ref in refs {
        let (prov, sel) = engine
          .register_request(
            &deployment.config.secret_sources,
            &ext_ref.provider_slug,
            &ext_ref.secret_key,
            SecretValueFormat::Text,
            "",
            true,
            true,
            SecretUsage::DeploymentEnv,
          )
          .await?;
        inline_lookup_keys.insert(ext_ref, (prov.id, sel));
      }
    }
  }

  let mut binding_selectors: Vec<(SecretProvider, SecretSelector)> =
    Vec::with_capacity(deployment.config.secret_bindings.len());
  for binding in &deployment.config.secret_bindings {
    validate_env_var_name(&binding.variable)?;
    let (prov, sel) = engine
      .register_request(
        &deployment.config.secret_sources,
        &binding.source,
        &binding.remote_key,
        SecretValueFormat::Text,
        &binding.json_field,
        binding.required,
        binding.allow_empty,
        SecretUsage::DeploymentEnv,
      )
      .await?;
    binding_selectors.push((prov, sel));
  }

  let resource_target =
    ResourceTarget::Deployment(deployment.id.clone());
  let batch_values = engine
    .execute_all(
      user,
      override_resource_level,
      &resource_target,
      cancel,
    )
    .await?;

  let mut inline_values: HashMap<
    ExternalSecretRef,
    ResolvedSecretValue,
  > = HashMap::new();
  for (ext_ref, (prov_id, sel)) in &inline_lookup_keys {
    let val = batch_values
      .get(&(
        prov_id.clone(),
        sel.clone(),
        ext_ref.secret_key.clone(),
      ))
      .and_then(|v| v.clone())
      .ok_or_else(|| {
        anyhow::anyhow!(
          "Missing resolved value for inline reference [[{}]]",
          ext_ref.canonical_token()
        )
      })?;
    inline_values.insert(ext_ref.clone(), val);
  }

  let mut standard_map: IndexMap<String, String> = IndexMap::new();
  let mut external_map: IndexMap<String, String> = IndexMap::new();
  let mut secret_env_key_set: BTreeSet<String> = BTreeSet::new();
  let mut raw_replacers: Vec<(String, String)> = Vec::new();
  let mut provenance: Vec<DeployedSecretProvenance> = Vec::new();

  for (ext_ref, val) in &inline_values {
    if let Ok(txt) = val.as_text() {
      raw_replacers
        .push((txt.to_string(), ext_ref.canonical_token()));
    }
  }

  for (kv_var, kv_val) in parsed_env_lines {
    let refs = collect_inline_vault_refs_in_str(&kv_val)?;
    if refs.is_empty() {
      let mut interpolated_val = kv_val;
      interpolator.interpolate_string(&mut interpolated_val)?;
      standard_map.insert(kv_var, interpolated_val);
    } else {
      let unquoted_tpl =
        strip_matching_outer_template_quotes(&kv_val);
      let (final_val, used_refs) = substitute_value_single_pass(
        unquoted_tpl,
        &inline_values,
        interpolator,
      )?;
      ensure_no_nul_bytes(
        &final_val,
        &format!("Deployment environment variable '{kv_var}'"),
      )?;
      raw_replacers
        .push((final_val.clone(), format!("env:{kv_var}")));
      secret_env_key_set.insert(kv_var.clone());
      for used in used_refs {
        let ver = inline_values.get(&used).and_then(|v| v.version);
        provenance.push(DeployedSecretProvenance {
          provider_slug: used.provider_slug,
          remote_key: used.secret_key,
          target: format!("env:{kv_var}"),
          version: ver,
        });
      }
      standard_map.shift_remove(&kv_var);
      external_map.insert(kv_var, final_val);
    }
  }

  for (binding, (prov, sel)) in deployment
    .config
    .secret_bindings
    .iter()
    .zip(binding_selectors.into_iter())
  {
    let Some(resolved) = batch_values
      .get(&(prov.id.clone(), sel, binding.remote_key.clone()))
      .and_then(|v| v.clone())
    else {
      continue;
    };
    let txt = resolved.as_text()?.to_string();
    ensure_no_nul_bytes(
      &txt,
      &format!("Deployment secret binding '{}'", binding.variable),
    )?;
    if (standard_map.contains_key(&binding.variable)
      || external_map.contains_key(&binding.variable))
      && binding.conflict_policy == SecretConflictPolicy::Fail
    {
      bail!(
        "Conflict on Deployment environment variable '{}': already defined and binding conflict_policy=Fail",
        binding.variable
      );
    }
    standard_map.shift_remove(&binding.variable);
    raw_replacers.push((
      txt.clone(),
      format!("vault.{}.{}", prov.slug, binding.remote_key),
    ));
    raw_replacers
      .push((txt.clone(), format!("env:{}", binding.variable)));
    secret_env_key_set.insert(binding.variable.clone());
    external_map.insert(binding.variable.clone(), txt);
    provenance.push(DeployedSecretProvenance {
      provider_slug: prov.slug,
      remote_key: binding.remote_key.clone(),
      target: format!("env:{}", binding.variable),
      version: resolved.version,
    });
  }

  let mut fp_entries: BTreeMap<String, Vec<u8>> = BTreeMap::new();
  for (k, v) in &external_map {
    fp_entries.insert(format!("env:{k}"), v.as_bytes().to_vec());
  }
  let fp_vec: Vec<(String, Vec<u8>)> =
    fp_entries.into_iter().collect();
  let (keyed_fingerprint_hmac, fingerprint_key_id) = keyring
    .compute_deployment_fingerprint(
      "Deployment",
      &deployment.id,
      &fp_vec,
    );

  Ok(Some(ResolvedDeploymentSecrets {
    standard_env_vars: standard_map
      .into_iter()
      .map(|(variable, value)| EnvironmentVar { variable, value })
      .collect(),
    external_env_vars: external_map
      .into_iter()
      .map(|(variable, value)| EnvironmentVar { variable, value })
      .collect(),
    secret_env_keys: secret_env_key_set.into_iter().collect(),
    replacers: finalize_secret_replacers(raw_replacers),
    provenance,
    keyed_fingerprint_hmac,
    fingerprint_key_id,
  }))
}

/// Resolve all inline `[[vault.<slug>.<key>]]` references in `build.config.secret_args`
/// and structured `BuildSecretBinding`s for a Build execution.
pub async fn resolve_build_secrets(
  build: &Build,
  user: &User,
  override_resource_level: Option<PermissionLevel>,
  interpolator: &mut Interpolator<'_>,
  cancel: &CancellationToken,
) -> anyhow::Result<Option<ResolvedBuildSecrets>> {
  if !build_uses_external_secrets(build)? {
    return Ok(None);
  }
  if build.config.skip_secret_interp {
    bail!(
      "Build '{}' references external secrets, which is incompatible with 'skip_secret_interp = true'",
      build.name
    );
  }

  let parsed_secret_args =
    parse_key_value_list(&build.config.secret_args).context(
      "Failed to parse Build 'secret_args' key-value definitions",
    )?;

  let mut engine = BatchFetchEngine::new();
  let mut inline_lookup_keys: HashMap<
    ExternalSecretRef,
    (String, SecretSelector),
  > = HashMap::new();

  for (kv_var, kv_val) in &parsed_secret_args {
    let refs = collect_inline_vault_refs_in_str(kv_val)?;
    if !refs.is_empty() {
      validate_buildkit_secret_id(kv_var)?;
      for ext_ref in refs {
        let (prov, sel) = engine
          .register_request(
            &build.config.secret_sources,
            &ext_ref.provider_slug,
            &ext_ref.secret_key,
            SecretValueFormat::Text,
            "",
            true,
            true,
            SecretUsage::BuildSecret,
          )
          .await?;
        inline_lookup_keys.insert(ext_ref, (prov.id, sel));
      }
    }
  }

  let mut binding_selectors: Vec<(SecretProvider, SecretSelector)> =
    Vec::with_capacity(build.config.secret_bindings.len());
  for binding in &build.config.secret_bindings {
    validate_buildkit_secret_id(&binding.secret_id)?;
    let (prov, sel) = engine
      .register_request(
        &build.config.secret_sources,
        &binding.source,
        &binding.remote_key,
        binding.format,
        &binding.json_field,
        binding.required,
        binding.allow_empty,
        SecretUsage::BuildSecret,
      )
      .await?;
    binding_selectors.push((prov, sel));
  }

  let resource_target = ResourceTarget::Build(build.id.clone());
  let batch_values = engine
    .execute_all(
      user,
      override_resource_level,
      &resource_target,
      cancel,
    )
    .await?;

  let mut inline_values: HashMap<
    ExternalSecretRef,
    ResolvedSecretValue,
  > = HashMap::new();
  for (ext_ref, (prov_id, sel)) in &inline_lookup_keys {
    let val = batch_values
      .get(&(
        prov_id.clone(),
        sel.clone(),
        ext_ref.secret_key.clone(),
      ))
      .and_then(|v| v.clone())
      .ok_or_else(|| {
        anyhow::anyhow!(
          "Missing resolved value for inline reference [[{}]]",
          ext_ref.canonical_token()
        )
      })?;
    inline_values.insert(ext_ref.clone(), val);
  }

  let mut standard_map: IndexMap<String, String> = IndexMap::new();
  let mut external_map: IndexMap<String, String> = IndexMap::new();
  let mut raw_replacers: Vec<(String, String)> = Vec::new();
  let mut provenance: Vec<DeployedSecretProvenance> = Vec::new();

  for (ext_ref, val) in &inline_values {
    if let Ok(txt) = val.as_text() {
      raw_replacers
        .push((txt.to_string(), ext_ref.canonical_token()));
    }
  }

  for (kv_var, kv_val) in parsed_secret_args {
    let refs = collect_inline_vault_refs_in_str(&kv_val)?;
    if refs.is_empty() {
      validate_buildkit_secret_id(&kv_var)?;
      let mut interpolated_val = kv_val;
      interpolator.interpolate_string(&mut interpolated_val)?;
      standard_map.insert(kv_var, interpolated_val);
    } else {
      let unquoted_tpl =
        strip_matching_outer_template_quotes(&kv_val);
      let (final_val, used_refs) = substitute_value_single_pass(
        unquoted_tpl,
        &inline_values,
        interpolator,
      )?;
      raw_replacers
        .push((final_val.clone(), format!("build_secret:{kv_var}")));
      for used in used_refs {
        let ver = inline_values.get(&used).and_then(|v| v.version);
        provenance.push(DeployedSecretProvenance {
          provider_slug: used.provider_slug,
          remote_key: used.secret_key,
          target: format!("build_secret:{kv_var}"),
          version: ver,
        });
      }
      standard_map.shift_remove(&kv_var);
      external_map.insert(kv_var, final_val);
    }
  }

  for (binding, (prov, sel)) in build
    .config
    .secret_bindings
    .iter()
    .zip(binding_selectors.into_iter())
  {
    let Some(resolved) = batch_values
      .get(&(prov.id.clone(), sel, binding.remote_key.clone()))
      .and_then(|v| v.clone())
    else {
      continue;
    };
    let txt = resolved.as_text()?.to_string();
    if (standard_map.contains_key(&binding.secret_id)
      || external_map.contains_key(&binding.secret_id))
      && binding.conflict_policy == SecretConflictPolicy::Fail
    {
      bail!(
        "Conflict on BuildKit secret id '{}': already defined and binding conflict_policy=Fail",
        binding.secret_id
      );
    }
    standard_map.shift_remove(&binding.secret_id);
    raw_replacers.push((
      txt.clone(),
      format!("vault.{}.{}", prov.slug, binding.remote_key),
    ));
    raw_replacers.push((
      txt.clone(),
      format!("build_secret:{}", binding.secret_id),
    ));
    external_map.insert(binding.secret_id.clone(), txt);
    provenance.push(DeployedSecretProvenance {
      provider_slug: prov.slug,
      remote_key: binding.remote_key.clone(),
      target: format!("build_secret:{}", binding.secret_id),
      version: resolved.version,
    });
  }

  Ok(Some(ResolvedBuildSecrets {
    standard_secret_args: standard_map
      .into_iter()
      .map(|(variable, value)| EnvironmentVar { variable, value })
      .collect(),
    external_build_secrets: external_map
      .into_iter()
      .map(|(variable, value)| EnvironmentVar { variable, value })
      .collect(),
    replacers: finalize_secret_replacers(raw_replacers),
    provenance,
  }))
}

/// Non-sensitive validation & upstream preview helper for `PreviewResourceSecrets`.
/// Never returns secret values under any circumstance!
pub fn build_preview_items_from_provenance(
  provenance: &[DeployedSecretProvenance],
  last_deployed_at: Option<i64>,
  deployed_provenance: Vec<DeployedSecretProvenance>,
) -> PreviewResourceSecretsResponse {
  let mut grouped: IndexMap<String, (Vec<String>, Option<i64>)> =
    IndexMap::new();
  for p in provenance {
    let entry = grouped
      .entry(p.target.clone())
      .or_insert_with(|| (Vec::new(), p.version));
    entry
      .0
      .push(format!("vault.{}.{}", p.provider_slug, p.remote_key));
    if entry.1.is_none() {
      entry.1 = p.version;
    }
  }
  let items = grouped
    .into_iter()
    .map(|(target, (references, version))| {
      ResourceSecretPreviewItem {
        target,
        references,
        status: String::from("Resolved & authorized"),
        ok: true,
        version,
      }
    })
    .collect();

  PreviewResourceSecretsResponse {
    valid: true,
    items,
    errors: Vec::new(),
    last_deployed_at,
    deployed_provenance,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn single_pass_substitution_preserves_literal_remote_values_and_never_rescans()
   {
    let mut vars = HashMap::new();
    vars.insert("DB_USER".to_string(), "komodo_user".to_string());
    vars.insert("APP_DB".to_string(), "app_prod".to_string());
    // Even if a non-secret Variable contains `[[vault...]]`, it MUST NOT be expanded into an external secret!
    vars.insert(
      "SNEAKY_VAR".to_string(),
      "[[vault.infisical-prod.DB_PASSWORD]]".to_string(),
    );

    let mut local_secrets = HashMap::new();
    local_secrets
      .insert("LOCAL_SUFFIX".to_string(), "loc#1".to_string());

    let mut interp = Interpolator::new(Some(&vars), &local_secrets);

    let mut inline_values = HashMap::new();
    // Remote value deliberately contains `[[DB_USER]]`, `[[vault.other.X]]`, `${UNSET}`, `%40`, `#`, `/`, quotes, backslash, newline!
    inline_values.insert(
      ExternalSecretRef {
        provider_slug: "infisical-prod".into(),
        secret_key: "DB_PASSWORD".into(),
      },
      ResolvedSecretValue::from_text(
        "p@ss#word/123_[[DB_USER]]_[[vault.other.X]]_${UNSET}_\"q\"\\b\nl2"
          .into(),
        Some(4),
      ),
    );
    inline_values.insert(
      ExternalSecretRef {
        provider_slug: "infisical-prod".into(),
        secret_key: "DATABASE_URL".into(),
      },
      ResolvedSecretValue::from_text(
        "postgres://user:p%40ss%23word%2F123@db:5432/app".into(),
        Some(1),
      ),
    );

    // 1. Whole-value pre-encoded URL: zero double-encoding or mutation!
    let (whole, used1) = substitute_value_single_pass(
      "[[vault.infisical-prod.DATABASE_URL]]",
      &inline_values,
      &mut interp,
    )
    .unwrap();
    assert_eq!(
      whole,
      "postgres://user:p%40ss%23word%2F123@db:5432/app"
    );
    assert_eq!(used1.len(), 1);

    // 2. Mixed string with Komodo Variable + local Secret + repeated external Vault ref + literal remote `[[...]]` & `${...}`
    let (mixed, used2) = substitute_value_single_pass(
      "postgres://[[DB_USER]]:[[vault.infisical-prod.DB_PASSWORD]]@db:5432/[[APP_DB]]?tag=[[LOCAL_SUFFIX]]&sneaky=[[SNEAKY_VAR]]&dup=[[vault.infisical-prod.DB_PASSWORD]]",
      &inline_values,
      &mut interp,
    )
    .unwrap();
    assert_eq!(
      mixed,
      "postgres://komodo_user:p@ss#word/123_[[DB_USER]]_[[vault.other.X]]_${UNSET}_\"q\"\\b\nl2@db:5432/app_prod?tag=loc#1&sneaky=[[vault.infisical-prod.DB_PASSWORD]]&dup=p@ss#word/123_[[DB_USER]]_[[vault.other.X]]_${UNSET}_\"q\"\\b\nl2"
    );
    assert_eq!(used2.len(), 2);

    // 3. Filter syntax fails closed!
    assert!(
      substitute_value_single_pass(
        "[[vault.infisical-prod.DB_PASSWORD|urlencode]]",
        &inline_values,
        &mut interp
      )
      .is_err()
    );
  }

  #[test]
  fn secret_replacers_are_sorted_longest_first_and_exclude_empty() {
    let replacers = finalize_secret_replacers(vec![
      ("".to_string(), "vault.p.EMPTY".to_string()),
      ("abc".to_string(), "vault.p.SHORT_PW".to_string()),
      (
        "postgres://user:abc@db:5432/app$1".to_string(),
        "env:DATABASE_URL".to_string(),
      ),
    ]);
    assert!(replacers.iter().all(|(v, _)| !v.is_empty()));
    // Longest (`$$` / `\$` escaped full URL, then raw full URL, then `abc`) MUST come first!
    for pair in replacers.windows(2) {
      assert!(pair[0].0.len() >= pair[1].0.len());
    }
    let sanitized = svi::replace_in_string(
      "connecting to postgres://user:abc@db:5432/app$1 with pass abc",
      &replacers,
    );
    assert!(!sanitized.contains("postgres://user:abc"));
    assert!(!sanitized.contains("abc"));
  }

  #[test]
  fn unsupported_fields_reject_vault_refs_cleanly() {
    let mut build = Build::default();
    build.config.build_args =
      "NPM_TOKEN=[[vault.infisical-prod.NPM_TOKEN]]".to_string();
    let err =
      validate_build_unsupported_vault_fields(&build).unwrap_err();
    assert!(format!("{err:#}").contains("build_args"));

    let mut stack = Stack::default();
    stack.config.file_contents =
      "services:\n  app:\n    environment:\n      X: [[vault.infisical-prod.X]]"
        .to_string();
    let err2 =
      validate_stack_unsupported_vault_fields(&stack).unwrap_err();
    assert!(format!("{err2:#}").contains("file_contents"));
  }

  #[test]
  fn strips_template_wrapping_quotes_while_preserving_remote_quotes()
  {
    let vars = HashMap::new();
    let secrets = HashMap::new();
    let mut interp = Interpolator::new(Some(&vars), &secrets);
    let mut inline = HashMap::new();
    inline.insert(
      ExternalSecretRef {
        provider_slug: String::from("infisical-prod"),
        secret_key: String::from("QUOTED_VAL"),
      },
      ResolvedSecretValue::from_text(
        String::from("\"hello-world\""),
        Some(1),
      ),
    );

    // Case 1: unquoted template -> remote quotes preserved verbatim
    let tpl1 = strip_matching_outer_template_quotes(
      "[[vault.infisical-prod.QUOTED_VAL]]",
    );
    let (v1, _) =
      substitute_value_single_pass(tpl1, &inline, &mut interp)
        .unwrap();
    assert_eq!(v1, "\"hello-world\"");

    // Case 2: quoted template in komodo.toml / UI -> outer template quotes stripped once, remote quotes preserved
    let tpl2 = strip_matching_outer_template_quotes(
      "\"prefix:[[vault.infisical-prod.QUOTED_VAL]]:suffix\"",
    );
    let (v2, _) =
      substitute_value_single_pass(tpl2, &inline, &mut interp)
        .unwrap();
    assert_eq!(v2, "prefix:\"hello-world\":suffix");
  }
}
