use std::time::Duration;

use anyhow::anyhow;
use encoding::{
  CastBytes as _, Decode as _, Encode as _, WithChannel,
};
use periphery_client::transport::{
  EncodedRequestMessage, EncodedTransportMessage, RequestMessage,
  TransportMessage,
};
use resolver_api::Resolve;
use transport::{
  auth::{
    ConnectionIdentifiers, LoginFlow, LoginFlowArgs,
    PublicKeyValidator,
  },
  channel::{BufferedReceiver, Sender},
  websocket::{
    Websocket, WebsocketReceiverExt as _, WebsocketSender as _,
  },
};

use crate::{
  api::{Args, PeripheryRequest},
  config::periphery_config,
  state::{CorePublicKeys, core_public_keys, periphery_keys},
};

pub mod client;
pub mod server;

impl PublicKeyValidator for &CorePublicKeys {
  type ValidationResult = ();
  #[instrument("ValidateCorePublicKey", skip(self))]
  async fn validate(&self, public_key: String) -> anyhow::Result<()> {
    if self.is_valid(&public_key).await {
      Ok(())
    } else {
      Err(
        anyhow!("{public_key} is invalid")
          .context("Ensure public key matches one of the 'core_public_keys' in periphery config (PERIPHERY_CORE_PUBLIC_KEYS)")
          .context("Periphery failed to validate Core public key"),
      )
    }
  }
}

#[instrument("StandardCoreLoginFlow", skip(socket, identifiers))]
async fn handle_login<W: Websocket, L: LoginFlow>(
  socket: &mut W,
  identifiers: ConnectionIdentifiers<'_>,
  should_close: bool,
) -> anyhow::Result<()> {
  L::login(LoginFlowArgs {
    socket,
    identifiers,
    private_key: periphery_keys().load().private.as_str(),
    public_key_validator: core_public_keys(),
    should_close,
  })
  .await
}

async fn handle_socket<W: Websocket>(
  socket: W,
  core: &str,
  sender: &Sender<EncodedTransportMessage>,
  receiver: &mut BufferedReceiver<EncodedTransportMessage>,
) {
  let config = periphery_config();
  info!(
    "Logged in to Komodo Core {core} websocket{}",
    if !config.core_addresses.is_empty()
      && !config.connect_as.is_empty()
    {
      format!(" as Server {}", config.connect_as)
    } else {
      String::new()
    }
  );

  let (mut ws_write, mut ws_read) = socket.split();

  let forward_writes = async {
    loop {
      let message = match tokio::time::timeout(
        Duration::from_secs(5),
        receiver.recv(),
      )
      .await
      {
        Ok(Ok(message)) => message,
        Ok(Err(_)) => break,
        // Handle sending Ping
        Err(_) => {
          if let Err(e) = ws_write.ping().await {
            warn!("Failed to send ping | {e:?}");
            break;
          }
          continue;
        }
      };
      match ws_write.send(message.into_bytes()).await {
        // Clears the stored message from receiver buffer.
        Ok(_) => receiver.clear_buffer(),
        Err(e) => {
          warn!("Failed to send response | {e:?}");
          break;
        }
      }
    }
    let _ = ws_write.close().await;
  };

  let handle_reads = async {
    loop {
      let message = match ws_read.recv_message().await {
        Ok(res) => res,
        Err(e) => {
          warn!("{e:#}");
          break;
        }
      };
      match message {
        TransportMessage::Request(message) => {
          handle_request(core.to_string(), sender.clone(), message)
        }
        TransportMessage::Terminal(message) => {
          crate::terminal::handle_message(message).await
        }
        // Rest shouldn't be received by Periphery
        _ => {}
      }
    }
  };

  tokio::select! {
    _ = forward_writes => {},
    _ = handle_reads => {},
  }
}

fn handle_request(
  core: String,
  sender: Sender<EncodedTransportMessage>,
  message: EncodedRequestMessage,
) {
  tokio::spawn(async move {
    let WithChannel {
      channel,
      data: request,
    }: WithChannel<PeripheryRequest> =
      match message.decode().and_then(RequestMessage::map_decode) {
        Ok(res) => res,
        Err(e) => {
          // TODO: handle:
          warn!("Failed to parse Request bytes | {e:#}");
          return;
        }
      };

    let resolve_response = async {
      let response =
        match request.resolve(&Args { core, id: channel }).await {
          Ok(res) => res,
          Err(e) => (&e).encode(),
        };
      if let Err(e) = sender.send_response(channel, response).await {
        error!("Failed to send response over channel | {e:?}");
      }
    };

    let ping_in_progress = async {
      loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        if let Err(e) = sender.send_in_progress(channel).await {
          error!("Failed to ping in progress over channel | {e:?}");
        }
      }
    };

    tokio::select! {
      _ = resolve_response => {},
      _ = ping_in_progress => {},
    }
  });
}
