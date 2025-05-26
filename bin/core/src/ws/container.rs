use axum::{
  extract::{Query, WebSocketUpgrade, ws::Message},
  response::IntoResponse,
};
use futures::SinkExt;
use komodo_client::{
  api::terminal::ConnectContainerExecQuery,
  entities::{permission::PermissionLevel, server::Server},
};

use crate::{
  helpers::periphery_client, permission::get_check_permissions,
  ws::core_periphery_forward_ws,
};

#[instrument(name = "ConnectContainerExec", skip(ws))]
pub async fn handler(
  Query(ConnectContainerExecQuery {
    server,
    container,
    shell,
  }): Query<ConnectContainerExecQuery>,
  ws: WebSocketUpgrade,
) -> impl IntoResponse {
  ws.on_upgrade(|socket| async move {
    let Some((mut client_socket, user)) = super::ws_login(socket).await
    else {
      return;
    };

    let server = match get_check_permissions::<Server>(
      &server,
      &user,
      PermissionLevel::Read.terminal(),
    )
    .await
    {
      Ok(server) => server,
      Err(e) => {
        debug!("could not get server | {e:#}");
        let _ =
          client_socket.send(Message::text(format!("ERROR: {e:#}"))).await;
        let _ = client_socket.close().await;
        return;
      }
    };

    let periphery = match periphery_client(&server) {
      Ok(periphery) => periphery,
      Err(e) => {
        debug!("couldn't get periphery | {e:#}");
        let _ =
          client_socket.send(Message::text(format!("ERROR: {e:#}"))).await;
        let _ = client_socket.close().await;
        return;
      }
    };

    trace!("connecting to periphery container exec websocket");

    let periphery_socket = match periphery
      .connect_container_exec(
        container,
        shell
      )
      .await
    {
      Ok(ws) => ws,
      Err(e) => {
        debug!("Failed connect to periphery container exec websocket | {e:#}");
        let _ =
          client_socket.send(Message::text(format!("ERROR: {e:#}"))).await;
        let _ = client_socket.close().await;
        return;
      }
    };

    trace!("connected to periphery container exec websocket");

    core_periphery_forward_ws(client_socket, periphery_socket).await
  })
}
