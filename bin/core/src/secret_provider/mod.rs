#![allow(dead_code)]

use std::str::FromStr;

use anyhow::{Context, bail};
use database::mungos::{
  find::find_collect,
  mongodb::bson::{Document, doc, oid::ObjectId, to_document},
};
use komodo_client::entities::secret_provider::{
  RotateSecretProviderEncryptionKeysResponse, SecretProvider,
  SecretProviderCredentialInput, SecretProviderKind,
};

use crate::state::db_client;

pub mod adapter;
pub mod authz;
pub mod crypto;
pub mod http;
pub mod infisical;
pub mod resolver;

#[cfg(test)]
mod live_infisical_test;

use crypto::SecretProviderKeyring;

/// Evict any cached auth tokens and HTTP connection pools associated with `provider_id`.
pub fn invalidate_provider_runtime_caches(provider_id: &str) {
  infisical::invalidate_infisical_provider_cache(provider_id);
}

/// Store or replace the encrypted bootstrap credential envelope for `provider`.
pub async fn save_provider_credential(
  provider_id: &str,
  kind: SecretProviderKind,
  credential_revision: i64,
  credential: &SecretProviderCredentialInput,
) -> anyhow::Result<()> {
  let keyring = SecretProviderKeyring::from_core_config()?;
  let mut envelope = keyring.encrypt_credential(
    provider_id,
    kind,
    credential_revision,
    credential,
  )?;

  let db = db_client();
  if let Some(existing) = db
    .secret_provider_credentials
    .find_one(doc! { "provider_id": provider_id })
    .await
    .context("Failed to query SecretProviderCredential collection")?
  {
    envelope.id = existing.id.clone();
    let oid = ObjectId::from_str(&existing.id).context(
      "Invalid ObjectId on StoredSecretProviderCredential",
    )?;
    let mut doc_body = to_document(&envelope)
      .context("Failed to serialize credential envelope")?;
    doc_body.remove("_id");
    db.secret_provider_credentials
      .update_one(doc! { "_id": oid }, doc! { "$set": doc_body })
      .await
      .context("Failed to update SecretProviderCredential")?;
  } else {
    db.secret_provider_credentials
      .insert_one(envelope)
      .await
      .context("Failed to insert SecretProviderCredential")?;
  }
  Ok(())
}

/// Load and decrypt the bootstrap credential envelope for `provider`, verifying AAD against `provider`.
pub async fn load_provider_credential(
  provider: &SecretProvider,
) -> anyhow::Result<SecretProviderCredentialInput> {
  if !provider.has_credentials {
    bail!(
      "Secret provider '{}' ({}) has no stored credentials",
      provider.name,
      provider.slug
    );
  }
  let keyring = SecretProviderKeyring::from_core_config()?;
  let envelope = db_client()
    .secret_provider_credentials
    .find_one(doc! { "provider_id": &provider.id })
    .await
    .context("Failed to query SecretProviderCredential collection")?
    .with_context(|| {
      format!(
        "Stored credential envelope missing in database for provider '{}' ({})",
        provider.name, provider.slug
      )
    })?;
  let kind: SecretProviderKind = (&provider.config).into();
  keyring.decrypt_credential(
    &provider.id,
    kind,
    provider.credential_revision,
    &envelope,
  )
}

/// Delete any stored credential envelope for `provider_id`.
pub async fn delete_provider_credential(
  provider_id: &str,
) -> anyhow::Result<()> {
  invalidate_provider_runtime_caches(provider_id);
  db_client()
    .secret_provider_credentials
    .delete_many(doc! { "provider_id": provider_id })
    .await
    .context("Failed to delete SecretProviderCredential")?;
  Ok(())
}

/// Re-encrypt all stored provider credential envelopes under the active master key in the keyring.
pub async fn rotate_all_provider_credentials()
-> anyhow::Result<RotateSecretProviderEncryptionKeysResponse> {
  let keyring = SecretProviderKeyring::from_core_config()?;
  let db = db_client();
  let providers =
    find_collect(&db.secret_providers, Document::new(), None)
      .await
      .context("Failed to list SecretProviders for key rotation")?;

  let mut rotated_count = 0usize;
  for provider in providers {
    if !provider.has_credentials {
      continue;
    }
    let envelope = db
      .secret_provider_credentials
      .find_one(doc! { "provider_id": &provider.id })
      .await
      .context("Failed to query SecretProviderCredential")?
      .with_context(|| {
        format!(
          "Missing credential envelope for provider '{}' ({})",
          provider.name, provider.slug
        )
      })?;
    let kind: SecretProviderKind = (&provider.config).into();
    if let Some(rotated) = keyring.rotate_envelope_if_needed(
      &provider.id,
      kind,
      provider.credential_revision,
      &envelope,
    )? {
      let oid = ObjectId::from_str(&envelope.id).context(
        "Invalid ObjectId on StoredSecretProviderCredential",
      )?;
      let mut doc_body = to_document(&rotated)
        .context("Failed to serialize rotated credential envelope")?;
      doc_body.remove("_id");
      db.secret_provider_credentials
        .update_one(doc! { "_id": oid }, doc! { "$set": doc_body })
        .await
        .context(
          "Failed to persist rotated SecretProviderCredential",
        )?;
      invalidate_provider_runtime_caches(&provider.id);
      rotated_count += 1;
    }
  }

  Ok(RotateSecretProviderEncryptionKeysResponse {
    rotated_count,
    active_key_id: keyring.active_key_id().to_string(),
  })
}

/// Run on Core startup: clean up any orphaned credential envelopes/grants and warn if
/// secret providers exist in MongoDB while `KOMODO_SECRET_PROVIDER_ENCRYPTION_KEY` is missing or invalid.
pub async fn verify_startup_integrity() {
  let db = db_client();
  let providers =
    match find_collect(&db.secret_providers, Document::new(), None)
      .await
    {
      Ok(p) => p,
      Err(e) => {
        warn!(
          "Failed to inspect SecretProviders during startup | {e:#}"
        );
        return;
      }
    };

  if providers.is_empty() {
    return;
  }

  if providers.iter().any(|p| p.has_credentials) {
    if let Err(e) = SecretProviderKeyring::from_core_config() {
      error!(
        "SecretProviders with stored credentials exist in MongoDB, but secret_provider_encryption_key is missing or invalid ({e:#}). External secret providers will refuse execution until a valid key is configured."
      );
    }
  }
}
