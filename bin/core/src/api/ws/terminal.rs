use anyhow::anyhow;
use axum::{
  extract::{FromRequestParts, WebSocketUpgrade, ws},
  http::request,
  response::IntoResponse,
};
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt as _};
use komodo_client::{
  api::terminal::ConnectTerminalQuery, entities::user::User,
};
use mogh_auth_server::request_ip::RequestIp;
use periphery_client::api::terminal::DisconnectTerminal;
use serde::de::DeserializeOwned;
use tokio_util::sync::CancellationToken;

use crate::{
  helpers::terminal::setup_target_for_user,
  periphery::{PeripheryClient, terminal::ConnectTerminalResponse},
  state::periphery_connections,
};

#[instrument("ConnectTerminal", skip(ws))]
pub async fn handler(
  RequestIp(ip): RequestIp,
  Qs(query): Qs<ConnectTerminalQuery>,
  ws: WebSocketUpgrade,
) -> impl IntoResponse {
  ws.on_upgrade(move |socket| async move {
    let Some((mut client_socket, user)) =
      super::user_ws_login(socket, ip).await
    else {
      return;
    };

    let (periphery, response) =
      match setup_forwarding(query, &user).await {
        Ok(response) => response,
        Err(e) => {
          let _ = client_socket
            .send(ws::Message::text(format!("ERROR: {e:#}")))
            .await;
          let _ = client_socket.close().await;
          return;
        }
      };

    forward_ws_channel(periphery, client_socket, response).await
  })
}

async fn setup_forwarding(
  ConnectTerminalQuery {
    target,
    terminal,
    init,
  }: ConnectTerminalQuery,
  user: &User,
) -> anyhow::Result<(PeripheryClient, ConnectTerminalResponse)> {
  let (target, terminal, periphery) =
    setup_target_for_user(target, terminal, init, user).await?;
  let response = periphery.connect_terminal(terminal, target).await?;
  Ok((periphery, response))
}

async fn forward_ws_channel(
  periphery: PeripheryClient,
  client_socket: axum::extract::ws::WebSocket,
  ConnectTerminalResponse {
    channel,
    sender: periphery_sender,
    receiver: mut periphery_receiver,
  }: ConnectTerminalResponse,
) {
  let (mut client_send, mut client_receive) = client_socket.split();
  let cancel = CancellationToken::new();

  periphery_receiver.set_cancel(cancel.clone());

  trace!("starting ws exchange");

  let core_to_periphery = async {
    loop {
      let client_recv_res = tokio::select! {
        res = client_receive.next() => res,
        _ = cancel.cancelled() => break,
      };
      let bytes = match client_recv_res {
        Some(Ok(ws::Message::Binary(bytes))) => bytes.into(),
        Some(Ok(ws::Message::Text(text))) => {
          let bytes: Bytes = text.into();
          bytes.into()
        }
        Some(Ok(ws::Message::Close(_frame))) => {
          break;
        }
        Some(Err(_e)) => {
          break;
        }
        None => {
          break;
        }
        // Ignore
        Some(Ok(_)) => continue,
      };
      if let Err(_e) =
        periphery_sender.send_terminal(channel, Ok(bytes)).await
      {
        break;
      };
    }
    cancel.cancel();
    let _ = periphery_sender
      .send_terminal(channel, Err(anyhow!("Client disconnected")))
      .await;
  };

  let periphery_to_core = async {
    loop {
      // Already adheres to cancellation token
      match periphery_receiver.recv().await {
        Ok(Ok(bytes)) => {
          if let Err(e) =
            client_send.send(ws::Message::Binary(bytes.into())).await
          {
            debug!("{e:?}");
            break;
          };
        }
        Ok(Err(e)) => {
          let _ = client_send
            .send(ws::Message::text(format!("{e:#}")))
            .await;
          break;
        }
        Err(_) => {
          let _ =
            client_send.send(ws::Message::text("STREAM EOF")).await;
          break;
        }
      }
    }
    let _ = client_send.close().await;
    cancel.cancel();
  };

  tokio::join!(core_to_periphery, periphery_to_core);

  // Cleanup
  if let Err(e) =
    periphery.request(DisconnectTerminal { channel }).await
  {
    warn!(
      "Failed to disconnect Periphery terminal forwarding | {e:#}",
    )
  }
  if let Some(connection) =
    periphery_connections().get(&periphery.id).await
  {
    connection.terminals.remove(&channel).await;
  }
}

pub struct Qs<T>(pub T);

impl<S, T> FromRequestParts<S> for Qs<T>
where
  S: Send + Sync,
  T: DeserializeOwned,
{
  type Rejection = axum::response::Response;

  async fn from_request_parts(
    parts: &mut request::Parts,
    _state: &S,
  ) -> Result<Self, Self::Rejection> {
    let raw = parts.uri.query().unwrap_or_default();
    serde_qs::from_str::<T>(raw).map(Qs).map_err(|e| {
      axum::response::IntoResponse::into_response((
        axum::http::StatusCode::BAD_REQUEST,
        format!("Failed to parse request query: {e}"),
      ))
    })
  }
}
