use std::{
  collections::{HashMap, HashSet},
  sync::{Arc, LazyLock, Mutex},
  time::{Duration, Instant},
};

use anyhow::{Context, bail};
use indexmap::IndexMap;
use komodo_client::entities::secret_provider::{
  ExternalSecretMetadataItem, InfisicalProviderConfig,
  InfisicalSecretSelector, SecretCheckStageStatus, SecretProvider,
  SecretProviderCapabilities, SecretProviderConfig,
  SecretProviderCredentialInput, SecretProviderKind, SecretSelector,
  SecretTestStageResult, SecretValueFormat,
  TestSecretProviderResponse, normalize_secret_path,
};
use rand::RngExt as _;
use reqwest::{Method, StatusCode, header::RETRY_AFTER};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex as AsyncMutex;
use tokio_util::sync::CancellationToken;

use super::{
  adapter::{
    AdapterFuture, AdapterResolveItemRequest, ResolvedSecretValue,
    SecretProviderAdapter,
  },
  http::{
    DEFAULT_OPERATION_DEADLINE, HardenedHttpClient,
    canonicalize_https_endpoint, effective_allow_loopback,
    evict_provider_http_clients, parse_ca_certificates,
    summarize_upstream_error_body,
  },
};

pub static INFISICAL_ADAPTER: InfisicalAdapter = InfisicalAdapter;

const LOGIN_SUBPATH: &str = "/api/v1/auth/universal-auth/login";
const SECRETS_V4_SUBPATH: &str = "/api/v4/secrets";
const MAX_TRANSIENT_RETRIES: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TokenCacheKey {
  provider_id: String,
  revision: i64,
  credential_revision: i64,
}

#[derive(Clone)]
struct CachedInfisicalToken {
  access_token: String,
  refresh_at: Instant,
}

impl std::fmt::Debug for CachedInfisicalToken {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CachedInfisicalToken")
      .field("access_token", &"[REDACTED]")
      .field("refresh_at", &self.refresh_at)
      .finish()
  }
}

type TokenSlot = Arc<AsyncMutex<Option<CachedInfisicalToken>>>;

static TOKEN_CACHE: LazyLock<
  Mutex<HashMap<TokenCacheKey, TokenSlot>>,
> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// Immediately evict all cached Universal Auth tokens and HTTP connection pools for `provider_id`.
/// Must be called whenever a provider is updated, disabled, credential-cleared, key-rotated, or deleted.
pub fn invalidate_infisical_provider_cache(provider_id: &str) {
  if provider_id.is_empty() {
    return;
  }
  if let Ok(mut guard) = TOKEN_CACHE.lock() {
    guard.retain(|k, _| k.provider_id != provider_id);
  }
  evict_provider_http_clients(provider_id);
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UniversalAuthLoginRequest<'a> {
  client_id: &'a str,
  client_secret: &'a str,
  #[serde(skip_serializing_if = "Option::is_none")]
  organization_slug: Option<&'a str>,
}

fn default_expires_in() -> u64 {
  7200
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UniversalAuthLoginResponse {
  access_token: String,
  #[serde(default = "default_expires_in")]
  expires_in: u64,
}

/// Raw Infisical v4 secret entry for value resolution (`viewSecretValue=true`).
/// Intentionally omits `Debug` so deserialized secret values are never logged.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InfisicalV4SecretItem {
  secret_key: String,
  #[serde(default)]
  secret_value: Option<String>,
  #[serde(default)]
  secret_value_hidden: Option<bool>,
  #[serde(default)]
  version: Option<i64>,
  #[serde(default, alias = "comment")]
  secret_comment: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InfisicalV4ImportBlock {
  #[serde(default)]
  secret_path: String,
  #[serde(default)]
  environment: String,
  #[serde(default)]
  secrets: Vec<InfisicalV4SecretItem>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InfisicalV4SecretsResponse {
  #[serde(default)]
  secrets: Vec<InfisicalV4SecretItem>,
  #[serde(default)]
  imports: Vec<InfisicalV4ImportBlock>,
}

/// Metadata-only Infisical v4 deserialization struct (`viewSecretValue=false`).
/// Never deserializes `secretValue`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InfisicalV4MetadataSecretItem {
  secret_key: String,
  #[serde(default)]
  secret_value_hidden: Option<bool>,
  #[serde(default)]
  version: Option<i64>,
  #[serde(default, alias = "comment")]
  secret_comment: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InfisicalV4MetadataImportBlock {
  #[serde(default)]
  secret_path: String,
  #[serde(default)]
  environment: String,
  #[serde(default)]
  secrets: Vec<InfisicalV4MetadataSecretItem>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InfisicalV4MetadataResponse {
  #[serde(default)]
  secrets: Vec<InfisicalV4MetadataSecretItem>,
  #[serde(default)]
  imports: Vec<InfisicalV4MetadataImportBlock>,
}

struct MergedSecretEntry {
  secret_value: Option<String>,
  secret_value_hidden: bool,
  version: Option<i64>,
  comment: String,
  imported: bool,
  source_environment: String,
  source_path: String,
}

pub struct InfisicalAdapter;

impl InfisicalAdapter {
  fn extract_config<'a>(
    config: &'a SecretProviderConfig,
  ) -> &'a InfisicalProviderConfig {
    match config {
      SecretProviderConfig::Infisical(cfg) => cfg,
    }
  }

  fn extract_credential<'a>(
    credential: &'a SecretProviderCredentialInput,
  ) -> &'a str {
    match credential {
      SecretProviderCredentialInput::Infisical { client_secret } => {
        client_secret.as_str()
      }
    }
  }

  fn extract_selector<'a>(
    selector: &'a SecretSelector,
  ) -> &'a InfisicalSecretSelector {
    match selector {
      SecretSelector::Infisical(sel) => sel,
    }
  }

  fn validate_infisical_selector_fields(
    sel: &InfisicalSecretSelector,
  ) -> anyhow::Result<String> {
    let project_id = sel.project_id.trim();
    if project_id.is_empty() {
      bail!("Infisical selector 'project_id' cannot be empty");
    }
    if project_id.bytes().any(|b| b <= 0x20 || b == 0x7f) {
      bail!(
        "Infisical selector 'project_id' contains invalid whitespace or control characters"
      );
    }
    let env = sel.environment.trim();
    if env.is_empty() {
      bail!("Infisical selector 'environment' cannot be empty");
    }
    if env.bytes().any(|b| b <= 0x20 || b == 0x7f)
      || env.contains('/')
      || env.contains('\\')
    {
      bail!(
        "Infisical selector 'environment' '{env}' contains invalid characters"
      );
    }
    normalize_secret_path(&sel.secret_path)
  }

  fn get_token_slot(key: &TokenCacheKey) -> TokenSlot {
    let mut guard = TOKEN_CACHE.lock().unwrap();
    guard
      .entry(key.clone())
      .or_insert_with(|| Arc::new(AsyncMutex::new(None)))
      .clone()
  }

  async fn login_fresh(
    http: &HardenedHttpClient,
    provider_label: &str,
    cfg: &InfisicalProviderConfig,
    client_secret: &str,
    deadline: Instant,
    cancel: &CancellationToken,
  ) -> anyhow::Result<(String, Instant)> {
    let org_slug = {
      let trimmed = cfg.organization_slug.trim();
      if trimmed.is_empty() {
        None
      } else {
        Some(trimmed)
      }
    };
    let req_body = UniversalAuthLoginRequest {
      client_id: cfg.client_id.trim(),
      client_secret,
      organization_slug: org_slug,
    };

    let mut attempt = 0usize;
    loop {
      let res = http
        .send_bounded(
          Method::POST,
          LOGIN_SUBPATH,
          |b| b.json(&req_body),
          deadline,
          cancel,
        )
        .await;

      match res {
        Ok(resp) if resp.status.is_success() => {
          let parsed: UniversalAuthLoginResponse =
            serde_json::from_slice(&resp.body).with_context(|| {
              format!(
                "Invalid JSON response from Infisical Universal Auth login for provider '{provider_label}'"
              )
            })?;
          let token = parsed.access_token;
          if token.trim().is_empty() {
            bail!(
              "Infisical Universal Auth login returned an empty accessToken for provider '{provider_label}'"
            );
          }
          let ttl_secs = parsed.expires_in.max(1);
          let early_margin = Duration::from_secs(30)
            .min(Duration::from_secs(ttl_secs) / 5);
          let refresh_after = Duration::from_secs(ttl_secs)
            .saturating_sub(early_margin);
          return Ok((token, Instant::now() + refresh_after));
        }
        Ok(resp)
          if (resp.status == StatusCode::TOO_MANY_REQUESTS
            || resp.status.is_server_error())
            && attempt < MAX_TRANSIENT_RETRIES =>
        {
          let wait = compute_retry_delay(
            resp.status,
            resp.headers.get(RETRY_AFTER),
            attempt,
            deadline,
          )?;
          sleep_with_cancel(wait, cancel).await?;
          attempt += 1;
        }
        Ok(resp) => {
          let detail = summarize_upstream_error_body(&resp.body);
          bail!(
            "Infisical Universal Auth login failed for provider '{provider_label}' with HTTP {}: {detail}",
            resp.status
          );
        }
        Err(err)
          if attempt < MAX_TRANSIENT_RETRIES
            && is_retryable_transport_error(
              &err, cancel, deadline,
            ) =>
        {
          let wait = compute_backoff_delay(attempt, deadline)?;
          sleep_with_cancel(wait, cancel).await?;
          attempt += 1;
        }
        Err(err) => {
          return Err(err.context(format!(
            "Infisical Universal Auth login request failed for provider '{provider_label}'"
          )));
        }
      }
    }
  }

  /// Obtain an access token using singleflight deduplication per `(provider_id, revision, credential_revision)`.
  async fn get_or_login_token(
    http: &HardenedHttpClient,
    provider: &SecretProvider,
    cfg: &InfisicalProviderConfig,
    client_secret: &str,
    force_refresh: bool,
    deadline: Instant,
    cancel: &CancellationToken,
  ) -> anyhow::Result<String> {
    let label = if provider.slug.is_empty() {
      provider.name.as_str()
    } else {
      provider.slug.as_str()
    };

    // Unsaved ad-hoc providers (`provider.id.is_empty()`) bypass the shared cache.
    if provider.id.is_empty() {
      let (token, _) = Self::login_fresh(
        http,
        label,
        cfg,
        client_secret,
        deadline,
        cancel,
      )
      .await?;
      return Ok(token);
    }

    let key = TokenCacheKey {
      provider_id: provider.id.clone(),
      revision: provider.revision,
      credential_revision: provider.credential_revision,
    };
    let slot = Self::get_token_slot(&key);

    let mut guard = tokio::select! {
      biased;
      _ = cancel.cancelled() => {
        bail!("Cancelled waiting for Infisical singleflight authentication");
      }
      g = slot.lock() => g,
    };

    if !force_refresh
      && let Some(cached) = guard.as_ref()
      && Instant::now() < cached.refresh_at
    {
      return Ok(cached.access_token.clone());
    }

    // Evict stale token before attempting fresh login so failures never leave expired state.
    *guard = None;

    let (token, refresh_at) = Self::login_fresh(
      http,
      label,
      cfg,
      client_secret,
      deadline,
      cancel,
    )
    .await?;

    *guard = Some(CachedInfisicalToken {
      access_token: token.clone(),
      refresh_at,
    });
    Ok(token)
  }

  async fn fetch_v4_secrets_raw(
    http: &HardenedHttpClient,
    provider: &SecretProvider,
    cfg: &InfisicalProviderConfig,
    client_secret: &str,
    sel: &InfisicalSecretSelector,
    normalized_path: &str,
    view_secret_value: bool,
    explicit_token: Option<String>,
    deadline: Instant,
    cancel: &CancellationToken,
  ) -> anyhow::Result<Vec<u8>> {
    let label = if provider.slug.is_empty() {
      provider.name.as_str()
    } else {
      provider.slug.as_str()
    };

    let query_params: [(&str, &str); 8] = [
      ("projectId", sel.project_id.trim()),
      ("environment", sel.environment.trim()),
      ("secretPath", normalized_path),
      (
        "viewSecretValue",
        if view_secret_value { "true" } else { "false" },
      ),
      (
        "includeImports",
        if sel.include_imports { "true" } else { "false" },
      ),
      (
        "expandSecretReferences",
        if sel.expand_references {
          "true"
        } else {
          "false"
        },
      ),
      ("recursive", "false"),
      ("includePersonalOverrides", "false"),
    ];

    let mut refreshed_401 = false;
    let mut transient_attempt = 0usize;
    let mut current_token = match explicit_token {
      Some(tok) => tok,
      None => {
        Self::get_or_login_token(
          http,
          provider,
          cfg,
          client_secret,
          false,
          deadline,
          cancel,
        )
        .await?
      }
    };

    loop {
      let token_ref = &current_token;
      let res = http
        .send_bounded(
          Method::GET,
          SECRETS_V4_SUBPATH,
          |b| b.bearer_auth(token_ref).query(&query_params),
          deadline,
          cancel,
        )
        .await;

      match res {
        Ok(resp) if resp.status.is_success() => {
          return Ok(resp.body);
        }
        // Single 401 token refresh & replay
        Ok(resp)
          if resp.status == StatusCode::UNAUTHORIZED
            && !refreshed_401 =>
        {
          refreshed_401 = true;
          current_token = Self::get_or_login_token(
            http,
            provider,
            cfg,
            client_secret,
            true,
            deadline,
            cancel,
          )
          .await?;
        }
        // 403 Forbidden: never retry or re-login
        Ok(resp) if resp.status == StatusCode::FORBIDDEN => {
          let detail = summarize_upstream_error_body(&resp.body);
          bail!(
            "Infisical returned 403 Forbidden for provider '{label}' (project '{}', environment '{}', path '{normalized_path}'): {detail}",
            sel.project_id.trim(),
            sel.environment.trim()
          );
        }
        Ok(resp)
          if (resp.status == StatusCode::TOO_MANY_REQUESTS
            || resp.status.is_server_error())
            && transient_attempt < MAX_TRANSIENT_RETRIES =>
        {
          let wait = compute_retry_delay(
            resp.status,
            resp.headers.get(RETRY_AFTER),
            transient_attempt,
            deadline,
          )?;
          sleep_with_cancel(wait, cancel).await?;
          transient_attempt += 1;
        }
        Ok(resp) => {
          let detail = summarize_upstream_error_body(&resp.body);
          bail!(
            "Infisical Secrets v4 request failed for provider '{label}' (project '{}', environment '{}', path '{normalized_path}') with HTTP {}: {detail}",
            sel.project_id.trim(),
            sel.environment.trim(),
            resp.status
          );
        }
        Err(err)
          if transient_attempt < MAX_TRANSIENT_RETRIES
            && is_retryable_transport_error(
              &err, cancel, deadline,
            ) =>
        {
          let wait =
            compute_backoff_delay(transient_attempt, deadline)?;
          sleep_with_cancel(wait, cancel).await?;
          transient_attempt += 1;
        }
        Err(err) => {
          return Err(err.context(format!(
            "Infisical Secrets v4 request failed for provider '{label}'"
          )));
        }
      }
    }
  }

  /// Merge Infisical v4 `imports` and top-level `secrets` according to Infisical's exact precedence rules:
  /// 1. When `include_imports == true`, iterate `imports[0..n]` in response order; within a single `imports[k].secrets`,
  ///    the first occurrence of a key wins; later `imports[k]` overrides earlier `imports[j]` (`j < k`).
  /// 2. Top-level `secrets[]` overrides all `imports[]`.
  fn merge_v4_value_response(
    parsed: InfisicalV4SecretsResponse,
    sel: &InfisicalSecretSelector,
    normalized_path: &str,
  ) -> IndexMap<String, MergedSecretEntry> {
    let mut merged = IndexMap::new();

    if sel.include_imports {
      for imp in parsed.imports {
        let mut seen_in_import = HashSet::new();
        for item in imp.secrets {
          if !seen_in_import.insert(item.secret_key.clone()) {
            continue;
          }
          merged.insert(
            item.secret_key,
            MergedSecretEntry {
              secret_value: item.secret_value,
              secret_value_hidden: item
                .secret_value_hidden
                .unwrap_or(false),
              version: item.version,
              comment: item.secret_comment.unwrap_or_default(),
              imported: true,
              source_environment: imp.environment.clone(),
              source_path: imp.secret_path.clone(),
            },
          );
        }
      }
    }

    for item in parsed.secrets {
      merged.insert(
        item.secret_key,
        MergedSecretEntry {
          secret_value: item.secret_value,
          secret_value_hidden: item
            .secret_value_hidden
            .unwrap_or(false),
          version: item.version,
          comment: item.secret_comment.unwrap_or_default(),
          imported: false,
          source_environment: sel.environment.trim().to_string(),
          source_path: normalized_path.to_string(),
        },
      );
    }

    merged
  }

  fn merge_v4_metadata_response(
    parsed: InfisicalV4MetadataResponse,
    sel: &InfisicalSecretSelector,
    normalized_path: &str,
  ) -> Vec<ExternalSecretMetadataItem> {
    let mut merged: IndexMap<String, ExternalSecretMetadataItem> =
      IndexMap::new();

    if sel.include_imports {
      for imp in parsed.imports {
        let mut seen_in_import = HashSet::new();
        for item in imp.secrets {
          if !seen_in_import.insert(item.secret_key.clone()) {
            continue;
          }
          merged.insert(
            item.secret_key.clone(),
            ExternalSecretMetadataItem {
              key: item.secret_key,
              version: item.version,
              comment: item.secret_comment.unwrap_or_default(),
              imported: true,
              source_environment: imp.environment.clone(),
              source_path: imp.secret_path.clone(),
              value_hidden: item.secret_value_hidden.unwrap_or(false),
            },
          );
        }
      }
    }

    for item in parsed.secrets {
      merged.insert(
        item.secret_key.clone(),
        ExternalSecretMetadataItem {
          key: item.secret_key,
          version: item.version,
          comment: item.secret_comment.unwrap_or_default(),
          imported: false,
          source_environment: sel.environment.trim().to_string(),
          source_path: normalized_path.to_string(),
          value_hidden: item.secret_value_hidden.unwrap_or(false),
        },
      );
    }

    let mut out: Vec<ExternalSecretMetadataItem> =
      merged.into_values().collect();
    out.sort_by(|a, b| a.key.cmp(&b.key));
    out
  }
}

impl SecretProviderAdapter for InfisicalAdapter {
  fn kind(&self) -> SecretProviderKind {
    SecretProviderKind::Infisical
  }

  fn capabilities(&self) -> SecretProviderCapabilities {
    SecretProviderCapabilities {
      supports_metadata_listing: true,
      supports_versions: true,
      supports_binary_values: false,
      supports_json_selector: false,
      supports_leases: false,
    }
  }

  fn validate_config(
    &self,
    config: &SecretProviderConfig,
  ) -> anyhow::Result<()> {
    let cfg = Self::extract_config(config);
    canonicalize_https_endpoint(
      &cfg.endpoint,
      cfg.allow_private_ips,
      effective_allow_loopback(),
    )?;
    if cfg.client_id.trim().is_empty() {
      bail!(
        "Infisical Machine Identity Universal Auth 'client_id' cannot be empty"
      );
    }
    if cfg.client_id.trim().bytes().any(|b| b <= 0x20 || b == 0x7f) {
      bail!(
        "Infisical 'client_id' contains invalid whitespace or control characters"
      );
    }
    if !cfg.organization_slug.trim().is_empty()
      && cfg
        .organization_slug
        .trim()
        .bytes()
        .any(|b| b <= 0x20 || b == 0x7f)
    {
      bail!(
        "Infisical 'organization_slug' contains invalid whitespace or control characters"
      );
    }
    parse_ca_certificates(&cfg.ca_cert_pem)?;
    if let Some(default_sel) = &cfg.default_selector {
      Self::validate_infisical_selector_fields(default_sel)?;
    }
    Ok(())
  }

  fn validate_credential(
    &self,
    credential: &SecretProviderCredentialInput,
  ) -> anyhow::Result<()> {
    let secret = Self::extract_credential(credential);
    if secret.trim().is_empty() {
      bail!(
        "Infisical Universal Auth 'client_secret' cannot be empty"
      );
    }
    Ok(())
  }

  fn validate_selector(
    &self,
    _config: &SecretProviderConfig,
    selector: &SecretSelector,
  ) -> anyhow::Result<()> {
    let sel = Self::extract_selector(selector);
    Self::validate_infisical_selector_fields(sel)?;
    Ok(())
  }

  fn test_connection<'a>(
    &'a self,
    provider: &'a SecretProvider,
    credential: &'a SecretProviderCredentialInput,
    selector_override: Option<&'a SecretSelector>,
    sample_key: Option<&'a str>,
    cancel: &'a CancellationToken,
  ) -> AdapterFuture<'a, TestSecretProviderResponse> {
    Box::pin(async move {
      self.validate_config(&provider.config)?;
      self.validate_credential(credential)?;

      let cfg = Self::extract_config(&provider.config);
      let client_secret = Self::extract_credential(credential);
      let http = HardenedHttpClient::for_provider(
        &provider.id,
        &cfg.endpoint,
        &cfg.ca_cert_pem,
        cfg.allow_private_ips,
      )?;
      let deadline = Instant::now() + DEFAULT_OPERATION_DEADLINE;
      let label = if provider.slug.is_empty() {
        provider.name.as_str()
      } else {
        provider.slug.as_str()
      };

      // Stage 1: Fresh Universal Auth login (verifies live credentials directly, bypassing any cached token).
      let token = match Self::login_fresh(
        &http,
        label,
        cfg,
        client_secret,
        deadline,
        cancel,
      )
      .await
      {
        Ok((tok, _)) => tok,
        Err(err) => {
          return Ok(TestSecretProviderResponse {
            ready_for_deploy: false,
            authentication: SecretTestStageResult {
              status: SecretCheckStageStatus::Failed,
              message: format!("{err:#}"),
            },
            metadata_access: SecretTestStageResult {
              status: SecretCheckStageStatus::Skipped,
              message: String::from(
                "Skipped because Universal Auth login failed",
              ),
            },
            value_read_access: SecretTestStageResult {
              status: SecretCheckStageStatus::Skipped,
              message: String::from(
                "Skipped because Universal Auth login failed",
              ),
            },
            key_count: None,
          });
        }
      };

      let auth_stage = SecretTestStageResult {
        status: SecretCheckStageStatus::Passed,
        message: String::from(
          "Universal Auth login succeeded (/api/v1/auth/universal-auth/login)",
        ),
      };

      // Determine selector for Stage 2 & Stage 3
      let effective_sel: Option<InfisicalSecretSelector> =
        match selector_override {
          Some(SecretSelector::Infisical(s)) => Some(s.clone()),
          None => cfg.default_selector.clone(),
        };

      let Some(sel) = effective_sel else {
        return Ok(TestSecretProviderResponse {
          ready_for_deploy: false,
          authentication: auth_stage,
          metadata_access: SecretTestStageResult {
            status: SecretCheckStageStatus::Skipped,
            message: String::from(
              "No selector (project_id / environment / secret_path) configured or supplied; provide a default selector or test selector to verify project and ReadValue permissions",
            ),
          },
          value_read_access: SecretTestStageResult {
            status: SecretCheckStageStatus::Skipped,
            message: String::from(
              "Skipped because no test selector was provided",
            ),
          },
          key_count: None,
        });
      };

      let normalized_path =
        match Self::validate_infisical_selector_fields(&sel) {
          Ok(p) => p,
          Err(err) => {
            return Ok(TestSecretProviderResponse {
              ready_for_deploy: false,
              authentication: auth_stage,
              metadata_access: SecretTestStageResult {
                status: SecretCheckStageStatus::Failed,
                message: format!("Invalid selector: {err:#}"),
              },
              value_read_access: SecretTestStageResult {
                status: SecretCheckStageStatus::Skipped,
                message: String::from(
                  "Skipped because selector validation failed",
                ),
              },
              key_count: None,
            });
          }
        };

      // Stage 2: Metadata discovery (`viewSecretValue=false`)
      let meta_items = match Self::fetch_v4_secrets_raw(
        &http,
        provider,
        cfg,
        client_secret,
        &sel,
        &normalized_path,
        false,
        Some(token.clone()),
        deadline,
        cancel,
      )
      .await
      .and_then(|bytes| {
        serde_json::from_slice::<InfisicalV4MetadataResponse>(&bytes)
          .context("Invalid JSON in Infisical v4 metadata response")
      }) {
        Ok(parsed) => Self::merge_v4_metadata_response(
          parsed,
          &sel,
          &normalized_path,
        ),
        Err(err) => {
          return Ok(TestSecretProviderResponse {
            ready_for_deploy: false,
            authentication: auth_stage,
            metadata_access: SecretTestStageResult {
              status: SecretCheckStageStatus::Failed,
              message: format!("{err:#}"),
            },
            value_read_access: SecretTestStageResult {
              status: SecretCheckStageStatus::Skipped,
              message: String::from(
                "Skipped because metadata listing failed",
              ),
            },
            key_count: None,
          });
        }
      };

      let key_count = meta_items.len();
      let meta_stage = SecretTestStageResult {
        status: SecretCheckStageStatus::Passed,
        message: format!(
          "Metadata listing succeeded (discovered {key_count} secret key(s) at path '{normalized_path}')"
        ),
      };

      // Stage 3: Actual value-read verification (`viewSecretValue=true` + `secretValueHidden` check)
      match Self::fetch_v4_secrets_raw(
        &http,
        provider,
        cfg,
        client_secret,
        &sel,
        &normalized_path,
        true,
        Some(token),
        deadline,
        cancel,
      )
      .await
      .and_then(|bytes| {
        serde_json::from_slice::<InfisicalV4SecretsResponse>(&bytes)
          .context("Invalid JSON in Infisical v4 value response")
      }) {
        Ok(parsed) => {
          let merged = Self::merge_v4_value_response(
            parsed,
            &sel,
            &normalized_path,
          );
          let hidden_keys: Vec<&str> = merged
            .iter()
            .filter(|(_, v)| v.secret_value_hidden)
            .map(|(k, _)| k.as_str())
            .collect();
          if !hidden_keys.is_empty() {
            Ok(TestSecretProviderResponse {
              ready_for_deploy: false,
              authentication: auth_stage,
              metadata_access: meta_stage,
              value_read_access: SecretTestStageResult {
                status: SecretCheckStageStatus::Failed,
                message: format!(
                  "Machine Identity can list metadata (DescribeSecret), but {} secret(s) returned secretValueHidden=true — grant ReadValue permission before deploying",
                  hidden_keys.len()
                ),
              },
              key_count: Some(key_count),
            })
          } else if let Some(sample) =
            sample_key.map(str::trim).filter(|s| !s.is_empty())
          {
            match merged.get(sample) {
              Some(entry) if !entry.secret_value_hidden => {
                Ok(TestSecretProviderResponse {
                  ready_for_deploy: true,
                  authentication: auth_stage,
                  metadata_access: meta_stage,
                  value_read_access: SecretTestStageResult {
                    status: SecretCheckStageStatus::Passed,
                    message: format!(
                      "Secret value read (ReadValue / viewSecretValue=true) verified including sample key '{sample}'"
                    ),
                  },
                  key_count: Some(key_count),
                })
              }
              Some(_) => Ok(TestSecretProviderResponse {
                ready_for_deploy: false,
                authentication: auth_stage,
                metadata_access: meta_stage,
                value_read_access: SecretTestStageResult {
                  status: SecretCheckStageStatus::Failed,
                  message: format!(
                    "Sample secret key '{sample}' returned secretValueHidden=true — grant ReadValue permission"
                  ),
                },
                key_count: Some(key_count),
              }),
              None => Ok(TestSecretProviderResponse {
                ready_for_deploy: false,
                authentication: auth_stage,
                metadata_access: meta_stage,
                value_read_access: SecretTestStageResult {
                  status: SecretCheckStageStatus::Failed,
                  message: format!(
                    "Sample secret key '{sample}' was not found in project '{}', environment '{}', path '{normalized_path}'",
                    sel.project_id.trim(),
                    sel.environment.trim()
                  ),
                },
                key_count: Some(key_count),
              }),
            }
          } else {
            Ok(TestSecretProviderResponse {
              ready_for_deploy: true,
              authentication: auth_stage,
              metadata_access: meta_stage,
              value_read_access: SecretTestStageResult {
                status: SecretCheckStageStatus::Passed,
                message: String::from(
                  "Secret value read (ReadValue / viewSecretValue=true) verified",
                ),
              },
              key_count: Some(key_count),
            })
          }
        }
        Err(err) => Ok(TestSecretProviderResponse {
          ready_for_deploy: false,
          authentication: auth_stage,
          metadata_access: meta_stage,
          value_read_access: SecretTestStageResult {
            status: SecretCheckStageStatus::Failed,
            message: format!("{err:#}"),
          },
          key_count: Some(key_count),
        }),
      }
    })
  }

  fn list_metadata<'a>(
    &'a self,
    provider: &'a SecretProvider,
    credential: &'a SecretProviderCredentialInput,
    selector: &'a SecretSelector,
    cancel: &'a CancellationToken,
  ) -> AdapterFuture<'a, Vec<ExternalSecretMetadataItem>> {
    Box::pin(async move {
      self.validate_config(&provider.config)?;
      self.validate_credential(credential)?;

      let cfg = Self::extract_config(&provider.config);
      let client_secret = Self::extract_credential(credential);
      let sel = Self::extract_selector(selector);
      let normalized_path =
        Self::validate_infisical_selector_fields(sel)?;

      let http = HardenedHttpClient::for_provider(
        &provider.id,
        &cfg.endpoint,
        &cfg.ca_cert_pem,
        cfg.allow_private_ips,
      )?;
      let deadline = Instant::now() + DEFAULT_OPERATION_DEADLINE;

      // Always request `viewSecretValue=false` and deserialize into metadata-only struct.
      let bytes = Self::fetch_v4_secrets_raw(
        &http,
        provider,
        cfg,
        client_secret,
        sel,
        &normalized_path,
        false,
        None,
        deadline,
        cancel,
      )
      .await?;

      let parsed: InfisicalV4MetadataResponse =
        serde_json::from_slice(&bytes).with_context(|| {
          format!(
            "Invalid JSON in Infisical v4 metadata response for provider '{}'",
            provider.slug
          )
        })?;

      Ok(Self::merge_v4_metadata_response(
        parsed,
        sel,
        &normalized_path,
      ))
    })
  }

  fn resolve_batch<'a>(
    &'a self,
    provider: &'a SecretProvider,
    credential: &'a SecretProviderCredentialInput,
    selector: &'a SecretSelector,
    items: &'a [AdapterResolveItemRequest],
    cancel: &'a CancellationToken,
  ) -> AdapterFuture<'a, IndexMap<String, Option<ResolvedSecretValue>>>
  {
    Box::pin(async move {
      if items.is_empty() {
        return Ok(IndexMap::new());
      }
      self.validate_config(&provider.config)?;
      self.validate_credential(credential)?;

      // Validate adapter-level capabilities before making any network calls (fail-closed).
      for item in items {
        if item.remote_key.is_empty() {
          bail!(
            "External secret remote_key cannot be empty on provider '{}'",
            provider.slug
          );
        }
        if item.format == SecretValueFormat::Binary {
          bail!(
            "Infisical provider '{}' returns UTF-8 text secrets and does not perform implicit binary/base64 decoding (requested format=Binary for key '{}'); use format=Text",
            provider.slug,
            item.remote_key
          );
        }
        if !item.json_field.trim().is_empty() {
          bail!(
            "JSON field extraction ('{}' on key '{}') is not enabled for Infisical v1 provider '{}'",
            item.json_field,
            item.remote_key,
            provider.slug
          );
        }
      }

      let cfg = Self::extract_config(&provider.config);
      let client_secret = Self::extract_credential(credential);
      let sel = Self::extract_selector(selector);
      let normalized_path =
        Self::validate_infisical_selector_fields(sel)?;

      let http = HardenedHttpClient::for_provider(
        &provider.id,
        &cfg.endpoint,
        &cfg.ca_cert_pem,
        cfg.allow_private_ips,
      )?;
      let deadline = Instant::now() + DEFAULT_OPERATION_DEADLINE;

      let bytes = Self::fetch_v4_secrets_raw(
        &http,
        provider,
        cfg,
        client_secret,
        sel,
        &normalized_path,
        true,
        None,
        deadline,
        cancel,
      )
      .await?;

      let parsed: InfisicalV4SecretsResponse =
        serde_json::from_slice(&bytes).with_context(|| {
          format!(
            "Invalid JSON in Infisical v4 secrets response for provider '{}'",
            provider.slug
          )
        })?;

      let merged =
        Self::merge_v4_value_response(parsed, sel, &normalized_path);

      let mut out = IndexMap::with_capacity(items.len());
      for req in items {
        match merged.get(&req.remote_key) {
          None => {
            if req.required {
              bail!(
                "Required external secret '{}' not found in Infisical provider '{}' (project '{}', environment '{}', path '{normalized_path}')",
                req.remote_key,
                provider.slug,
                sel.project_id.trim(),
                sel.environment.trim()
              );
            }
            out.insert(req.remote_key.clone(), None);
          }
          Some(entry) => {
            if entry.secret_value_hidden {
              bail!(
                "Infisical returned secretValueHidden=true for secret '{}' in provider '{}' (project '{}', environment '{}', path '{normalized_path}') — Machine Identity lacks ReadValue permission",
                req.remote_key,
                provider.slug,
                sel.project_id.trim(),
                sel.environment.trim()
              );
            }
            let raw_val =
              entry.secret_value.as_ref().ok_or_else(|| {
                anyhow::anyhow!(
                  "Infisical omitted secretValue for secret '{}' in provider '{}'",
                  req.remote_key,
                  provider.slug
                )
              })?;
            if raw_val.is_empty() && !req.allow_empty {
              bail!(
                "External secret '{}' in provider '{}' resolved to an empty value, which is forbidden when allow_empty=false",
                req.remote_key,
                provider.slug
              );
            }
            // Literal byte-for-byte UTF-8 pass-through: ZERO trimming, encoding, decoding, or interpolation!
            out.insert(
              req.remote_key.clone(),
              Some(ResolvedSecretValue::from_text(
                raw_val.clone(),
                entry.version,
              )),
            );
          }
        }
      }

      #[allow(clippy::used_underscore_binding)]
      let _ = (merged.values().map(|e| {
        (
          &e.comment,
          e.imported,
          &e.source_environment,
          &e.source_path,
        )
      }),);

      Ok(out)
    })
  }
}

fn is_retryable_transport_error(
  err: &anyhow::Error,
  cancel: &CancellationToken,
  deadline: Instant,
) -> bool {
  if cancel.is_cancelled() || Instant::now() >= deadline {
    return false;
  }
  let msg = format!("{err:#}");
  // Do NOT retry deterministic security policy / TLS certificate / SSRF / size-cap errors!
  if msg.contains("forbidden")
    || msg.contains("HTTP redirect")
    || msg.contains("exceeds maximum allowed size")
    || msg.contains("certificate")
    || msg.contains("Certificate")
    || msg.contains("UnknownIssuer")
  {
    return false;
  }
  true
}

fn compute_retry_delay(
  status: StatusCode,
  retry_after_header: Option<&reqwest::header::HeaderValue>,
  attempt: usize,
  deadline: Instant,
) -> anyhow::Result<Duration> {
  if status == StatusCode::TOO_MANY_REQUESTS
    && let Some(hv) = retry_after_header
    && let Ok(s) = hv.to_str()
    && let Ok(secs) = s.trim().parse::<u64>()
  {
    let capped = Duration::from_secs(secs.min(5));
    let now = Instant::now();
    if now + capped >= deadline {
      bail!(
        "Upstream returned 429 Too Many Requests with Retry-After={secs}s exceeding operation deadline"
      );
    }
    return Ok(capped);
  }
  compute_backoff_delay(attempt, deadline)
}

fn compute_backoff_delay(
  attempt: usize,
  deadline: Instant,
) -> anyhow::Result<Duration> {
  let base_ms: u64 = match attempt {
    0 => 150,
    _ => 400,
  };
  let jitter_ms: u64 = rand::rng().random_range(0..50);
  let wait = Duration::from_millis(base_ms + jitter_ms);
  let now = Instant::now();
  if now + wait >= deadline {
    bail!(
      "Secret provider operation deadline exceeded before retry attempt"
    );
  }
  Ok(wait)
}

async fn sleep_with_cancel(
  duration: Duration,
  cancel: &CancellationToken,
) -> anyhow::Result<()> {
  tokio::select! {
    biased;
    _ = cancel.cancelled() => {
      bail!("Secret provider request cancelled during retry backoff");
    }
    _ = tokio::time::sleep(duration) => Ok(()),
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use std::{
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
  };

  use rustls::pki_types::{
    CertificateDer, PrivateKeyDer, pem::PemObject,
  };
  use serde_json::json;
  use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
  };
  use tokio_rustls::TlsAcceptor;

  use super::*;
  use crate::secret_provider::http::{
    MAX_SECRET_RESPONSE_BYTES, set_test_allow_loopback,
  };

  pub struct GeneratedTestPki {
    pub ca_cert_pem: String,
    pub server_cert_pem: String,
    pub server_key_pem: String,
    pub untrusted_ca_cert_pem: String,
    _dir: std::path::PathBuf,
  }

  impl Drop for GeneratedTestPki {
    fn drop(&mut self) {
      let _ = std::fs::remove_dir_all(&self._dir);
    }
  }

  pub fn generate_test_pki() -> GeneratedTestPki {
    let dir = std::env::temp_dir()
      .join(format!("komodo-sp-pki-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();

    let ca_key = dir.join("ca.key");
    let ca_crt = dir.join("ca.crt");
    let ca_ext = dir.join("ca.cnf");
    std::fs::write(
      &ca_ext,
      "[req]\ndistinguished_name=dn\nx509_extensions=v3_ca\nprompt=no\n[dn]\nCN=Komodo Test Root CA\n[v3_ca]\nbasicConstraints=critical,CA:TRUE\nkeyUsage=critical,keyCertSign,cRLSign\nsubjectKeyIdentifier=hash\n",
    )
    .unwrap();

    let status = Command::new("openssl")
      .args([
        "req",
        "-x509",
        "-newkey",
        "ec",
        "-pkeyopt",
        "ec_paramgen_curve:prime256v1",
        "-days",
        "30",
        "-nodes",
        "-keyout",
        ca_key.to_str().unwrap(),
        "-out",
        ca_crt.to_str().unwrap(),
        "-config",
        ca_ext.to_str().unwrap(),
      ])
      .output()
      .unwrap();
    assert!(
      status.status.success(),
      "openssl CA gen failed: {}",
      String::from_utf8_lossy(&status.stderr)
    );

    let srv_key = dir.join("server.key");
    let srv_csr = dir.join("server.csr");
    let srv_crt = dir.join("server.crt");
    let srv_ext = dir.join("server.ext");
    std::fs::write(
      &srv_ext,
      "basicConstraints=CA:FALSE\nkeyUsage=critical,digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\nsubjectAltName=IP:127.0.0.1,DNS:localhost\n",
    )
    .unwrap();

    let status = Command::new("openssl")
      .args([
        "req",
        "-newkey",
        "ec",
        "-pkeyopt",
        "ec_paramgen_curve:prime256v1",
        "-nodes",
        "-keyout",
        srv_key.to_str().unwrap(),
        "-out",
        srv_csr.to_str().unwrap(),
        "-subj",
        "/CN=127.0.0.1",
      ])
      .output()
      .unwrap();
    assert!(status.status.success());

    let status = Command::new("openssl")
      .args([
        "x509",
        "-req",
        "-in",
        srv_csr.to_str().unwrap(),
        "-CA",
        ca_crt.to_str().unwrap(),
        "-CAkey",
        ca_key.to_str().unwrap(),
        "-CAcreateserial",
        "-out",
        srv_crt.to_str().unwrap(),
        "-days",
        "30",
        "-sha256",
        "-extfile",
        srv_ext.to_str().unwrap(),
      ])
      .output()
      .unwrap();
    assert!(status.status.success());

    // Generate a second unrelated CA to verify untrusted CA rejection
    let other_key = dir.join("other_ca.key");
    let other_crt = dir.join("other_ca.crt");
    let status = Command::new("openssl")
      .args([
        "req",
        "-x509",
        "-newkey",
        "ec",
        "-pkeyopt",
        "ec_paramgen_curve:prime256v1",
        "-days",
        "30",
        "-nodes",
        "-keyout",
        other_key.to_str().unwrap(),
        "-out",
        other_crt.to_str().unwrap(),
        "-config",
        ca_ext.to_str().unwrap(),
      ])
      .output()
      .unwrap();
    assert!(status.status.success());

    GeneratedTestPki {
      ca_cert_pem: std::fs::read_to_string(&ca_crt).unwrap(),
      server_cert_pem: std::fs::read_to_string(&srv_crt).unwrap(),
      server_key_pem: std::fs::read_to_string(&srv_key).unwrap(),
      untrusted_ca_cert_pem: std::fs::read_to_string(&other_crt)
        .unwrap(),
      _dir: dir,
    }
  }

  pub fn build_tls_acceptor(pki: &GeneratedTestPki) -> TlsAcceptor {
    let _ =
      rustls::crypto::aws_lc_rs::default_provider().install_default();
    let certs: Vec<CertificateDer<'static>> =
      CertificateDer::pem_slice_iter(pki.server_cert_pem.as_bytes())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let key =
      PrivateKeyDer::from_pem_slice(pki.server_key_pem.as_bytes())
        .unwrap();
    let cfg = rustls::ServerConfig::builder()
      .with_no_client_auth()
      .with_single_cert(certs, key)
      .unwrap();
    TlsAcceptor::from(Arc::new(cfg))
  }

  /// Minimal HTTP/1.1 over TLS mock server for deterministic adapter tests.
  async fn spawn_mock_https_server<F>(
    pki: &GeneratedTestPki,
    handler: F,
  ) -> (String, tokio::task::JoinHandle<()>)
  where
    F: Fn(
        String,
        String,
        String,
      ) -> (u16, Vec<(String, String)>, Vec<u8>)
      + Send
      + Sync
      + 'static,
  {
    let acceptor = build_tls_acceptor(pki);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handler = Arc::new(handler);

    let join = tokio::spawn(async move {
      while let Ok((tcp, _)) = listener.accept().await {
        let acceptor = acceptor.clone();
        let handler = handler.clone();
        tokio::spawn(async move {
          let Ok(mut tls) = acceptor.accept(tcp).await else {
            return;
          };
          loop {
            let mut buf = Vec::new();
            let mut tmp = [0u8; 4096];
            let header_end;
            loop {
              let Ok(n) = tls.read(&mut tmp).await else {
                return;
              };
              if n == 0 {
                return;
              }
              buf.extend_from_slice(&tmp[..n]);
              if let Some(pos) =
                buf.windows(4).position(|w| w == b"\r\n\r\n")
              {
                header_end = pos + 4;
                break;
              }
            }
            let head_str =
              String::from_utf8_lossy(&buf[..header_end]).to_string();
            let req_line =
              head_str.lines().next().unwrap_or_default().to_string();
            let content_len = head_str
              .lines()
              .find_map(|l| {
                let (k, v) = l.split_once(':')?;
                if k.eq_ignore_ascii_case("content-length") {
                  v.trim().parse::<usize>().ok()
                } else {
                  None
                }
              })
              .unwrap_or(0);
            while buf.len() < header_end + content_len {
              let Ok(n) = tls.read(&mut tmp).await else {
                return;
              };
              if n == 0 {
                break;
              }
              buf.extend_from_slice(&tmp[..n]);
            }
            let body_str = String::from_utf8_lossy(
              &buf[header_end..header_end + content_len],
            )
            .to_string();

            let (status, extra_headers, resp_body) =
              handler(req_line, head_str, body_str);
            let mut resp_head = format!(
              "HTTP/1.1 {status} Mock\r\nContent-Length: {}\r\nConnection: close\r\n",
              resp_body.len()
            );
            for (hk, hv) in extra_headers {
              resp_head.push_str(&format!("{hk}: {hv}\r\n"));
            }
            resp_head.push_str("\r\n");
            let _ = tls.write_all(resp_head.as_bytes()).await;
            let _ = tls.write_all(&resp_body).await;
            let _ = tls.shutdown().await;
            break;
          }
        });
      }
    });

    (format!("https://127.0.0.1:{}", addr.port()), join)
  }

  fn make_provider(
    id: &str,
    slug: &str,
    endpoint: &str,
    ca_cert_pem: &str,
  ) -> SecretProvider {
    SecretProvider {
      id: id.to_string(),
      slug: slug.to_string(),
      name: slug.to_string(),
      description: String::new(),
      enabled: true,
      revision: 1,
      credential_revision: 1,
      has_credentials: true,
      updated_at: 0,
      config: SecretProviderConfig::Infisical(
        InfisicalProviderConfig {
          endpoint: endpoint.to_string(),
          client_id: "cid-123".to_string(),
          organization_slug: "org-abc".to_string(),
          ca_cert_pem: ca_cert_pem.to_string(),
          allow_private_ips: false,
          default_selector: Some(InfisicalSecretSelector {
            project_id: "proj-1".to_string(),
            environment: "prod".to_string(),
            secret_path: "/backend/api".to_string(),
            include_imports: true,
            expand_references: false,
          }),
        },
      ),
    }
  }

  #[tokio::test]
  async fn infisical_v4_contract_singleflight_merge_literal_and_security_controls()
   {
    set_test_allow_loopback(true);
    let pki = generate_test_pki();

    let login_calls = Arc::new(AtomicUsize::new(0));
    let secret_calls = Arc::new(AtomicUsize::new(0));
    let login_calls_c = login_calls.clone();
    let secret_calls_c = secret_calls.clone();

    let (endpoint, server_handle) =
      spawn_mock_https_server(&pki, move |req_line, headers, body| {
        if req_line.starts_with("POST /api/v1/auth/universal-auth/login ")
        {
          let n = login_calls_c.fetch_add(1, Ordering::SeqCst) + 1;
          let v: serde_json::Value =
            serde_json::from_str(&body).unwrap();
          assert_eq!(v["clientId"], "cid-123");
          assert_eq!(v["clientSecret"], "csec-xyz");
          assert_eq!(v["organizationSlug"], "org-abc");
          let tok = format!("tok-gen-{n}");
          let resp = json!({
            "accessToken": tok,
            "expiresIn": 7200,
            "accessTokenMaxTTL": 86400,
            "tokenType": "Bearer"
          });
          return (
            200,
            vec![("Content-Type".into(), "application/json".into())],
            serde_json::to_vec(&resp).unwrap(),
          );
        }

        if req_line.starts_with("GET /api/v4/secrets?") {
          let call_idx =
            secret_calls_c.fetch_add(1, Ordering::SeqCst) + 1;
          // Verify exact v4 query parameters!
          assert!(req_line.contains("projectId=proj-1"));
          assert!(req_line.contains("environment=prod"));
          assert!(req_line.contains("secretPath=%2Fbackend%2Fapi"));
          assert!(req_line.contains("includeImports=true"));
          assert!(req_line.contains("expandSecretReferences=false"));
          assert!(req_line.contains("recursive=false"));
          assert!(
            req_line.contains("includePersonalOverrides=false")
          );
          assert!(
            headers
              .to_ascii_lowercase()
              .contains("authorization: bearer tok-gen-")
          );

          // Simulate transient 429 on call #12 and 401 token expiry on call #14
          if call_idx == 12 {
            return (
              429,
              vec![("Retry-After".into(), "0".into())],
              b"{\"message\":\"rate limited\"}".to_vec(),
            );
          }
          if call_idx == 14
            && headers
              .to_ascii_lowercase()
              .contains("authorization: bearer tok-gen-1")
          {
            return (
              401,
              vec![],
              b"{\"message\":\"token expired\"}".to_vec(),
            );
          }

          let view_val = req_line.contains("viewSecretValue=true");
          let resp = if view_val {
            json!({
              "imports": [
                {
                  "secretPath": "/shared",
                  "environment": "prod",
                  "secrets": [
                    { "secretKey": "OVERRIDE_ME", "secretValue": "from-import-0-first", "secretValueHidden": false, "version": 1 },
                    { "secretKey": "OVERRIDE_ME", "secretValue": "from-import-0-duplicate-ignored", "secretValueHidden": false, "version": 2 },
                    { "secretKey": "IMPORT_ONLY", "secretValue": "imp-0", "secretValueHidden": false, "version": 3 }
                  ]
                },
                {
                  "secretPath": "/tier2",
                  "environment": "prod",
                  "secrets": [
                    { "secretKey": "IMPORT_ONLY", "secretValue": "imp-1-wins-over-imp-0", "secretValueHidden": false, "version": 4 },
                    { "secretKey": "OVERRIDE_ME", "secretValue": "from-import-1", "secretValueHidden": false, "version": 5 }
                  ]
                }
              ],
              "secrets": [
                {
                  "secretKey": "OVERRIDE_ME",
                  "secretValue": "local-top-level-wins",
                  "secretValueHidden": false,
                  "version": 9
                },
                {
                  "secretKey": "DATABASE_URL",
                  "secretValue": "postgres://user:p%40ss%23word%2F123@db:5432/app",
                  "secretValueHidden": false,
                  "version": 7
                },
                {
                  "secretKey": "app.db.password",
                  "secretValue": "p@ss#word/123 [[LOCAL_VAR]] ${UNSET} \"quote\" \\slash\nline2",
                  "secretValueHidden": false,
                  "version": 2
                },
                {
                  "secretKey": "EMPTY_KEY",
                  "secretValue": "",
                  "secretValueHidden": false,
                  "version": 1
                }
              ]
            })
          } else {
            json!({
              "imports": [],
              "secrets": [
                {
                  "secretKey": "DATABASE_URL",
                  "secretValue": "<hidden-by-infisical>",
                  "secretValueHidden": true,
                  "version": 7,
                  "secretComment": "Pre-encoded DB URL"
                }
              ]
            })
          };
          return (
            200,
            vec![("Content-Type".into(), "application/json".into())],
            serde_json::to_vec(&resp).unwrap(),
          );
        }

        (404, vec![], b"not found".to_vec())
      })
      .await;

    let cred = SecretProviderCredentialInput::Infisical {
      client_secret: "csec-xyz".to_string(),
    };
    let cancel = CancellationToken::new();

    // 1. Untrusted CA certificate MUST fail closed!
    let bad_provider = make_provider(
      "prov-bad-ca",
      "infisical-bad-ca",
      &endpoint,
      &pki.untrusted_ca_cert_pem,
    );
    let bad_res = INFISICAL_ADAPTER
      .test_connection(&bad_provider, &cred, None, None, &cancel)
      .await
      .unwrap();
    assert!(!bad_res.ready_for_deploy);
    assert_eq!(
      bad_res.authentication.status,
      SecretCheckStageStatus::Failed
    );

    // 2. Singleflight concurrency test: 10 concurrent resolve_batch calls -> exactly 1 login!
    let provider = make_provider(
      "prov-sf-1",
      "infisical-prod",
      &endpoint,
      &pki.ca_cert_pem,
    );
    let sel = SecretSelector::Infisical(
      InfisicalAdapter::extract_config(&provider.config)
        .default_selector
        .clone()
        .unwrap(),
    );
    let items = vec![
      AdapterResolveItemRequest {
        remote_key: "DATABASE_URL".into(),
        format: SecretValueFormat::Text,
        json_field: String::new(),
        required: true,
        allow_empty: false,
      },
      AdapterResolveItemRequest {
        remote_key: "app.db.password".into(),
        format: SecretValueFormat::Text,
        json_field: String::new(),
        required: true,
        allow_empty: false,
      },
      AdapterResolveItemRequest {
        remote_key: "OVERRIDE_ME".into(),
        format: SecretValueFormat::Text,
        json_field: String::new(),
        required: true,
        allow_empty: false,
      },
      AdapterResolveItemRequest {
        remote_key: "IMPORT_ONLY".into(),
        format: SecretValueFormat::Text,
        json_field: String::new(),
        required: true,
        allow_empty: false,
      },
      AdapterResolveItemRequest {
        remote_key: "OPTIONAL_MISSING".into(),
        format: SecretValueFormat::Text,
        json_field: String::new(),
        required: false,
        allow_empty: true,
      },
    ];

    let mut handles = Vec::new();
    for _ in 0..10 {
      let p = provider.clone();
      let c = cred.clone();
      let s = sel.clone();
      let it = items.clone();
      let tok = cancel.clone();
      handles.push(tokio::spawn(async move {
        INFISICAL_ADAPTER
          .resolve_batch(&p, &c, &s, &it, &tok)
          .await
          .unwrap()
      }));
    }

    for h in handles {
      let resolved = h.await.unwrap();
      assert_eq!(
        resolved["DATABASE_URL"]
          .as_ref()
          .unwrap()
          .as_text()
          .unwrap(),
        "postgres://user:p%40ss%23word%2F123@db:5432/app"
      );
      assert_eq!(
        resolved["app.db.password"]
          .as_ref()
          .unwrap()
          .as_text()
          .unwrap(),
        "p@ss#word/123 [[LOCAL_VAR]] ${UNSET} \"quote\" \\slash\nline2"
      );
      assert_eq!(
        resolved["OVERRIDE_ME"].as_ref().unwrap().as_text().unwrap(),
        "local-top-level-wins"
      );
      assert_eq!(
        resolved["IMPORT_ONLY"].as_ref().unwrap().as_text().unwrap(),
        "imp-1-wins-over-imp-0"
      );
      assert!(resolved["OPTIONAL_MISSING"].is_none());
    }
    assert_eq!(
      login_calls.load(Ordering::SeqCst),
      1,
      "10 concurrent batches must share a single Universal Auth login"
    );

    // 3. Metadata listing (`viewSecretValue=false`) returns metadata only (call #11)
    let meta = INFISICAL_ADAPTER
      .list_metadata(&provider, &cred, &sel, &cancel)
      .await
      .unwrap();
    assert_eq!(meta.len(), 1);
    assert_eq!(meta[0].key, "DATABASE_URL");
    assert_eq!(meta[0].comment, "Pre-encoded DB URL");

    // 4. Call #12 hits 429 Retry-After -> retries as call #13; next call hits 401 -> refreshes token once!
    let _ = INFISICAL_ADAPTER
      .resolve_batch(&provider, &cred, &sel, &items[..1], &cancel)
      .await
      .unwrap();
    let _ = INFISICAL_ADAPTER
      .resolve_batch(&provider, &cred, &sel, &items[..1], &cancel)
      .await
      .unwrap();
    assert_eq!(
      login_calls.load(Ordering::SeqCst),
      2,
      "401 must trigger exactly 1 token refresh"
    );

    // 5. allow_empty=false rejects empty secret; format=Binary & json_field fail closed
    assert!(
      INFISICAL_ADAPTER
        .resolve_batch(
          &provider,
          &cred,
          &sel,
          &[AdapterResolveItemRequest {
            remote_key: "EMPTY_KEY".into(),
            format: SecretValueFormat::Text,
            json_field: String::new(),
            required: true,
            allow_empty: false,
          }],
          &cancel
        )
        .await
        .is_err()
    );
    assert!(
      INFISICAL_ADAPTER
        .resolve_batch(
          &provider,
          &cred,
          &sel,
          &[AdapterResolveItemRequest {
            remote_key: "DATABASE_URL".into(),
            format: SecretValueFormat::Binary,
            json_field: String::new(),
            required: true,
            allow_empty: true,
          }],
          &cancel
        )
        .await
        .is_err()
    );

    invalidate_infisical_provider_cache("prov-sf-1");
    server_handle.abort();
  }

  #[tokio::test]
  async fn rejects_redirects_oversized_responses_403_and_hidden_values()
   {
    set_test_allow_loopback(true);
    let pki = generate_test_pki();

    // Server A: returns 302 redirect on login -> must fail closed!
    let (redir_ep, h1) = spawn_mock_https_server(&pki, |_, _, _| {
      (
        302,
        vec![(
          "Location".into(),
          "https://169.254.169.254/latest/meta-data".into(),
        )],
        Vec::new(),
      )
    })
    .await;

    let cred = SecretProviderCredentialInput::Infisical {
      client_secret: "csec".into(),
    };
    let cancel = CancellationToken::new();
    let p_redir = make_provider(
      "prov-redir",
      "infisical-redir",
      &redir_ep,
      &pki.ca_cert_pem,
    );
    let res_redir = INFISICAL_ADAPTER
      .test_connection(&p_redir, &cred, None, None, &cancel)
      .await
      .unwrap();
    assert!(!res_redir.ready_for_deploy);
    assert!(
      res_redir.authentication.message.contains("redirect"),
      "got: {}",
      res_redir.authentication.message
    );
    h1.abort();

    // Server B: returns >4 MiB response body -> must fail closed!
    let (huge_ep, h2) = spawn_mock_https_server(&pki, |_, _, _| {
      (200, vec![], vec![b'a'; MAX_SECRET_RESPONSE_BYTES + 1024])
    })
    .await;
    let p_huge = make_provider(
      "prov-huge",
      "infisical-huge",
      &huge_ep,
      &pki.ca_cert_pem,
    );
    let res_huge = INFISICAL_ADAPTER
      .test_connection(&p_huge, &cred, None, None, &cancel)
      .await
      .unwrap();
    assert!(!res_huge.ready_for_deploy);
    assert!(
      res_huge
        .authentication
        .message
        .contains("exceeds maximum allowed size")
    );
    h2.abort();

    // Server C: login succeeds, metadata succeeds, but ReadValue returns secretValueHidden=true!
    let secret_hits = Arc::new(AtomicUsize::new(0));
    let secret_hits_c = secret_hits.clone();
    let (hidden_ep, h3) =
      spawn_mock_https_server(&pki, move |req_line, _, _| {
        if req_line
          .starts_with("POST /api/v1/auth/universal-auth/login ")
        {
          return (
            200,
            vec![],
            serde_json::to_vec(
              &json!({ "accessToken": "t1", "expiresIn": 3600 }),
            )
            .unwrap(),
          );
        }
        secret_hits_c.fetch_add(1, Ordering::SeqCst);
        if req_line.contains("projectId=proj-403") {
          return (
            403,
            vec![],
            b"{\"message\":\"Forbidden\"}".to_vec(),
          );
        }
        (
          200,
          vec![],
          serde_json::to_vec(&json!({
            "secrets": [{
              "secretKey": "DB_PASSWORD",
              "secretValue": "<hidden-by-infisical>",
              "secretValueHidden": true,
              "version": 1
            }],
            "imports": []
          }))
          .unwrap(),
        )
      })
      .await;

    let p_hidden = make_provider(
      "prov-hidden",
      "infisical-hidden",
      &hidden_ep,
      &pki.ca_cert_pem,
    );
    // 3-stage test_connection must report Auth=Passed, Metadata=Passed, ValueRead=Failed, ready_for_deploy=false!
    let diag = INFISICAL_ADAPTER
      .test_connection(&p_hidden, &cred, None, None, &cancel)
      .await
      .unwrap();
    assert!(!diag.ready_for_deploy);
    assert_eq!(
      diag.authentication.status,
      SecretCheckStageStatus::Passed
    );
    assert_eq!(
      diag.metadata_access.status,
      SecretCheckStageStatus::Passed
    );
    assert_eq!(
      diag.value_read_access.status,
      SecretCheckStageStatus::Failed
    );

    // resolve_batch must fail closed and NEVER return "<hidden-by-infisical>"!
    let sel = SecretSelector::Infisical(
      InfisicalAdapter::extract_config(&p_hidden.config)
        .default_selector
        .clone()
        .unwrap(),
    );
    let err = INFISICAL_ADAPTER
      .resolve_batch(
        &p_hidden,
        &cred,
        &sel,
        &[AdapterResolveItemRequest {
          remote_key: "DB_PASSWORD".into(),
          format: SecretValueFormat::Text,
          json_field: String::new(),
          required: true,
          allow_empty: true,
        }],
        &cancel,
      )
      .await
      .unwrap_err();
    assert!(format!("{err:#}").contains("secretValueHidden=true"));

    // 403 must NOT retry!
    let before_403 = secret_hits.load(Ordering::SeqCst);
    let sel_403 =
      SecretSelector::Infisical(InfisicalSecretSelector {
        project_id: "proj-403".into(),
        environment: "prod".into(),
        secret_path: "/".into(),
        include_imports: false,
        expand_references: false,
      });
    assert!(
      INFISICAL_ADAPTER
        .resolve_batch(
          &p_hidden,
          &cred,
          &sel_403,
          &[AdapterResolveItemRequest {
            remote_key: "DB_PASSWORD".into(),
            format: SecretValueFormat::Text,
            json_field: String::new(),
            required: true,
            allow_empty: true,
          }],
          &cancel,
        )
        .await
        .is_err()
    );
    assert_eq!(
      secret_hits.load(Ordering::SeqCst),
      before_403 + 1,
      "403 Forbidden must fail immediately without retry"
    );

    invalidate_infisical_provider_cache("prov-hidden");
    h3.abort();
  }
}
