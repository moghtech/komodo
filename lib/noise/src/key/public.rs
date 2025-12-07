use std::path::Path;

use anyhow::{Context, anyhow};
use der::{Decode as _, Encode as _, asn1::BitStringRef};
use data_encoding::BASE64;
use spki::SubjectPublicKeyInfoRef;

#[derive(PartialEq, Clone)]
pub struct SpkiPublicKey(String);

impl From<String> for SpkiPublicKey {
  fn from(value: String) -> Self {
    Self(value)
  }
}

impl std::fmt::Display for SpkiPublicKey {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0)
  }
}

impl SpkiPublicKey {
  pub fn as_str(&self) -> &str {
    &self.0
  }

  pub fn as_bytes(&self) -> &[u8] {
    self.0.as_bytes()
  }

  pub fn into_inner(self) -> String {
    self.0
  }

  pub fn as_pem(&self) -> String {
    let public_key = &self.0;
    format!(
      r#"-----BEGIN PUBLIC KEY-----
{public_key}
-----END PUBLIC KEY-----
"#
    )
  }

  pub fn write_pem_sync(
    &self,
    path: impl AsRef<Path>,
  ) -> anyhow::Result<()> {
    let path = path.as_ref();
    tracing::info!("Writing public key to {path:?}");
    secret_file::write_sync(path, self.as_pem()).with_context(|| {
      format!("Failed to write public key pem to {path:?}")
    })
  }

  pub async fn write_pem_async(
    &self,
    path: impl AsRef<Path>,
  ) -> anyhow::Result<()> {
    let path = path.as_ref();
    tracing::info!("Writing public key to {path:?}");
    secret_file::write_async(path, self.as_pem())
      .await
      .with_context(|| {
        format!("Failed to write public key pem to {path:?}")
      })
  }

  pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
    let path = path.as_ref();
    let contents =
      std::fs::read_to_string(path).with_context(|| {
        format!("Failed to read public key at {path:?}")
      })?;
    Self::from_maybe_pem(&contents)
  }

  /// Accepts pem rfc7468 (openssl) or base64 der (second line of rfc7468 pem).
  pub fn from_maybe_pem(
    public_key_maybe_pem: &str,
  ) -> anyhow::Result<Self> {
    // check pem rfc7468 (openssl)
    let public_key_der =
      if public_key_maybe_pem.starts_with("-----BEGIN") {
        let (_label, public_key_der) =
          pem_rfc7468::decode_vec(public_key_maybe_pem.as_bytes())
            .map_err(anyhow::Error::msg)
            .context("Failed to get der from pem")
            .unwrap();
        public_key_der
      } else {
        BASE64
          .decode(public_key_maybe_pem.as_bytes())
          .context("Public key is not base64")?
      };
    Self::from_der(&public_key_der)
  }

  /// Accepts der (not base64)
  pub fn from_der(public_key_der: &[u8]) -> anyhow::Result<Self> {
    let spki = SubjectPublicKeyInfoRef::from_der(public_key_der)
      .map_err(anyhow::Error::msg)
      .context("Invalid public key der")?;
    if spki.algorithm.oid != super::OID_X25519 {
      return Err(anyhow!("Public key is not X25519"));
    }
    Self::from_raw_bytes(spki.subject_public_key.raw_bytes())
  }

  pub fn from_raw_bytes(public_key: &[u8]) -> anyhow::Result<Self> {
    let bs = BitStringRef::new(0, public_key)
      .map_err(anyhow::Error::msg)
      .context("Failed to parse public key bytes into bit string")?;

    let spki = spki::SubjectPublicKeyInfo {
      algorithm: super::algorithm(),
      subject_public_key: bs,
    };

    let mut buf = [0u8; 128];
    let public_key = spki
      .encode_to_slice(&mut buf)
      .map_err(anyhow::Error::msg)
      .context("Failed to write subject public key info into der")?;

    Ok(Self(BASE64.encode(public_key)))
  }

  pub fn from_private_key(
    maybe_pkcs8_private_key: &str,
  ) -> anyhow::Result<Self> {
    // Create mock client handshake. The private key doesn't matter.
    // Trying to get the "server" public key.
    let mut client_handshake =
      crate::NoiseHandshake::new_initiator("0000", &[])
        .context("Failed to create client handshake")?;
    // Create mock server handshake.
    // Use the target private key with server handshake,
    // since its public key is the first available in the flow.
    let mut server_handshake = crate::NoiseHandshake::new_responder(
      maybe_pkcs8_private_key,
      &[],
    )
    .context("Failed to create server handshake")?;
    // write message 1
    let message_1 = client_handshake
      .next_message()
      .context("CLIENT: failed to write message 1")?;
    // read message 1
    server_handshake
      .read_message(&message_1)
      .context("SERVER: failed to read message 1")?;
    // write message 2
    let message_2 = server_handshake
      .next_message()
      .context("SERVER: failed to write message 2")?;
    // read message 2
    client_handshake
      .read_message(&message_2)
      .context("CLIENT: failed to read message 2")?;
    // client now has server public key
    let raw_public_key = client_handshake
      .remote_public_key()
      .map(Vec::from)
      .context("Failed to get public key")?;
    Self::from_raw_bytes(&raw_public_key)
  }
}
