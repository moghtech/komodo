use anyhow::anyhow;
use axum::{
  extract::{WebSocketUpgrade, ws::Message},
  response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use komodo_client::entities::{
  ResourceTarget, permission::PermissionLevel, user::User,
};
use serde_json::json;
use serror::serialize_error;
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::helpers::{
  channel::update_channel, query::get_user_permission_on_target,
};

#[instrument(level = "debug")]
pub async fn handler(ws: WebSocketUpgrade) -> impl IntoResponse {
  // get a reveiver for internal update messages.
  let mut receiver = update_channel().receiver.resubscribe();

  // handle http -> ws updgrade
  ws.on_upgrade(|socket| async move {
    let Some((socket, user)) = super::ws_login(socket).await else {
      return
    };

    let (mut ws_sender, mut ws_reciever) = socket.split();

    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();

    tokio::spawn(async move {
      loop {
        // poll for updates off the receiver / await cancel.
        let update = select! {
          _ = cancel_clone.cancelled() => break,
          update = receiver.recv() => {update.expect("failed to recv update msg")}
        };

        // before sending every update, verify user is still valid.
        // kill the connection is user if found to be invalid.
        let user = super::check_user_valid(&user.id).await;
        let user = match user {
          Err(e) => {
            let _ = ws_sender
              .send(Message::text(json!({ "type": "INVALID_USER", "msg": serialize_error(&e) }).to_string()))
              .await;
            let _ = ws_sender.close().await;
            return;
          },
          Ok(user) => user,
        };

        // Only send if user has permission on the target resource.
        if user_can_see_update(&user, &update.target).await.is_ok() {
          let _ = ws_sender
            .send(Message::text(serde_json::to_string(&update).unwrap()))
            .await;
        }
      }
    });

    // Handle messages from the client.
    // After login, only handles close message.
    while let Some(msg) = ws_reciever.next().await {
      match msg {
        Ok(msg) => {
          if let Message::Close(_) = msg {
            cancel.cancel();
            return;
          }
        }
        Err(_) => {
          cancel.cancel();
          return;
        }
      }
    }
    })
}

#[instrument(level = "debug")]
async fn user_can_see_update(
  user: &User,
  update_target: &ResourceTarget,
) -> anyhow::Result<()> {
  if user.admin {
    return Ok(());
  }
  let permissions =
    get_user_permission_on_target(user, update_target).await?;
  if permissions > PermissionLevel::None {
    Ok(())
  } else {
    Err(anyhow!(
      "user does not have permissions on {update_target:?}"
    ))
  }
}
