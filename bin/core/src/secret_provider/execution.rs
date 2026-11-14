use std::{
  collections::{BTreeSet, HashSet},
  fmt::Write as _,
  str::FromStr,
};

use anyhow::{Context, bail};
use database::mungos::mongodb::bson::{
  doc, oid::ObjectId, to_document,
};
use komodo_client::entities::{
  FileContents,
  deployment::Deployment,
  komodo_timestamp,
  secret_provider::{
    DeployedSecretProvenance, SecretDeploymentMetadata,
  },
  stack::{Stack, StackRemoteFileContents},
  update::Log,
};
use periphery_client::api::{
  ExternalEnvVar,
  compose::{ExternalFileSecretBinding, ExternalServiceEnvBinding},
};
use rand::RngExt;

use crate::state::db_client;

use super::resolver::{
  ResolvedDeploymentSecrets, ResolvedStackSecrets,
  finalize_secret_replacers, stack_uses_external_secrets,
};

/// Materialized external secret fields sent over confidential RPC to Periphery for a Stack execution.
#[derive(Debug, Clone, Default)]
pub struct PeripheryStackSecretPayload {
  pub external_env_vars: Vec<ExternalEnvVar>,
  pub service_env_bindings: Vec<ExternalServiceEnvBinding>,
  pub file_secret_bindings: Vec<ExternalFileSecretBinding>,
  pub secret_env_keys: Vec<String>,
}

impl From<&ResolvedStackSecrets> for PeripheryStackSecretPayload {
  fn from(resolved: &ResolvedStackSecrets) -> Self {
    Self {
      external_env_vars: resolved
        .external_compose_env
        .iter()
        .cloned()
        .map(ExternalEnvVar::from)
        .collect(),
      service_env_bindings: resolved
        .service_env_bindings
        .iter()
        .map(|b| ExternalServiceEnvBinding {
          variable: b.variable.clone(),
          value: b.value.clone(),
          services: b.services.clone(),
        })
        .collect(),
      file_secret_bindings: resolved
        .file_secret_bindings
        .iter()
        .map(|f| ExternalFileSecretBinding {
          secret_name: f.secret_name.clone(),
          bytes: f.bytes.clone(),
          services: f.services.clone(),
          target_path: f.target_path.clone(),
          uid: f.uid.clone(),
          gid: f.gid.clone(),
          mode: f.mode,
        })
        .collect(),
      secret_env_keys: resolved.secret_env_keys.clone(),
    }
  }
}

/// Build a strictly non-sensitive `Update` log entry recording which external secret references
/// were resolved and their versions (never logs values, URLs, credentials, or ciphertexts).
pub fn build_external_secrets_log(
  provenance: &[DeployedSecretProvenance],
) -> Log {
  let mut msg = format!(
    "Resolved {} external secret binding(s):",
    provenance.len()
  );
  for item in provenance {
    match item.version {
      Some(ver) => {
        let _ = write!(
          msg,
          "\n  - {} <- vault.{}.{} (version {})",
          item.target, item.provider_slug, item.remote_key, ver
        );
      }
      None => {
        let _ = write!(
          msg,
          "\n  - {} <- vault.{}.{}",
          item.target, item.provider_slug, item.remote_key
        );
      }
    }
  }
  Log::simple("Interpolate External Secrets", msg)
}

/// Combine Komodo `Interpolator.secret_replacers` with external secret replacers in deterministic
/// length-descending order so longer mixed strings (e.g. `DATABASE_URL`) are sanitized before shorter substrings.
pub fn merge_secret_replacers(
  local: HashSet<(String, String)>,
  external: Vec<(String, String)>,
) -> Vec<(String, String)> {
  finalize_secret_replacers(local.into_iter().chain(external))
}

/// Generate a fresh 128-bit random hex nonce for `--build-arg KOMODO_SECRET_CACHE_NONCE=<nonce>`
/// when `build.config.secret_cache_bust = true`.
pub fn generate_build_cache_bust_nonce() -> String {
  let mut bytes = [0u8; 16];
  rand::rng().fill(&mut bytes);
  hex::encode(bytes)
}

/// Fetch `SecretDeploymentMetadata` for `(resource_type, resource_id)` if present.
pub async fn get_resource_deployment_metadata(
  resource_type: &str,
  resource_id: &str,
) -> anyhow::Result<Option<SecretDeploymentMetadata>> {
  db_client()
    .secret_deployment_metadata
    .find_one(doc! {
      "resource_type": resource_type,
      "resource_id": resource_id,
    })
    .await
    .context("Failed to query SecretDeploymentMetadata")
}

/// Delete `SecretDeploymentMetadata` for `(resource_type, resource_id)`.
pub async fn delete_resource_deployment_metadata(
  resource_type: &str,
  resource_id: &str,
) -> anyhow::Result<()> {
  db_client()
    .secret_deployment_metadata
    .delete_many(doc! {
      "resource_type": resource_type,
      "resource_id": resource_id,
    })
    .await
    .context("Failed to delete SecretDeploymentMetadata")?;
  Ok(())
}

async fn upsert_deployment_metadata_doc(
  mut meta: SecretDeploymentMetadata,
) -> anyhow::Result<()> {
  let db = db_client();
  if let Some(existing) = db
    .secret_deployment_metadata
    .find_one(doc! {
      "resource_type": &meta.resource_type,
      "resource_id": &meta.resource_id,
    })
    .await
    .context("Failed to query SecretDeploymentMetadata")?
  {
    meta.id = existing.id.clone();
    let oid = ObjectId::from_str(&existing.id)
      .context("Invalid ObjectId on SecretDeploymentMetadata")?;
    let mut doc_body = to_document(&meta)
      .context("Failed to serialize SecretDeploymentMetadata")?;
    doc_body.remove("_id");
    db.secret_deployment_metadata
      .update_one(doc! { "_id": oid }, doc! { "$set": doc_body })
      .await
      .context("Failed to update SecretDeploymentMetadata")?;
  } else {
    db.secret_deployment_metadata
      .insert_one(meta)
      .await
      .context("Failed to insert SecretDeploymentMetadata")?;
  }
  Ok(())
}

/// Upsert non-sensitive `SecretDeploymentMetadata` after a successful Stack deploy.
pub async fn upsert_stack_deployment_metadata(
  stack: &Stack,
  server_id: &str,
  resolved: &ResolvedStackSecrets,
  active_generation: Option<String>,
) -> anyhow::Result<()> {
  let meta = SecretDeploymentMetadata {
    id: String::new(),
    resource_type: String::from("Stack"),
    resource_id: stack.id.clone(),
    server_id: server_id.to_string(),
    runtime_project_or_container: stack.project_name(true),
    secret_env_keys: resolved.secret_env_keys.clone(),
    file_secret_names: resolved.file_secret_names.clone(),
    active_generation,
    keyed_fingerprint_hmac: resolved.keyed_fingerprint_hmac.clone(),
    fingerprint_key_id: resolved.fingerprint_key_id.clone(),
    provenance: resolved.provenance.clone(),
    deployed_at: komodo_timestamp(),
  };
  upsert_deployment_metadata_doc(meta).await
}

/// Upsert non-sensitive `SecretDeploymentMetadata` after a successful Deployment container run.
pub async fn upsert_deployment_metadata(
  deployment: &Deployment,
  server_id: &str,
  deployed_container_name: &str,
  resolved: &ResolvedDeploymentSecrets,
) -> anyhow::Result<()> {
  let meta = SecretDeploymentMetadata {
    id: String::new(),
    resource_type: String::from("Deployment"),
    resource_id: deployment.id.clone(),
    server_id: server_id.to_string(),
    runtime_project_or_container: deployed_container_name
      .trim_start_matches('/')
      .to_string(),
    secret_env_keys: resolved.secret_env_keys.clone(),
    file_secret_names: Vec::new(),
    active_generation: None,
    keyed_fingerprint_hmac: resolved.keyed_fingerprint_hmac.clone(),
    fingerprint_key_id: resolved.fingerprint_key_id.clone(),
    provenance: resolved.provenance.clone(),
    deployed_at: komodo_timestamp(),
  };
  upsert_deployment_metadata_doc(meta).await
}

pub fn normalize_relative_stack_path(path: &str) -> String {
  path
    .trim()
    .trim_start_matches("./")
    .trim_end_matches('/')
    .to_string()
}

/// Compute relative file paths in a Stack's run directory that hold resolved external secrets
/// and must NEVER be persisted into `StackInfo` (`deployed_contents` / `remote_contents`)
/// or overwritten via `WriteStackFileContents`.
pub async fn excluded_secret_paths_for_stack(
  stack: &Stack,
) -> Vec<String> {
  let mut set = BTreeSet::new();
  set.insert(String::from(".komodo-secrets"));

  let uses_ext = stack_uses_external_secrets(stack).unwrap_or(true);
  let has_deployed_meta = if uses_ext {
    true
  } else {
    get_resource_deployment_metadata("Stack", &stack.id)
      .await
      .ok()
      .flatten()
      .is_some()
  };

  if has_deployed_meta {
    set.insert(String::from(".env"));
    let custom_env =
      normalize_relative_stack_path(&stack.config.env_file_path);
    if !custom_env.is_empty() {
      set.insert(custom_env);
    }
  }

  set.into_iter().collect()
}

pub fn is_excluded_secret_stack_path(
  path: &str,
  excluded: &[String],
) -> bool {
  let norm = normalize_relative_stack_path(path);
  if norm.is_empty() {
    return false;
  }
  excluded.iter().any(|ex| {
    let ex_norm = normalize_relative_stack_path(ex);
    !ex_norm.is_empty()
      && (norm == ex_norm || norm.starts_with(&format!("{ex_norm}/")))
  })
}

pub fn filter_stack_remote_contents(
  contents: &mut Vec<StackRemoteFileContents>,
  errors: &mut Vec<FileContents>,
  excluded: &[String],
) {
  if excluded.is_empty() {
    return;
  }
  contents.retain(|item| {
    !is_excluded_secret_stack_path(&item.path, excluded)
  });
  errors.retain(|item| {
    !is_excluded_secret_stack_path(&item.path, excluded)
  });
}

/// Prevent `WriteStackFileContents` from writing to managed runtime secret paths
/// or committing/overwriting the Komodo-generated secret `.env` file.
pub async fn ensure_writable_stack_file_path(
  stack: &Stack,
  file_path: &str,
) -> anyhow::Result<()> {
  let excluded = excluded_secret_paths_for_stack(stack).await;
  if is_excluded_secret_stack_path(file_path, &excluded) {
    bail!(
      "Refusing to write Stack file '{}': path is reserved for Komodo-managed runtime external secrets",
      file_path
    );
  }
  Ok(())
}
