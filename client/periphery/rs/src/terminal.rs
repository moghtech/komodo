use std::sync::Arc;

use anyhow::Context;
use komodo_client::terminal::TerminalStreamResponse;
use rustls::{ClientConfig, client::danger::ServerCertVerifier};
use tokio::net::TcpStream;
use tokio_tungstenite::{Connector, MaybeTlsStream, WebSocketStream};

use crate::{
  PeripheryClient,
  api::terminal::{
    ConnectTerminalQuery, CreateTerminalAuthToken,
    ExecuteTerminalBody,
  },
};

impl PeripheryClient {
  /// Handles ws connect and login.
  /// Does not handle reconnect.
  pub async fn connect_terminal(
    &self,
    terminal: String,
    init: Option<String>,
  ) -> anyhow::Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    tracing::trace!(
      "request | type: ConnectTerminal | terminal name: {terminal} | init command: {init:?}",
    );

    let token = self
      .request(CreateTerminalAuthToken {})
      .await
      .context("Failed to create terminal auth token")?;

    let query_str = serde_qs::to_string(&ConnectTerminalQuery {
      token: token.token,
      terminal,
      init,
    })
    .context("Failed to serialize query string")?;

    let url = format!(
      "{}/terminal?{query_str}",
      self.address.replacen("http", "ws", 1)
    );

    let (stream, _) = if url.starts_with("wss") {
      tokio_tungstenite::connect_async_tls_with_config(
        url,
        None,
        false,
        Some(Connector::Rustls(Arc::new(
          ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(
              InsecureVerifier,
            ))
            .with_no_client_auth(),
        ))),
      )
      .await
      .context("failed to connect to websocket")?
    } else {
      tokio_tungstenite::connect_async(url)
        .await
        .context("failed to connect to websocket")?
    };

    Ok(stream)
  }

  /// Executes command on specified terminal,
  /// and streams the response ending in [KOMODO_EXIT_CODE][komodo_client::entities::KOMODO_EXIT_CODE]
  /// sentinal value as the expected final line of the stream.
  ///
  /// Example final line:
  /// ```
  /// __KOMODO_EXIT_CODE:0
  /// ```
  ///
  /// This means the command exited with code 0 (success).
  ///
  /// If this value is NOT the final item before stream closes, it means
  /// the terminal exited mid command, before giving status. Example: running `exit`.
  #[tracing::instrument(level = "debug", skip(self))]
  pub async fn execute_terminal(
    &self,
    terminal: String,
    command: String,
  ) -> anyhow::Result<TerminalStreamResponse> {
    tracing::trace!(
      "sending request | type: ExecuteTerminal | terminal name: {terminal} | command: {command}",
    );
    let req = crate::periphery_http_client()
      .post(format!("{}/terminal/execute", self.address))
      .json(&ExecuteTerminalBody { terminal, command })
      .header("authorization", &self.passkey);
    let res =
      req.send().await.context("Failed at request to periphery")?;
    let status = res.status();
    tracing::debug!(
      "got response | type: ExecuteTerminal | {status} | response: {res:?}",
    );
    if status.is_success() {
      Ok(TerminalStreamResponse(res))
    } else {
      tracing::debug!("response is non-200");

      let text = res
        .text()
        .await
        .context("Failed to convert response to text")?;

      tracing::debug!("got response text, deserializing error");

      let error = serror::deserialize_error(text).context(status);

      Err(error)
    }
  }
}

#[derive(Debug)]
struct InsecureVerifier;

impl ServerCertVerifier for InsecureVerifier {
  fn verify_server_cert(
    &self,
    _end_entity: &rustls::pki_types::CertificateDer<'_>,
    _intermediates: &[rustls::pki_types::CertificateDer<'_>],
    _server_name: &rustls::pki_types::ServerName<'_>,
    _ocsp_response: &[u8],
    _now: rustls::pki_types::UnixTime,
  ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error>
  {
    Ok(rustls::client::danger::ServerCertVerified::assertion())
  }

  fn verify_tls12_signature(
    &self,
    _message: &[u8],
    _cert: &rustls::pki_types::CertificateDer<'_>,
    _dss: &rustls::DigitallySignedStruct,
  ) -> Result<
    rustls::client::danger::HandshakeSignatureValid,
    rustls::Error,
  > {
    Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
  }

  fn verify_tls13_signature(
    &self,
    _message: &[u8],
    _cert: &rustls::pki_types::CertificateDer<'_>,
    _dss: &rustls::DigitallySignedStruct,
  ) -> Result<
    rustls::client::danger::HandshakeSignatureValid,
    rustls::Error,
  > {
    Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
  }

  fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
    vec![
      rustls::SignatureScheme::RSA_PKCS1_SHA1,
      rustls::SignatureScheme::ECDSA_SHA1_Legacy,
      rustls::SignatureScheme::RSA_PKCS1_SHA256,
      rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
      rustls::SignatureScheme::RSA_PKCS1_SHA384,
      rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
      rustls::SignatureScheme::RSA_PKCS1_SHA512,
      rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
      rustls::SignatureScheme::RSA_PSS_SHA256,
      rustls::SignatureScheme::RSA_PSS_SHA384,
      rustls::SignatureScheme::RSA_PSS_SHA512,
      rustls::SignatureScheme::ED25519,
      rustls::SignatureScheme::ED448,
    ]
  }
}
