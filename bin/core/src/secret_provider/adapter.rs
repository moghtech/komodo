use std::{future::Future, pin::Pin};

use anyhow::Context;
use indexmap::IndexMap;
use komodo_client::entities::secret_provider::{
  ExternalSecretMetadataItem, SecretProvider,
  SecretProviderCapabilities, SecretProviderConfig,
  SecretProviderCredentialInput, SecretProviderKind, SecretSelector,
  SecretValueFormat, TestSecretProviderResponse,
};
use tokio_util::sync::CancellationToken;

use super::infisical::INFISICAL_ADAPTER;

pub type AdapterFuture<'a, T> =
  Pin<Box<dyn Future<Output = anyhow::Result<T>> + Send + 'a>>;

/// Resolved external secret value held exclusively in memory during a single Core execution.
/// Custom `Debug` ensures raw secret bytes/strings are never emitted to debug logs.
#[derive(Clone, PartialEq, Eq)]
pub struct ResolvedSecretValue {
  pub bytes: Vec<u8>,
  pub format: SecretValueFormat,
  pub version: Option<i64>,
  /// Reserved extension point for future dynamic lease-backed secret providers.
  pub lease_ttl_seconds: Option<u64>,
}

impl std::fmt::Debug for ResolvedSecretValue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ResolvedSecretValue")
      .field("bytes_len", &self.bytes.len())
      .field("format", &self.format)
      .field("version", &self.version)
      .field("lease_ttl_seconds", &self.lease_ttl_seconds)
      .finish()
  }
}

impl ResolvedSecretValue {
  pub fn from_text(value: String, version: Option<i64>) -> Self {
    Self {
      bytes: value.into_bytes(),
      format: SecretValueFormat::Text,
      version,
      lease_ttl_seconds: None,
    }
  }

  /// View the resolved secret as a literal UTF-8 string without modifying or trimming any byte.
  pub fn as_text(&self) -> anyhow::Result<&str> {
    std::str::from_utf8(&self.bytes).context(
      "Resolved external secret value is not valid UTF-8 and cannot be injected as text",
    )
  }
}

/// Provider-agnostic per-key resolution request within a single selector batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterResolveItemRequest {
  pub remote_key: String,
  pub format: SecretValueFormat,
  pub json_field: String,
  pub required: bool,
  pub allow_empty: bool,
}

/// Provider-agnostic adapter trait.
/// Does NOT assume every future provider uses Infisical's `project_id`/`environment`/`secret_path` model
/// or that all secret values are environment-variable strings.
pub trait SecretProviderAdapter: Send + Sync {
  fn kind(&self) -> SecretProviderKind;

  fn capabilities(&self) -> SecretProviderCapabilities;

  fn validate_config(
    &self,
    config: &SecretProviderConfig,
  ) -> anyhow::Result<()>;

  fn validate_credential(
    &self,
    credential: &SecretProviderCredentialInput,
  ) -> anyhow::Result<()>;

  fn validate_selector(
    &self,
    config: &SecretProviderConfig,
    selector: &SecretSelector,
  ) -> anyhow::Result<()>;

  fn test_connection<'a>(
    &'a self,
    provider: &'a SecretProvider,
    credential: &'a SecretProviderCredentialInput,
    selector_override: Option<&'a SecretSelector>,
    cancel: &'a CancellationToken,
  ) -> AdapterFuture<'a, TestSecretProviderResponse>;

  fn list_metadata<'a>(
    &'a self,
    provider: &'a SecretProvider,
    credential: &'a SecretProviderCredentialInput,
    selector: &'a SecretSelector,
    cancel: &'a CancellationToken,
  ) -> AdapterFuture<'a, Vec<ExternalSecretMetadataItem>>;

  fn resolve_batch<'a>(
    &'a self,
    provider: &'a SecretProvider,
    credential: &'a SecretProviderCredentialInput,
    selector: &'a SecretSelector,
    items: &'a [AdapterResolveItemRequest],
    cancel: &'a CancellationToken,
  ) -> AdapterFuture<'a, IndexMap<String, Option<ResolvedSecretValue>>>;
}

/// Look up the registered adapter implementation for `kind`.
pub fn get_adapter(
  kind: SecretProviderKind,
) -> &'static dyn SecretProviderAdapter {
  match kind {
    SecretProviderKind::Infisical => &INFISICAL_ADAPTER,
  }
}
