use axum::{
  extract::{Query, WebSocketUpgrade, ws::Message},
  response::IntoResponse,
};
use futures::SinkExt;
use komodo_client::{
  api::terminal::ConnectStackExecQuery,
  entities::{
    permission::PermissionLevel, server::Server, stack::Stack,
  },
};

use crate::{
  permission::get_check_permissions, resource::get,
  state::stack_status_cache,
};

#[instrument(name = "ConnectStackExec", skip(ws))]
pub async fn terminal(
  Query(ConnectStackExecQuery {
    stack,
    service,
    shell,
  }): Query<ConnectStackExecQuery>,
  ws: WebSocketUpgrade,
) -> impl IntoResponse {
  ws.on_upgrade(|socket| async move {
    let Some((mut client_socket, user)) =
      super::ws_login(socket).await
    else {
      return;
    };

    let stack = match get_check_permissions::<Stack>(
      &stack,
      &user,
      PermissionLevel::Read.terminal(),
    )
    .await
    {
      Ok(stack) => stack,
      Err(e) => {
        debug!("could not get stack | {e:#}");
        let _ = client_socket
          .send(Message::text(format!("ERROR: {e:#}")))
          .await;
        let _ = client_socket.close().await;
        return;
      }
    };

    let server = match get::<Server>(&stack.config.server_id).await {
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

    let Some(status) = stack_status_cache().get(&stack.id).await
    else {
      debug!("could not get stack status");
      let _ = client_socket
        .send(Message::text(String::from(
          "ERROR: could not get stack status",
        )))
        .await;
      let _ = client_socket.close().await;
      return;
    };

    let container = match status
      .curr
      .services
      .iter()
      .find(|s| s.service == service)
      .map(|s| s.container.as_ref())
    {
      Some(Some(container)) => container.name.clone(),
      Some(None) => {
        let _ = client_socket
          .send(Message::text(format!(
            "ERROR: Service {service} container could not be found"
          )))
          .await;
        let _ = client_socket.close().await;
        return;
      }
      None => {
        let _ = client_socket
          .send(Message::text(format!(
            "ERROR: Service {service} could not be found"
          )))
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
