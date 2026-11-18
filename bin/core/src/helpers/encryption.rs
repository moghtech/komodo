//! Optional encryption of secret Variable values in the database.

use std::sync::LazyLock;

use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
use anyhow::{Context, anyhow};
use data_encoding::BASE64;
use komodo_client::entities::variable::Variable;
use sha2::{Digest, Sha256};

use crate::config::core_config;

/// Stored as `komodo-enc:v1:<kid>:<base64(nonce || ciphertext)>`
const PREFIX: &str = "komodo-enc:";
const NONCE_LEN: usize = 12;

pub struct EncryptionKey {
  cipher: Aes256Gcm,
  kid: String,
}

impl EncryptionKey {
  fn parse(key: &str) -> anyhow::Result<EncryptionKey> {
    let key = BASE64
      .decode(key.trim().as_bytes())
      .context("Must be base64 encoded")?;
    anyhow::ensure!(
      key.len() == 32,
      "Must decode to 32 bytes, got {}",
      key.len()
    );
    Ok(EncryptionKey {
      cipher: Aes256Gcm::new_from_slice(&key)?,
      kid: hex::encode(&Sha256::digest(&key)[..8]),
    })
  }
}

pub static ENCRYPTION_KEY: LazyLock<Option<EncryptionKey>> =
  LazyLock::new(|| {
    let key = &core_config().variable_encryption_key;
    if key.is_empty() {
      return None;
    }
    let key = EncryptionKey::parse(key).unwrap_or_else(|e| {
      panic!("Invalid 'variable_encryption_key' | {e:#}")
    });
    info!("Secret Variable encryption enabled | key id: {}", key.kid);
    Some(key)
  });

fn seal(key: &EncryptionKey, plain: &str) -> anyhow::Result<String> {
  let nonce = rand::random::<[u8; NONCE_LEN]>();
  let ct = key
    .cipher
    .encrypt(Nonce::from_slice(&nonce), plain.as_bytes())
    .map_err(|_| anyhow!("Encryption failed"))?;
  Ok(format!(
    "{PREFIX}v1:{}:{}",
    key.kid,
    BASE64.encode(&[&nonce[..], &ct].concat())
  ))
}

fn open(
  key: Option<&EncryptionKey>,
  value: &str,
) -> anyhow::Result<String> {
  let Some(value) = value.strip_prefix(PREFIX) else {
    return Ok(value.to_string());
  };
  let (kid, data) = value
    .strip_prefix("v1:")
    .and_then(|value| value.split_once(':'))
    .context("Invalid encrypted value")?;
  let key = key.with_context(|| {
    format!(
      "Value is encrypted with key id {kid}, but no 'variable_encryption_key' is configured"
    )
  })?;
  anyhow::ensure!(
    kid == key.kid,
    "Value is encrypted with key id {kid}, but the configured 'variable_encryption_key' has key id {}",
    key.kid
  );
  let data = BASE64
    .decode(data.as_bytes())
    .context("Invalid encrypted value encoding")?;
  anyhow::ensure!(data.len() > NONCE_LEN, "Invalid encrypted value");
  let (nonce, ct) = data.split_at(NONCE_LEN);
  let plain = key
    .cipher
    .decrypt(Nonce::from_slice(nonce), ct)
    .map_err(|_| anyhow!("Decryption failed, value is corrupted"))?;
  String::from_utf8(plain).context("Decrypted value is not UTF-8")
}

pub fn is_encrypted(value: &str) -> bool {
  value.starts_with(PREFIX)
}

/// Values which are already encrypted are stored as is.
pub fn encrypt_value(
  is_secret: bool,
  value: String,
) -> anyhow::Result<String> {
  match ENCRYPTION_KEY.as_ref() {
    Some(key) if is_secret && !is_encrypted(&value) => {
      seal(key, &value)
    }
    _ => Ok(value),
  }
}

pub fn decrypt_variable(
  mut variable: Variable,
) -> anyhow::Result<Variable> {
  if variable.is_secret {
    variable.value = open(ENCRYPTION_KEY.as_ref(), &variable.value)
      .with_context(|| {
      format!("Failed to decrypt secret Variable '{}'", variable.name)
    })?;
  }
  Ok(variable)
}

/// Like [decrypt_variable], but keeps the stored value on failure.
pub fn decrypt_variable_or_stored(variable: Variable) -> Variable {
  decrypt_variable(variable.clone()).unwrap_or_else(|e| {
    debug!("{e:#} | Using the stored value");
    variable
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn round_trip() {
    let key = || {
      EncryptionKey::parse(
        &BASE64.encode(&rand::random::<[u8; 32]>()),
      )
      .unwrap()
    };
    let sealed_with = key();
    let sealed = seal(&sealed_with, "secret").unwrap();
    assert!(sealed.starts_with(PREFIX));
    assert_eq!(open(Some(&sealed_with), &sealed).unwrap(), "secret");
    assert!(open(Some(&key()), &sealed).is_err());
    assert!(open(None, &sealed).is_err());
    assert_eq!(open(None, "plain").unwrap(), "plain");
  }
}
