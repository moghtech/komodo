use axum::{
  extract::{Query, WebSocketUpgrade, ws::Message},
  response::IntoResponse,
};
use futures::SinkExt;
use komodo_client::{
  api::terminal::ConnectContainerExecQuery,
  entities::{permission::PermissionLevel, server::Server},
};

use crate::permission::get_check_permissions;

#[instrument(name = "ConnectContainerExec", skip(ws))]
pub async fn terminal(
  Query(ConnectContainerExecQuery {
    server,
    container,
    shell,
  }): Query<ConnectContainerExecQuery>,
  ws: WebSocketUpgrade,
) -> impl IntoResponse {
  ws.on_upgrade(|socket| async move {
    let Some((mut client_socket, user)) =
      super::ws_login(socket).await
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
        let _ = client_socket
          .send(Message::text(format!("ERROR: {e:#}")))
          .await;
        let _ = client_socket.close().await;
        return;
      }
    };

    super::handle_container_terminal(
      client_socket,
      &server,
      container,
      shell,
    )
    .await
  })
}
