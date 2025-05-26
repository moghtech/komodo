use axum::{
  extract::{Query, WebSocketUpgrade, ws::Message},
  response::IntoResponse,
};
use futures::SinkExt;
use komodo_client::{
  api::terminal::ConnectDeploymentExecQuery,
  entities::{
    deployment::Deployment, permission::PermissionLevel,
    server::Server,
  },
};

use crate::{permission::get_check_permissions, resource::get};

#[instrument(name = "ConnectDeploymentExec", skip(ws))]
pub async fn terminal(
  Query(ConnectDeploymentExecQuery { deployment, shell }): Query<
    ConnectDeploymentExecQuery,
  >,
  ws: WebSocketUpgrade,
) -> impl IntoResponse {
  ws.on_upgrade(|socket| async move {
    let Some((mut client_socket, user)) =
      super::ws_login(socket).await
    else {
      return;
    };

    let deployment = match get_check_permissions::<Deployment>(
      &deployment,
      &user,
      PermissionLevel::Read.terminal(),
    )
    .await
    {
      Ok(deployment) => deployment,
      Err(e) => {
        debug!("could not get deployment | {e:#}");
        let _ = client_socket
          .send(Message::text(format!("ERROR: {e:#}")))
          .await;
        let _ = client_socket.close().await;
        return;
      }
    };

    let server =
      match get::<Server>(&deployment.config.server_id).await {
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
      deployment.name,
      shell,
    )
    .await
  })
}
