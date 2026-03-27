use std::sync::Arc;

use anyhow::{Context, anyhow};
use axum::http::HeaderValue;
use bytes::Bytes;
use encoding::CastBytes as _;
use futures_util::{
  SinkExt, Stream, StreamExt, TryStreamExt,
  stream::{SplitSink, SplitStream},
};
use mogh_error::AddStatusCodeError;
use periphery_client::transport::EncodedTransportMessage;
use rustls::{ClientConfig, client::danger::ServerCertVerifier};
use tokio::net::TcpStream;
use tokio_tungstenite::{
  Connector, MaybeTlsStream, WebSocketStream,
  tungstenite::{
    self, handshake::client::Response, protocol::CloseFrame,
  },
};
use tokio_util::sync::CancellationToken;

use crate::timeout::MaybeWithTimeout;

use super::{
  Websocket, WebsocketMessage, WebsocketReceiver, WebsocketSender,
};

pub type InnerWebsocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct TungsteniteWebsocket(pub InnerWebsocket);

impl Websocket for TungsteniteWebsocket {
  fn split(self) -> (impl WebsocketSender, impl WebsocketReceiver) {
    let (tx, rx) = self.0.split();
    (
      TungsteniteWebsocketSender(tx),
      TungsteniteWebsocketReceiver::new(rx),
    )
  }

  fn recv_inner(
    &mut self,
  ) -> MaybeWithTimeout<
    impl Future<Output = anyhow::Result<WebsocketMessage>>,
  > {
    MaybeWithTimeout::new(try_next(&mut self.0))
  }

  async fn send(&mut self, bytes: Bytes) -> anyhow::Result<()> {
    self
      .0
      .send(tungstenite::Message::Binary(bytes))
      .await
      .context("Failed to send message over websocket")
  }

  async fn close(&mut self) -> anyhow::Result<()> {
    self
      .0
      .close(None)
      .await
      .context("Failed to send websocket close frame")
  }
}

pub type InnerWebsocketSender = SplitSink<
  WebSocketStream<MaybeTlsStream<TcpStream>>,
  tungstenite::Message,
>;

pub struct TungsteniteWebsocketSender(pub InnerWebsocketSender);

impl WebsocketSender for TungsteniteWebsocketSender {
  async fn ping(&mut self) -> anyhow::Result<()> {
    self
      .0
      .send(tungstenite::Message::Ping(Bytes::new()))
      .await
      .context("Failed to send ping over websocket")
  }

  async fn send(&mut self, bytes: Bytes) -> anyhow::Result<()> {
    self
      .0
      .send(tungstenite::Message::Binary(bytes))
      .await
      .context("Failed to send message over websocket")
  }

  async fn close(&mut self) -> anyhow::Result<()> {
    self
      .0
      .send(tungstenite::Message::Close(None))
      .await
      .context("Failed to send websocket close frame")
  }
}

async fn try_next<S>(
  stream: &mut S,
) -> anyhow::Result<WebsocketMessage>
where
  S: Stream<Item = Result<tungstenite::Message, tungstenite::Error>>
    + Unpin,
{
  loop {
    match stream.try_next().await? {
      Some(tungstenite::Message::Binary(bytes)) => {
        return Ok(WebsocketMessage::Message(
          EncodedTransportMessage::from_vec(bytes.into()),
        ));
      }
      Some(tungstenite::Message::Text(text)) => {
        let bytes: Bytes = text.into();
        return Ok(WebsocketMessage::Message(
          EncodedTransportMessage::from_vec(bytes.into()),
        ));
      }
      Some(tungstenite::Message::Ping(_)) => {
        return Ok(WebsocketMessage::Ping);
      }
      Some(tungstenite::Message::Close(_)) => {
        return Ok(WebsocketMessage::Close);
      }
      None => return Ok(WebsocketMessage::Closed),
      // Ignored messages
      Some(tungstenite::Message::Pong(_))
      | Some(tungstenite::Message::Frame(_)) => continue,
    }
  }
}

pub type InnerWebsocketReceiver =
  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

pub struct TungsteniteWebsocketReceiver {
  receiver: InnerWebsocketReceiver,
  cancel: Option<CancellationToken>,
}

impl TungsteniteWebsocketReceiver {
  pub fn new(receiver: InnerWebsocketReceiver) -> Self {
    Self {
      receiver,
      cancel: None,
    }
  }
}

impl WebsocketReceiver for TungsteniteWebsocketReceiver {
  type CloseFrame = CloseFrame;

  fn set_cancel(&mut self, cancel: CancellationToken) {
    self.cancel = Some(cancel);
  }

  async fn recv(&mut self) -> anyhow::Result<WebsocketMessage> {
    let fut = try_next(&mut self.receiver);
    if let Some(cancel) = &self.cancel {
      tokio::select! {
        res = fut => res,
        _ = cancel.cancelled() => Err(anyhow!("Cancelled before receive"))
      }
    } else {
      fut.await
    }
  }
}

impl TungsteniteWebsocket {
  pub async fn connect_maybe_tls_insecure(
    url: &str,
    insecure: bool,
  ) -> mogh_error::Result<(Self, HeaderValue)> {
    if insecure {
      Self::connect_tls_insecure(url).await
    } else {
      Self::connect(url).await
    }
  }

  pub async fn connect(
    url: &str,
  ) -> mogh_error::Result<(Self, HeaderValue)> {
    let res = tokio_tungstenite::connect_async(url).await;
    Self::handle_connection_result(url, res)
  }

  pub async fn connect_tls_insecure(
    url: &str,
  ) -> mogh_error::Result<(Self, HeaderValue)> {
    let res = tokio_tungstenite::connect_async_tls_with_config(
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
    .await;
    Self::handle_connection_result(url, res)
  }

  fn handle_connection_result(
    url: &str,
    res: Result<
      (WebSocketStream<MaybeTlsStream<TcpStream>>, Response),
      tungstenite::Error,
    >,
  ) -> mogh_error::Result<(Self, HeaderValue)> {
    let (ws, mut response) = res
      .map_err(|e| {
        let status = if let tungstenite::Error::Http(response) = &e {
          response.status()
        } else {
          return anyhow::Error::from(e).into();
        };
        e.status_code(status)
      })
      .map_err(|mut e| {
        e.error = e.error.context({
          format!("Failed to connect to websocket | url: {url}")
        });
        e
      })?;

    let accept = response
      .headers_mut()
      .remove("sec-websocket-accept")
      .context("Headers do not contain Sec-Websocket-Accept")?;

    Ok((Self(ws), accept))
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
