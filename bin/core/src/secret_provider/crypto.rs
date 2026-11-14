use std::{collections::HashMap, path::Path, sync::OnceLock};

use aes_gcm::{
  Aes256Gcm, KeyInit as _, Nonce,
  aead::{Aead, Payload},
};
use anyhow::{Context, anyhow, bail};
use data_encoding::{
  BASE64, BASE64_NOPAD, BASE64URL, BASE64URL_NOPAD,
};
use hmac::{Hmac, Mac};
use komodo_client::entities::{
  komodo_timestamp,
  secret_provider::{
    SecretProviderCredentialInput, SecretProviderKind,
    StoredSecretProviderCredential,
  },
};
use rand::Rng as _;
use sha2::{Digest, Sha256};

use crate::config::core_config;

pub const CREDENTIAL_SCHEMA_VERSION: u32 = 1;
pub const CREDENTIAL_ALGORITHM: &str = "AES-256-GCM";
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

type HmacSha256 = Hmac<Sha256>;

/// A single validated 32-byte master key with a stable non-sensitive `key_id`.
#[derive(Clone)]
pub struct MasterKey {
  key_id: String,
  bytes: [u8; KEY_LEN],
}

impl std::fmt::Debug for MasterKey {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("MasterKey")
      .field("key_id", &self.key_id)
      .field("bytes", &"[REDACTED]")
      .finish()
  }
}

impl MasterKey {
  pub fn key_id(&self) -> &str {
    &self.key_id
  }

  /// Parse a master key specification:
  /// - Supports `file:/path/to/key`
  /// - Supports optional explicit `<key_id>:<encoded_32_byte_key>`
  /// - Supports 64-char hex, standard Base64, or URL-safe Base64 encoding of 32 bytes
  /// - Rejects all-zero keys and any reuse of `jwt_secret`.
  pub fn parse_spec(
    raw_spec: &str,
    jwt_secret: &str,
  ) -> anyhow::Result<Self> {
    let loaded = if let Some(file_path) =
      raw_spec.trim().strip_prefix("file:")
    {
      let path = Path::new(file_path.trim());
      std::fs::read_to_string(path).with_context(|| {
        format!(
          "Failed to read secret provider encryption key file at {}",
          path.display()
        )
      })?
    } else {
      raw_spec.to_string()
    };

    let trimmed = loaded.trim();
    if trimmed.is_empty() {
      bail!(
        "Secret provider encryption key is not configured (set KOMODO_SECRET_PROVIDER_ENCRYPTION_KEY or secret_provider_encryption_key)"
      );
    }

    if !jwt_secret.trim().is_empty() && trimmed == jwt_secret.trim() {
      bail!(
        "Secret provider encryption key must NOT reuse Komodo jwt_secret"
      );
    }

    let (explicit_key_id, encoded_key) = if let Some((kid, rest)) =
      trimmed.split_once(':')
    {
      let kid = kid.trim();
      if kid.is_empty()
        || kid.len() > 64
        || !kid.chars().all(|c| {
          c.is_ascii_alphanumeric() || matches!(c, '-' | '_')
        })
      {
        bail!(
          "Invalid key_id '{kid}' in secret provider encryption key specification"
        );
      }
      (Some(kid.to_string()), rest.trim())
    } else {
      (None, trimmed)
    };

    if !jwt_secret.trim().is_empty()
      && encoded_key == jwt_secret.trim()
    {
      bail!(
        "Secret provider encryption key must NOT reuse Komodo jwt_secret"
      );
    }

    let decoded = decode_32_byte_key(encoded_key)?;

    if decoded.iter().all(|&b| b == 0) {
      bail!(
        "Secret provider encryption key must not be all-zero bytes"
      );
    }

    if !jwt_secret.trim().is_empty() {
      let jwt_bytes = jwt_secret.trim().as_bytes();
      if jwt_bytes == decoded.as_slice() {
        bail!(
          "Secret provider encryption key must NOT reuse Komodo jwt_secret bytes"
        );
      }
      let jwt_sha = Sha256::digest(jwt_bytes);
      if jwt_sha.as_slice() == decoded.as_slice() {
        bail!(
          "Secret provider encryption key must NOT be derived from Komodo jwt_secret"
        );
      }
    }

    let key_id =
      explicit_key_id.unwrap_or_else(|| derive_key_id(&decoded));

    Ok(Self {
      key_id,
      bytes: decoded,
    })
  }
}

fn decode_32_byte_key(
  encoded: &str,
) -> anyhow::Result<[u8; KEY_LEN]> {
  let bytes = if encoded.len() == 64
    && encoded.chars().all(|c| c.is_ascii_hexdigit())
  {
    hex::decode(encoded)
      .context("Failed to decode hex encryption key")?
  } else if let Ok(b) = BASE64.decode(encoded.as_bytes()) {
    b
  } else if let Ok(b) = BASE64_NOPAD.decode(encoded.as_bytes()) {
    b
  } else if let Ok(b) = BASE64URL.decode(encoded.as_bytes()) {
    b
  } else if let Ok(b) = BASE64URL_NOPAD.decode(encoded.as_bytes()) {
    b
  } else {
    bail!(
      "Secret provider encryption key must be a 32-byte key encoded as 64-char hex or Base64"
    );
  };

  if bytes.len() != KEY_LEN {
    bail!(
      "Secret provider encryption key must decode to exactly 32 bytes (got {} bytes)",
      bytes.len()
    );
  }

  let mut arr = [0u8; KEY_LEN];
  arr.copy_from_slice(&bytes);
  Ok(arr)
}

fn derive_key_id(key_bytes: &[u8; KEY_LEN]) -> String {
  let mut mac =
    <HmacSha256 as hmac::KeyInit>::new_from_slice(key_bytes)
      .expect("32-byte HMAC key is always valid");
  mac.update(b"komodo-secret-provider-key-id-v1");
  let digest = mac.finalize().into_bytes();
  format!("k1-{}", hex::encode(&digest[..6]))
}

/// Canonical Additional Authenticated Data (AAD) binding the encrypted credential
/// to its exact `schema_version`, `provider_id`, `provider_kind`, and `credential_revision`.
pub fn build_credential_aad(
  schema_version: u32,
  provider_id: &str,
  provider_kind: &str,
  credential_revision: i64,
) -> Vec<u8> {
  format!(
    "komodo-secret-provider-cred-v1|schema={schema_version}|provider_id={provider_id}|kind={provider_kind}|cred_rev={credential_revision}"
  )
  .into_bytes()
}

/// Keyring supporting an active encryption key and optional previous keys for online key rotation.
#[derive(Debug, Clone)]
pub struct SecretProviderKeyring {
  active: MasterKey,
  by_id: HashMap<String, MasterKey>,
}

impl SecretProviderKeyring {
  pub fn from_specs(
    active_spec: &str,
    previous_specs: &[String],
    jwt_secret: &str,
  ) -> anyhow::Result<Self> {
    let active = MasterKey::parse_spec(active_spec, jwt_secret)?;
    let mut by_id = HashMap::new();
    by_id.insert(active.key_id.clone(), active.clone());

    for spec in previous_specs {
      if spec.trim().is_empty() {
        continue;
      }
      let prev = MasterKey::parse_spec(spec, jwt_secret)?;
      by_id.entry(prev.key_id.clone()).or_insert(prev);
    }

    Ok(Self { active, by_id })
  }

  pub fn from_core_config() -> anyhow::Result<&'static Self> {
    static KEYRING: OnceLock<Result<SecretProviderKeyring, String>> =
      OnceLock::new();
    let res = KEYRING.get_or_init(|| {
      let cfg = core_config();
      Self::from_specs(
        &cfg.secret_provider_encryption_key,
        &cfg.secret_provider_previous_encryption_keys,
        &cfg.jwt_secret,
      )
      .map_err(|e| format!("{e:#}"))
    });
    match res {
      Ok(kr) => Ok(kr),
      Err(msg) => Err(anyhow!("{msg}")),
    }
  }

  pub fn active_key_id(&self) -> &str {
    self.active.key_id()
  }

  /// Encrypt bootstrap credentials into a versioned `StoredSecretProviderCredential` envelope.
  pub fn encrypt_credential(
    &self,
    provider_id: &str,
    kind: SecretProviderKind,
    credential_revision: i64,
    credential: &SecretProviderCredentialInput,
  ) -> anyhow::Result<StoredSecretProviderCredential> {
    if provider_id.trim().is_empty() {
      bail!("Cannot encrypt credential without provider_id");
    }
    let kind_str = kind.to_string();
    let plaintext = serde_json::to_vec(credential)
      .context("Failed to serialize credential payload")?;
    let aad = build_credential_aad(
      CREDENTIAL_SCHEMA_VERSION,
      provider_id,
      &kind_str,
      credential_revision,
    );

    let cipher = Aes256Gcm::new_from_slice(&self.active.bytes)
      .map_err(|_| anyhow!("Failed to initialize AES-256-GCM"))?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
      .encrypt(
        nonce,
        Payload {
          msg: &plaintext,
          aad: &aad,
        },
      )
      .map_err(|_| {
        anyhow!("Failed to encrypt secret provider credential")
      })?;

    Ok(StoredSecretProviderCredential {
      id: String::new(),
      provider_id: provider_id.to_string(),
      provider_kind: kind_str,
      schema_version: CREDENTIAL_SCHEMA_VERSION,
      credential_revision,
      key_id: self.active.key_id.clone(),
      algorithm: CREDENTIAL_ALGORITHM.to_string(),
      nonce: nonce_bytes.to_vec(),
      ciphertext,
      updated_at: komodo_timestamp(),
    })
  }

  /// Decrypt and authenticate a stored credential envelope, verifying AAD against expected provider metadata.
  pub fn decrypt_credential(
    &self,
    expected_provider_id: &str,
    expected_kind: SecretProviderKind,
    expected_credential_revision: i64,
    envelope: &StoredSecretProviderCredential,
  ) -> anyhow::Result<SecretProviderCredentialInput> {
    if envelope.algorithm != CREDENTIAL_ALGORITHM {
      bail!(
        "Unsupported secret provider credential algorithm '{}'",
        envelope.algorithm
      );
    }
    if envelope.schema_version != CREDENTIAL_SCHEMA_VERSION {
      bail!(
        "Unsupported secret provider credential schema_version '{}'",
        envelope.schema_version
      );
    }
    if envelope.provider_id != expected_provider_id {
      bail!(
        "Stored credential provider_id mismatch for provider '{expected_provider_id}'"
      );
    }
    let expected_kind_str = expected_kind.to_string();
    if envelope.provider_kind != expected_kind_str {
      bail!(
        "Stored credential provider_kind mismatch for provider '{expected_provider_id}'"
      );
    }
    if envelope.credential_revision != expected_credential_revision {
      bail!(
        "Stored credential revision mismatch for provider '{expected_provider_id}'"
      );
    }
    if envelope.nonce.len() != NONCE_LEN {
      bail!("Invalid AES-256-GCM nonce length on stored credential");
    }

    let Some(key) = self.by_id.get(&envelope.key_id) else {
      bail!(
        "Secret provider credential requires encryption key_id '{}', which is not present in the configured keyring",
        envelope.key_id
      );
    };

    let aad = build_credential_aad(
      envelope.schema_version,
      expected_provider_id,
      &expected_kind_str,
      expected_credential_revision,
    );

    let cipher = Aes256Gcm::new_from_slice(&key.bytes)
      .map_err(|_| anyhow!("Failed to initialize AES-256-GCM"))?;
    let nonce = Nonce::from_slice(&envelope.nonce);
    let plaintext = cipher
      .decrypt(
        nonce,
        Payload {
          msg: &envelope.ciphertext,
          aad: &aad,
        },
      )
      .map_err(|_| {
        anyhow!(
          "Failed to authenticate and decrypt secret provider credential (cipher/AAD verification failed)"
        )
      })?;

    serde_json::from_slice(&plaintext)
      .context("Failed to parse decrypted secret provider credential")
  }

  /// Re-encrypt an existing envelope under the active key with a fresh random nonce.
  /// Returns `Ok(None)` if the envelope is already encrypted with `self.active`.
  pub fn rotate_envelope_if_needed(
    &self,
    expected_provider_id: &str,
    expected_kind: SecretProviderKind,
    expected_credential_revision: i64,
    envelope: &StoredSecretProviderCredential,
  ) -> anyhow::Result<Option<StoredSecretProviderCredential>> {
    let decrypted = self.decrypt_credential(
      expected_provider_id,
      expected_kind,
      expected_credential_revision,
      envelope,
    )?;
    if envelope.key_id == self.active.key_id {
      return Ok(None);
    }
    let mut rotated = self.encrypt_credential(
      expected_provider_id,
      expected_kind,
      expected_credential_revision,
      &decrypted,
    )?;
    rotated.id = envelope.id.clone();
    Ok(Some(rotated))
  }

  /// Compute a keyed HMAC-SHA256 hex digest over canonical resolved secret bindings
  /// for `DeployStackIfChanged` comparison (never stores or exposes raw unkeyed hashes).
  pub fn compute_deployment_fingerprint(
    &self,
    resource_type: &str,
    resource_id: &str,
    entries: &[(String, Vec<u8>)],
  ) -> (String, String) {
    self.compute_fingerprint_with_key(
      &self.active,
      resource_type,
      resource_id,
      entries,
    )
  }

  /// Verify whether `entries` match a previously recorded `(fingerprint_hmac, fingerprint_key_id)`,
  /// supporting keys in the rotation keyring.
  pub fn matches_deployment_fingerprint(
    &self,
    resource_type: &str,
    resource_id: &str,
    entries: &[(String, Vec<u8>)],
    previous_hmac: &str,
    previous_key_id: &str,
  ) -> bool {
    let Some(key) = self.by_id.get(previous_key_id) else {
      return false;
    };
    let (computed, _) = self.compute_fingerprint_with_key(
      key,
      resource_type,
      resource_id,
      entries,
    );
    constant_time_eq(computed.as_bytes(), previous_hmac.as_bytes())
  }

  fn compute_fingerprint_with_key(
    &self,
    master_key: &MasterKey,
    resource_type: &str,
    resource_id: &str,
    entries: &[(String, Vec<u8>)],
  ) -> (String, String) {
    // Derive a domain-separated subkey from the master key
    let mut kdf = <HmacSha256 as hmac::KeyInit>::new_from_slice(
      &master_key.bytes,
    )
    .expect("valid key");
    kdf.update(b"komodo-secret-deployment-fingerprint-subkey-v1");
    let subkey = kdf.finalize().into_bytes();

    let mut sorted = entries.to_vec();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    let mut mac =
      <HmacSha256 as hmac::KeyInit>::new_from_slice(&subkey)
        .expect("valid subkey");
    mac.update(b"komodo-secret-deployment-fingerprint-v1\0");
    mac.update(resource_type.as_bytes());
    mac.update(b"\0");
    mac.update(resource_id.as_bytes());
    mac.update(&(sorted.len() as u64).to_be_bytes());
    for (target, value) in &sorted {
      mac.update(&(target.len() as u64).to_be_bytes());
      mac.update(target.as_bytes());
      mac.update(&(value.len() as u64).to_be_bytes());
      mac.update(value);
    }
    (
      hex::encode(mac.finalize().into_bytes()),
      master_key.key_id.clone(),
    )
  }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
  if a.len() != b.len() {
    return false;
  }
  let mut diff = 0u8;
  for (&x, &y) in a.iter().zip(b.iter()) {
    diff |= x ^ y;
  }
  diff == 0
}

#[cfg(test)]
mod tests {
  use super::*;

  const KEY_A_HEX: &str = "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20";
  const KEY_B_HEX: &str = "f1f2f3f4f5f6f7f8f9fafbfcfdfeff00112233445566778899aabbccddeeff01";

  #[test]
  fn rejects_missing_invalid_zero_or_jwt_reused_keys() {
    assert!(MasterKey::parse_spec("", "jwt").is_err());
    assert!(MasterKey::parse_spec("short", "jwt").is_err());
    assert!(
      MasterKey::parse_spec(
        "0000000000000000000000000000000000000000000000000000000000000000",
        "jwt"
      )
      .is_err()
    );
    // Exact string reuse of jwt_secret rejected
    assert!(MasterKey::parse_spec(KEY_A_HEX, KEY_A_HEX).is_err());
    // SHA-256(jwt_secret) reuse rejected
    let jwt = "super-secret-jwt";
    let jwt_sha_hex = hex::encode(Sha256::digest(jwt.as_bytes()));
    assert!(MasterKey::parse_spec(&jwt_sha_hex, jwt).is_err());
  }

  #[test]
  fn encrypt_decrypt_roundtrip_and_aad_tamper_detection() {
    let kr = SecretProviderKeyring::from_specs(
      &format!("primary:{KEY_A_HEX}"),
      &[],
      "distinct-jwt-secret",
    )
    .unwrap();
    let cred = SecretProviderCredentialInput::Infisical {
      client_secret: "st.universal-auth.secret.value".to_string(),
    };
    let env1 = kr
      .encrypt_credential(
        "670000000000000000000001",
        SecretProviderKind::Infisical,
        1,
        &cred,
      )
      .unwrap();
    let env2 = kr
      .encrypt_credential(
        "670000000000000000000001",
        SecretProviderKind::Infisical,
        1,
        &cred,
      )
      .unwrap();

    // Nonces and ciphertexts must differ across encryptions of identical plaintext
    assert_ne!(env1.nonce, env2.nonce);
    assert_ne!(env1.ciphertext, env2.ciphertext);
    assert_eq!(env1.key_id, "primary");

    // Debug output must redact nonce & ciphertext & client_secret
    let dbg_cred = format!("{cred:?}");
    let dbg_env = format!("{env1:?}");
    assert!(!dbg_cred.contains("st.universal-auth"));
    assert!(dbg_env.contains("[REDACTED]"));

    let out = kr
      .decrypt_credential(
        "670000000000000000000001",
        SecretProviderKind::Infisical,
        1,
        &env1,
      )
      .unwrap();
    assert_eq!(out, cred);

    // Tampering with provider_id fails AAD verification
    assert!(
      kr.decrypt_credential(
        "670000000000000000000002",
        SecretProviderKind::Infisical,
        1,
        &env1,
      )
      .is_err()
    );
    // Tampering with credential_revision fails AAD verification
    assert!(
      kr.decrypt_credential(
        "670000000000000000000001",
        SecretProviderKind::Infisical,
        2,
        &env1,
      )
      .is_err()
    );
    // Tampering with ciphertext byte fails GCM tag check
    let mut corrupted = env1.clone();
    corrupted.ciphertext[0] ^= 0xff;
    assert!(
      kr.decrypt_credential(
        "670000000000000000000001",
        SecretProviderKind::Infisical,
        1,
        &corrupted,
      )
      .is_err()
    );
  }

  #[test]
  fn keyring_rotation_and_keyed_fingerprint() {
    let old_kr = SecretProviderKeyring::from_specs(
      &format!("k-old:{KEY_A_HEX}"),
      &[],
      "jwt",
    )
    .unwrap();
    let cred = SecretProviderCredentialInput::Infisical {
      client_secret: "rotate-me-secret".to_string(),
    };
    let old_env = old_kr
      .encrypt_credential(
        "prov-1",
        SecretProviderKind::Infisical,
        3,
        &cred,
      )
      .unwrap();

    let (fp_old, fp_kid) = old_kr.compute_deployment_fingerprint(
      "Stack",
      "stack-1",
      &[(
        "env:DATABASE_URL".to_string(),
        b"postgres://u:p@db/app".to_vec(),
      )],
    );
    assert_eq!(fp_kid, "k-old");

    // New keyring with k-new active and k-old in previous_keys
    let new_kr = SecretProviderKeyring::from_specs(
      &format!("k-new:{KEY_B_HEX}"),
      &[format!("k-old:{KEY_A_HEX}")],
      "jwt",
    )
    .unwrap();

    // Fingerprint recorded under k-old still verifies cleanly before next deploy!
    assert!(new_kr.matches_deployment_fingerprint(
      "Stack",
      "stack-1",
      &[(
        "env:DATABASE_URL".to_string(),
        b"postgres://u:p@db/app".to_vec(),
      )],
      &fp_old,
      &fp_kid,
    ));
    // Changed secret value does NOT match!
    assert!(!new_kr.matches_deployment_fingerprint(
      "Stack",
      "stack-1",
      &[(
        "env:DATABASE_URL".to_string(),
        b"postgres://u:NEW@db/app".to_vec(),
      )],
      &fp_old,
      &fp_kid,
    ));

    // Rotate envelope re-encrypts to k-new
    let rotated = new_kr
      .rotate_envelope_if_needed(
        "prov-1",
        SecretProviderKind::Infisical,
        3,
        &old_env,
      )
      .unwrap()
      .expect("should rotate from k-old to k-new");
    assert_eq!(rotated.key_id, "k-new");
    assert_ne!(rotated.nonce, old_env.nonce);

    // Subsequent rotation check is a no-op
    assert!(
      new_kr
        .rotate_envelope_if_needed(
          "prov-1",
          SecretProviderKind::Infisical,
          3,
          &rotated,
        )
        .unwrap()
        .is_none()
    );

    // Keyring with ONLY k-new can decrypt rotated envelope, and rejects old_env
    let strict_new_kr = SecretProviderKeyring::from_specs(
      &format!("k-new:{KEY_B_HEX}"),
      &[],
      "jwt",
    )
    .unwrap();
    assert_eq!(
      strict_new_kr
        .decrypt_credential(
          "prov-1",
          SecretProviderKind::Infisical,
          3,
          &rotated
        )
        .unwrap(),
      cred
    );
    assert!(
      strict_new_kr
        .decrypt_credential(
          "prov-1",
          SecretProviderKind::Infisical,
          3,
          &old_env
        )
        .is_err()
    );
  }
}
