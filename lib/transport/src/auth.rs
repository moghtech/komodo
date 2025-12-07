//! Implementes both sides of Noise handshake
//! using asymmetric private-public key authentication.

use std::time::Duration;

use anyhow::Context;
use axum::http::{HeaderMap, HeaderValue};
use data_encoding::BASE64;
use noise::{NoiseHandshake, key::SpkiPublicKey};
use periphery_client::transport::LoginMessage;
use rand::RngCore;
use sha2::{Digest, Sha256};
use tracing::warn;

use crate::websocket::{
  Websocket, WebsocketExt, login::LoginWebsocketExt,
};

pub trait PublicKeyValidator {
  type ValidationResult;
  fn validate(
    &self,
    public_key: String,
  ) -> impl Future<Output = anyhow::Result<Self::ValidationResult>>;
}

pub struct LoginFlowArgs<'a, 's, V, W> {
  pub identifiers: ConnectionIdentifiers<'a>,
  pub private_key: &'a str,
  pub public_key_validator: V,
  pub socket: &'s mut W,
}

pub trait LoginFlow {
  fn login<'a, 's, V: PublicKeyValidator, W: Websocket>(
    args: LoginFlowArgs<'a, 's, V, W>,
  ) -> impl Future<Output = anyhow::Result<V::ValidationResult>>;
}

pub const AUTH_TIMEOUT: Duration = Duration::from_secs(2);

pub struct ServerLoginFlow;

impl LoginFlow for ServerLoginFlow {
  async fn login<'a, 's, V: PublicKeyValidator, W: Websocket>(
    LoginFlowArgs {
      identifiers,
      private_key,
      public_key_validator,
      socket,
    }: LoginFlowArgs<'a, 's, V, W>,
  ) -> anyhow::Result<V::ValidationResult> {
    // Server generates random nonce / uuid and sends to client
    let nonce = nonce();

    let res = async {
      socket
        .send_message(LoginMessage::Nonce(nonce))
        .await
        .context("Failed to send connection nonce")?;

      let mut handshake = NoiseHandshake::new_responder(
        private_key,
        // Builds the handshake using the connection-unique prologue hash.
        // The prologue must be the same on both sides of connection.
        &identifiers.hash(&nonce),
      )
      .context("Failed to initialize handshake")?;

      // Receive and read handshake_m1
      let handshake_m1 = socket
        .recv_login_handshake_bytes()
        .await
        .context("Failed to get handshake_m1")?;
      handshake
        .read_message(&handshake_m1)
        .context("Failed to read handshake_m1")?;

      // Send handshake_m2
      let handshake_m2 = handshake
        .next_message()
        .context("Failed to write handshake_m2")?;
      socket
        .send_message(LoginMessage::Handshake(handshake_m2))
        .await
        .context("Failed to send handshake_m2")?;

      // Receive and read handshake_m3
      let handshake_m3 = socket
        .recv_login_handshake_bytes()
        .await
        .context("Failed to get handshake_m3")?;
      handshake
        .read_message(&handshake_m3)
        .context("Failed to read handshake_m3")?;

      // Server now has client public key
      let public_key =
        SpkiPublicKey::from_raw_bytes(handshake.remote_public_key()?)
          .context("Invalid public key")?
          .into_inner();

      public_key_validator.validate(public_key).await
    }
    .await;

    match res {
      Ok(res) => {
        socket
          .send_message(LoginMessage::Success)
          .await
          .context("Failed to send login successful to client")?;
        Ok(res)
      }
      Err(e) => {
        if let Err(e) = socket
          .send_login_error(&e)
          .await
          .context("Failed to send login failed to client")
        {
          // Log additional error
          warn!("{e:#}");
        }
        // Close socket
        let _ = socket.close().await;
        // Return the original error
        Err(e)
      }
    }
  }
}

pub struct ClientLoginFlow;

impl LoginFlow for ClientLoginFlow {
  async fn login<'a, 's, V: PublicKeyValidator, W: Websocket>(
    LoginFlowArgs {
      identifiers,
      private_key,
      public_key_validator,
      socket,
    }: LoginFlowArgs<'a, 's, V, W>,
  ) -> anyhow::Result<V::ValidationResult> {
    let res = async {
      // Receive nonce and channel from server
      let nonce = socket
        .recv_login_nonce()
        .await
        .context("Failed to receive connection nonce")?;

      let mut handshake = NoiseHandshake::new_initiator(
        private_key,
        // Builds the handshake using the connection-unique prologue hash.
        // The prologue must be the same on both sides of connection.
        &identifiers.hash(&nonce),
      )
      .context("Failed to initialize handshake")?;

      // Send handshake_m1
      let handshake_m1 = handshake
        .next_message()
        .context("Failed to write handshake m1")?;
      socket
        .send_message(LoginMessage::Handshake(handshake_m1))
        .await
        .context("Failed to send handshake_m1")?;

      // Receive and read handshake_m2
      let handshake_m2 = socket
        .recv_login_handshake_bytes()
        .await
        .context("Failed to get handshake_m2")?;
      handshake
        .read_message(&handshake_m2)
        .context("Failed to read handshake_m2")?;

      // Client now has server public key.
      // Perform validation before proceeding.
      let public_key =
        SpkiPublicKey::from_raw_bytes(handshake.remote_public_key()?)
          .context("Invalid public key")?
          .into_inner();
      let validation_result =
        public_key_validator.validate(public_key).await?;

      // Send handshake_m3
      let handshake_m3 = handshake
        .next_message()
        .context("Failed to write handshake_m3")?;
      socket
        .send_message(LoginMessage::Handshake(handshake_m3))
        .await
        .context("Failed to send handshake_m3")?;

      // Receive login sucessful
      socket
        .recv_login_success()
        .await
        .context("Failed to receive Login Success message")?;

      anyhow::Ok(validation_result)
    }
    .await;

    match res {
      Ok(res) => Ok(res),
      Err(e) => {
        if let Err(e) = socket
          .send_login_error(&e)
          .await
          .context("Failed to send login failed to client")
        {
          // Log additional error
          warn!("{e:#}");
        }
        // Close socket
        let _ = socket.close().await;
        // Return the original error
        Err(e)
      }
    }
  }
}

fn nonce() -> [u8; 32] {
  let mut out = [0u8; 32];
  rand::rng().fill_bytes(&mut out);
  out
}

#[derive(Clone, Copy)]
pub struct ConnectionIdentifiers<'a> {
  /// Server hostname
  pub host: &'a [u8],
  /// Query: 'server=<SERVER>'
  pub query: &'a [u8],
  /// Sec-Websocket-Accept, unique for each connection
  pub accept: &'a [u8],
}

impl ConnectionIdentifiers<'_> {
  /// nonce: Server computed random connection nonce, sent to client before auth handshake
  pub fn hash(&self, nonce: &[u8]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"noise-wss-v1|");
    hash.update(self.host);
    hash.update(b"|");
    hash.update(self.query);
    hash.update(b"|");
    hash.update(self.accept);
    hash.update(b"|");
    hash.update(nonce);
    hash.finalize().into()
  }
}

pub struct AddressConnectionIdentifiers {
  host: String,
}

impl AddressConnectionIdentifiers {
  pub fn extract(address: &str) -> anyhow::Result<Self> {
    let url = ::url::Url::parse(address)
      .context("Failed to parse server address")?;
    let mut host = url.host().context("url has no host")?.to_string();
    if let Some(port) = url.port() {
      host.push(':');
      host.push_str(&port.to_string());
    };
    Ok(Self { host })
  }

  pub fn host(&self) -> &String {
    &self.host
  }

  pub fn build<'a>(
    &'a self,
    accept: &'a [u8],
    query: &'a [u8],
  ) -> ConnectionIdentifiers<'a> {
    ConnectionIdentifiers {
      host: self.host.as_bytes(),
      query,
      accept,
    }
  }
}

/// Used to extract owned connection identifier
/// in server side connection handler.
pub struct HeaderConnectionIdentifiers {
  host: HeaderValue,
  accept: String,
}

impl HeaderConnectionIdentifiers {
  pub fn extract(
    headers: &mut HeaderMap,
  ) -> anyhow::Result<HeaderConnectionIdentifiers> {
    let host = headers
      .remove("x-forwarded-host")
      .or(headers.remove("host"))
      .context("Failed to get connection host")?;
    let key = headers
      .remove("sec-websocket-key")
      .context("Headers do not contain Sec-Websocket-Key")?;
    let accept = compute_accept(key.as_bytes());
    Ok(HeaderConnectionIdentifiers { host, accept })
  }

  pub fn host(&self) -> anyhow::Result<String> {
    self
      .host
      .to_str()
      .map(str::to_string)
      .context("Failed to parse header host to string")
  }

  pub fn build<'a>(
    &'a self,
    query: &'a [u8],
  ) -> ConnectionIdentifiers<'a> {
    ConnectionIdentifiers {
      host: self.host.as_bytes(),
      accept: self.accept.as_bytes(),
      query,
    }
  }
}

pub fn compute_accept(sec_websocket_key: &[u8]) -> String {
  // This is standard GUID to compute Sec-Websocket-Accept
  const GUID: &[u8] = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
  let mut sha1 = sha1::Sha1::new();
  sha1.update(sec_websocket_key);
  sha1.update(GUID);
  let digest = sha1.finalize();
  BASE64.encode(&digest)
}
