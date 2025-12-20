use std::time::Duration;

use anyhow::{Context, anyhow};
use periphery_client::{
  CONNECTION_RETRY_SECONDS, transport::LoginMessage,
};
use transport::{
  auth::{
    AddressConnectionIdentifiers, ClientLoginFlow,
    ConnectionIdentifiers,
  },
  fix_ws_address,
  websocket::{
    Websocket, WebsocketExt as _, login::LoginWebsocketExt,
    tungstenite::TungsteniteWebsocket,
  },
};

use crate::{
  config::{core_config, core_connection_query},
  periphery::PeripheryClient,
  state::periphery_connections,
};

use super::{PeripheryConnection, PeripheryConnectionArgs};

impl PeripheryConnectionArgs<'_> {
  pub async fn spawn_client_connection(
    self,
    id: String,
    insecure: bool,
  ) -> anyhow::Result<PeripheryClient> {
    let Some(address) = self.address else {
      return Err(anyhow!(
        "Cannot spawn client connection with empty address"
      ));
    };

    let address = fix_ws_address(address);
    let identifiers =
      AddressConnectionIdentifiers::extract(&address)?;
    let endpoint = format!("{address}/?{}", core_connection_query());

    let (connection, mut receiver) =
      periphery_connections().insert(id.clone(), self).await;

    let responses = connection.responses.clone();

    tokio::spawn(async move {
      loop {
        let ws = tokio::select! {
          ws = TungsteniteWebsocket::connect_maybe_tls_insecure(
            &endpoint,
            insecure && endpoint.starts_with("wss"),
          ) => ws,
          _ = connection.cancel.cancelled() => {
            break
          }
        };

        let (mut socket, accept) = match ws {
          Ok(res) => res,
          Err(e) => {
            connection.set_error(e.error).await;
            tokio::time::sleep(Duration::from_secs(
              CONNECTION_RETRY_SECONDS,
            ))
            .await;
            continue;
          }
        };

        let identifiers = identifiers.build(
          accept.as_bytes(),
          core_connection_query().as_bytes(),
        );

        if let Err(e) =
          connection.client_login(&mut socket, identifiers).await
        {
          connection.set_error(e).await;
          tokio::time::sleep(Duration::from_secs(
            CONNECTION_RETRY_SECONDS,
          ))
          .await;
          continue;
        };

        connection.handle_socket(socket, &mut receiver).await
      }
    });

    Ok(PeripheryClient { id, responses })
  }
}

impl PeripheryConnection {
  /// Custom Core -> Periphery side only login wrapper
  /// to implement passkey support for backward compatibility
  #[instrument(
    "PeripheryLogin",
    skip(self, socket, identifiers),
    fields(
      server_id = self.args.id,
      address = self.args.address,
      direction = "CoreToPeriphery"
    )
  )]
  async fn client_login(
    &self,
    socket: &mut TungsteniteWebsocket,
    identifiers: ConnectionIdentifiers<'_>,
  ) -> anyhow::Result<()> {
    // Get the required auth type
    let v1_passkey_flow =
      socket
        .recv_login_v1_passkey_flow()
        .await
        .context("Failed to receive Login V1PasskeyFlow message")?;

    if v1_passkey_flow {
      handle_passkey_login(socket, self.args.passkey.as_deref()).await
    } else {
      self
        .handle_login::<_, ClientLoginFlow>(socket, identifiers, true)
        .await
    }
  }
}

#[instrument("V1PasskeyPeripheryLoginFlow", skip(socket, passkey))]
async fn handle_passkey_login(
  socket: &mut TungsteniteWebsocket,
  // for legacy auth
  passkey: Option<&str>,
) -> anyhow::Result<()> {
  let res = async {
    let passkey = if let Some(passkey) = passkey {
      passkey.as_bytes().to_vec()
    } else {
      core_config()
        .passkey
        .as_deref()
        .context("Periphery requires passkey auth")?
        .as_bytes()
        .to_vec()
    };

    socket
      .send_message(LoginMessage::V1Passkey(passkey))
      .await
      .context("Failed to send Login V1Passkey message")?;

    // Receive login state message and return based on value
    socket
      .recv_login_success()
      .await
      .context("Failed to receive Login Success message")?;

    anyhow::Ok(())
  }
  .await;
  if let Err(e) = res {
    if let Err(e) = socket
      .send_login_error(&e)
      .await
      .context("Failed to send login failed to Periphery")
    {
      // Log additional error
      warn!("{e:#}");
    }
    // Close socket
    let _ = socket.close().await;
    // Return the original error
    Err(e)
  } else {
    Ok(())
  }
}
