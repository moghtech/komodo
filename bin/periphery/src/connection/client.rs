use std::time::Duration;

use anyhow::{Context, anyhow};
use axum::http::{HeaderValue, StatusCode};
use periphery_client::{
  CONNECTION_RETRY_SECONDS, transport::LoginMessage,
};
use tracing::Instrument;
use transport::{
  auth::{
    AddressConnectionIdentifiers, ClientLoginFlow,
    ConnectionIdentifiers, LoginFlow, LoginFlowArgs,
  },
  fix_ws_address,
  websocket::{
    WebsocketExt, login::LoginWebsocketExt,
    tungstenite::TungsteniteWebsocket,
  },
};

use crate::{
  config::periphery_config,
  connection::core_public_keys,
  state::{core_connections, periphery_keys},
};

#[instrument("StartCoreConnection")]
pub async fn handler(
  address: &str,
) -> anyhow::Result<tokio::task::JoinHandle<anyhow::Result<()>>> {
  let config = periphery_config();
  let address = fix_ws_address(address);
  let identifiers = AddressConnectionIdentifiers::extract(&address)?;
  let query =
    format!("server={}", urlencoding::encode(&config.connect_as));
  let endpoint = format!("{address}/ws/periphery?{query}");

  info!("Initiating outbound connection to {endpoint}");

  let mut already_logged_connection_error = false;
  let mut already_logged_login_error = false;
  let mut already_logged_onboarding_error = false;

  let core = identifiers.host().to_string();

  let channel = core_connections().get_or_insert_default(&core).await;

  let handle = tokio::spawn(async move {
    let mut receiver = channel.receiver()?;
    loop {
      let (mut socket, accept) =
        match connect_websocket(&endpoint).await {
          Ok(res) => res,
          Err(e) => {
            if !already_logged_connection_error {
              warn!("{e:#}");
              already_logged_connection_error = true;
              // If error transitions from login to connection,
              // set to false to see login error after reconnect.
              already_logged_login_error = false;
              already_logged_onboarding_error = false;
            }
            tokio::time::sleep(Duration::from_secs(
              CONNECTION_RETRY_SECONDS,
            ))
            .await;
            continue;
          }
        };

      // Receive whether to use Server connection flow vs Server onboarding flow.
      let onboarding_flow = match socket
        .recv_login_onboarding_flow()
        .await
        .context("Failed to receive Login OnboardingFlow message")
      {
        Ok(onboarding_flow) => onboarding_flow,
        Err(e) => {
          if !already_logged_connection_error {
            warn!("{e:#}");
            already_logged_connection_error = true;
            // If error transitions from login to connection,
            // set to false to see login error after reconnect.
            already_logged_login_error = false;
            already_logged_onboarding_error = false;
          }
          tokio::time::sleep(Duration::from_secs(
            CONNECTION_RETRY_SECONDS,
          ))
          .await;
          continue;
        }
      };

      already_logged_connection_error = false;

      let identifiers =
        identifiers.build(accept.as_bytes(), query.as_bytes());

      if onboarding_flow {
        if let Err(e) = handle_onboarding(socket, identifiers).await.map(|onboarding_key| if onboarding_key {
          Ok(())
        } else {
          Err(anyhow!("Server '{}' does not exist or is misconfigured, and no PERIPHERY_ONBOARDING_KEY is provided.", config.connect_as))
        }) {
          if !already_logged_onboarding_error {
            error!("{e:#}");
            already_logged_onboarding_error = true;
          }
          tokio::time::sleep(Duration::from_secs(
            CONNECTION_RETRY_SECONDS,
          ))
          .await;
          continue;
        };
      } else {
        let span = info_span!(
          "CoreLogin",
          address,
          direction = "PeripheryToCore",
        );
        let login = async {
          super::handle_login::<_, ClientLoginFlow>(
            &mut socket,
            identifiers,
            false,
          )
          .await
        }
        .instrument(span)
        .await;
        if let Err(e) = login {
          // Try using onboarding key to fix public key issue.
          let e = match handle_onboarding(socket, identifiers).await {
            // Should work on next reconnect
            Ok(true) => continue,
            // No onboarding key available, use original error.
            Ok(false) => e,
            // Onboarding key available but failed.
            Err(e) => e,
          };
          if !already_logged_login_error {
            warn!("Failed to login | {e:#}");
            already_logged_login_error = true;
          }
          tokio::time::sleep(Duration::from_secs(
            CONNECTION_RETRY_SECONDS,
          ))
          .await;
          continue;
        }

        already_logged_login_error = false;

        super::handle_socket(
          socket,
          &core,
          &channel.sender,
          &mut receiver,
        )
        .await
      }
    }
  });

  Ok(handle)
}

#[instrument("OnboardingFlow", skip_all)]
async fn handle_onboarding(
  mut socket: TungsteniteWebsocket,
  identifiers: ConnectionIdentifiers<'_>,
) -> anyhow::Result<bool> {
  let config = periphery_config();
  let Some(onboarding_key) = config.onboarding_key.as_deref() else {
    return Ok(false);
  };

  // .with_context(|| format!("Server '{}' does not exist or is misconfigured, and no PERIPHERY_ONBOARDING_KEY is provided.", config.connect_as))?;

  ClientLoginFlow::login(LoginFlowArgs {
    private_key: onboarding_key,
    identifiers,
    public_key_validator: core_public_keys(),
    socket: &mut socket,
    should_close: true,
  })
  .await
  .context("Onboarding failed")?;

  // Post onboarding login 1: Send public key
  socket
    .send_message(LoginMessage::PublicKey(
      periphery_keys().load().public.clone(),
    ))
    .await
    .context("Failed to send public key bytes")?;

  socket
    .recv_login_success()
    .await
    .context("Failed to receive Server onboarding result")?;

  info!(
    "Server onboarding flow for '{}' successful ✅",
    config.connect_as
  );

  Ok(true)
}

async fn connect_websocket(
  url: &str,
) -> anyhow::Result<(TungsteniteWebsocket, HeaderValue)> {
  let config = periphery_config();
  TungsteniteWebsocket::connect_maybe_tls_insecure(url, config.core_tls_insecure_skip_verify)
    .await
    .map_err(|e| match e.status {
      StatusCode::NOT_FOUND => anyhow!("404 Not Found: Server '{}' does not exist.", config.connect_as),
      StatusCode::BAD_REQUEST => anyhow!("400 Bad Request: Server '{}' is disabled or configured to make Core → Periphery connection", config.connect_as),
      StatusCode::UNAUTHORIZED => anyhow!("401 Unauthorized: Only one Server connected as '{}' is allowed. Or the Core reverse proxy needs to forward host and websocket headers.", config.connect_as),
      _ => e.error,
    })
}
