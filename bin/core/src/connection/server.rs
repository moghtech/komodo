use std::{str::FromStr, time::Duration};

use anyhow::{Context, anyhow};
use axum::{
  extract::{Query, WebSocketUpgrade},
  http::{HeaderMap, StatusCode},
  response::Response,
};
use database::mungos::mongodb::bson::{doc, oid::ObjectId};
use komodo_client::{
  api::write::{
    CreateBuilder, CreateServer, UpdateResourceMeta, UpdateServer,
    UpdateServerPublicKey,
  },
  entities::{
    builder::{PartialBuilderConfig, PartialServerBuilderConfig},
    komodo_timestamp,
    onboarding_key::OnboardingKey,
    server::{PartialServerConfig, Server},
    user::system_user,
  },
};
use mogh_error::{AddStatusCode, AddStatusCodeError};
use mogh_resolver::Resolve;
use partial_derive2::MaybeNone;
use periphery_client::{
  api::PeripheryConnectionQuery, transport::LoginMessage,
};
use tracing::Instrument;
use transport::{
  auth::{
    HeaderConnectionIdentifiers, LoginFlow, LoginFlowArgs,
    PublicKeyValidator, ServerLoginFlow,
  },
  websocket::{
    Websocket, WebsocketExt as _, axum::AxumWebsocket,
    login::LoginWebsocketExt,
  },
};

use crate::{
  api::write::WriteArgs,
  config::core_keys,
  helpers::query::id_or_name_filter,
  monitor::refresh_server_cache,
  resource::KomodoResource,
  state::{db_client, periphery_connections},
};

use super::PeripheryConnectionArgs;

pub async fn handler(
  Query(PeripheryConnectionQuery {
    server: server_query,
  }): Query<PeripheryConnectionQuery>,
  mut headers: HeaderMap,
  ws: WebSocketUpgrade,
) -> mogh_error::Result<Response> {
  let identifiers =
    HeaderConnectionIdentifiers::extract(&mut headers)
      .status_code(StatusCode::UNAUTHORIZED)?;

  if server_query.is_empty() {
    return Err(
      anyhow!("Must provide non-empty server specifier")
        .status_code(StatusCode::UNAUTHORIZED),
    );
  }

  // Handle connection vs. onboarding flow.
  match Server::coll()
    .find_one(id_or_name_filter(&server_query))
    .await
    .context("Failed to query database for Server")?
  {
    Some(server) => {
      let connections = periphery_connections();

      // Ensure connected server can't get bumped off the connection.
      if let Some(existing_connection) = connections.get(&server.id).await
        && existing_connection.connected()
      {
        return Err(
          anyhow!("A Server '{server_query}' is already connected")
            .status_code(StatusCode::UNAUTHORIZED),
        );
      }

      if server.config.enabled && server.config.address.is_empty() {
        existing_server_handler(server_query, server, identifiers, ws)
          .await
      } else {
        fix_existing_server_handler(server_query, server, identifiers, ws).await
      }
    }
    None if ObjectId::from_str(&server_query).is_err() => {
      onboard_new_server_handler(server_query, identifiers, ws).await
    }
    None => Err(
      anyhow!("Must provide name based Server specifier for onboarding flow, name cannot be valid ObjectId (hex)")
        .status_code(StatusCode::UNAUTHORIZED),
    ),
  }
}

async fn existing_server_handler(
  server_query: String,
  server: Server,
  identifiers: HeaderConnectionIdentifiers,
  ws: WebSocketUpgrade,
) -> mogh_error::Result<Response> {
  let (connection, mut receiver) = periphery_connections()
    .insert(
      server.id.clone(),
      PeripheryConnectionArgs::from_server(&server),
    )
    .await;

  Ok(ws.on_upgrade(|socket| async move {
    let query =
      format!("server={}", urlencoding::encode(&server_query));
    let mut socket = AxumWebsocket(socket);

    if let Err(e) = socket
      .send_message(LoginMessage::OnboardingFlow(false))
      .await
      .context("Failed to send Login OnboardingFlow false message")
    {
      connection.set_error(e).await;
      return;
    };

    debug!(
      host = identifiers.host.to_str().unwrap_or_default(),
      query,
      sec_websocket_accept = identifiers.accept,
      resource_id = &server.id,
      "[PERIPHERY AUTH] Zero trust identifiers"
    );

    let span = info_span!(
      "PeripheryLogin",
      server_id = server.id,
      direction = "PeripheryToCore"
    );
    let login = async {
      connection
        .handle_login::<_, ServerLoginFlow>(
          &mut socket,
          identifiers.build(query.as_bytes()),
          false,
        )
        .await
    }
    .instrument(span)
    .await;

    if let Err(login_error) = login {
      // First need to receive login error msg from Periphery
      // so onboarding flow works correctly.
      let _ = socket.recv_login_message().await;

      if let Err(e) = ServerLoginFlow::login(LoginFlowArgs {
        socket: &mut socket,
        identifiers: identifiers.build(query.as_bytes()),
        private_key: core_keys().load().private.as_str(),
        public_key_validator: OnboardingKeyValidator { privileged_required: true },
        should_close: true
      })
      .await
      {
        debug!("Server {server_query} has invalid public key and failed to onboard | {e:#}");
        connection.set_error(login_error).await;
        return;
      };

      // Post onboarding login 1: Receive public key
      let public_key = match socket
        .recv_login_public_key()
        .await
      {
        Ok(public_key) => public_key,
        Err(e) => {
          warn!("Server {server_query} failed to onboard | failed to receive Server public key | {e:#}");
          connection.set_error(login_error).await;
          return;
        }
      };

      if let Err(e) = fix_server(server, public_key.into_inner()).await {
        warn!("Server {server_query} failed to onboard | Failed to fix Server config on database | {e:#}");
        if let Err(e) = socket.send_login_error(&e).await {
          warn!("Server {server_query} onboarding notice | Failed to send onboarding error to Periphery | {e:#}")
        }
        return;
      }

      if let Err(e) = socket.send_message(LoginMessage::Success).await {
        warn!("Server {server_query} onboarding notice | Failed to send onboarding successful | {e:#}")
      }

      return;
    }

    // Waits until after connection is handled then
    // force refreshes the server cache.
    tokio::spawn(async move {
      tokio::time::sleep(Duration::from_millis(100)).await;
      refresh_server_cache(&server, true).await;
    });

    connection.handle_socket(socket, &mut receiver).await
  }))
}

async fn fix_existing_server_handler(
  server_query: String,
  server: Server,
  identifiers: HeaderConnectionIdentifiers,
  ws: WebSocketUpgrade,
) -> mogh_error::Result<Response> {
  Ok(ws.on_upgrade(|socket| async move {
    let query =
      format!("server={}", urlencoding::encode(&server_query));
    let mut socket = AxumWebsocket(socket);

    if let Err(e) = socket.send_message(LoginMessage::OnboardingFlow(true)).await.context(
      "Failed to send Login OnboardingFlow true message",
    ).context("Server onboarding error") {
      warn!("{e:#}");
      return;
    };

    if let Err(e) = ServerLoginFlow::login(LoginFlowArgs {
      socket: &mut socket,
      identifiers: identifiers.build(query.as_bytes()),
      private_key: core_keys().load().private.as_str(),
      public_key_validator: OnboardingKeyValidator { privileged_required: true },
      should_close: true,
    })
    .await
    {
      debug!("Server {server_query} failed to onboard | {e:#}");
      return;
    };

    // Post onboarding login 1: Receive public key
    let public_key = match socket
      .recv_login_public_key()
      .await
    {
      Ok(public_key) => public_key,
      Err(e) => {
        warn!("Server {server_query} failed to onboard | failed to receive Server public key | {e:#}");
        return;
      }
    };

    if let Err(e) = fix_server(server, public_key.into_inner()).await {
      warn!("Server {server_query} failed to onboard | failed to receive Server public key | {e:#}");
    }

    if let Err(e) = socket
      .send_message(LoginMessage::Success)
      .await
      .context("Failed to send Login Onboarding Successful message")
    {
      // Log additional error
      warn!("{e:#}");
    }

    // Server fixed, close and trigger reconnect.
    // The next time the server connects it should work.
    let _ = socket.close().await;
  }))
}

async fn fix_server(
  server: Server,
  public_key: String,
) -> anyhow::Result<()> {
  let args = WriteArgs {
    user: system_user().to_owned(),
  };
  // Fix public key if needed
  if server.info.public_key != public_key {
    UpdateServerPublicKey {
      server: server.id.clone(),
      public_key,
    }
    .resolve(&args)
    .await
    .map_err(|e| {
      e.error.context(
        "Server onboarding flow failed at updating Server public key."
      )
    })?;
  }
  let config = PartialServerConfig {
    enabled: if !server.config.enabled {
      Some(true)
    } else {
      None
    },
    address: if !server.config.address.is_empty() {
      Some(String::new())
    } else {
      None
    },
    // Move address to external_address if not set.
    // This helps preserve container port link behavior.
    external_address: if !server.config.external_address.is_empty() {
      Some(server.config.address)
    } else {
      None
    },
    ..Default::default()
  };
  if !config.is_none() {
    UpdateServer {
      id: server.id,
      config: PartialServerConfig {
        enabled: Some(true),
        address: Some(String::default()),
        ..Default::default()
      },
    }
    .resolve(&args)
    .await
    .map_err(|e| {
      e.error
        .context("Server onboarding flow failed at updating Server.")
    })?;
  }
  Ok(())
}

async fn onboard_new_server_handler(
  server_query: String,
  identifiers: HeaderConnectionIdentifiers,
  ws: WebSocketUpgrade,
) -> mogh_error::Result<Response> {
  Ok(ws.on_upgrade(|socket| async move {
    let query =
      format!("server={}", urlencoding::encode(&server_query));
    let mut socket = AxumWebsocket(socket);

    if let Err(e) = socket.send_message(LoginMessage::OnboardingFlow(true)).await.context(
      "Failed to send Login OnboardingFlow true message",
    ).context("Server onboarding error") {
      warn!("{e:#}");
      return;
    };

    let onboarding_key = match ServerLoginFlow::login(LoginFlowArgs {
      socket: &mut socket,
      identifiers: identifiers.build(query.as_bytes()),
      private_key: core_keys().load().private.as_str(),
      public_key_validator: OnboardingKeyValidator { privileged_required: false },
      should_close: true
    })
    .await
    {
      Ok(onboarding_key) => onboarding_key,
      Err(e) => {
        debug!("Server {server_query} failed to onboard | {e:#}");
        return;
      }
    };

    // Post onboarding login 1: Receive public key
    let public_key = match socket
      .recv_login_public_key()
      .await
    {
      Ok(public_key) => public_key,
      Err(e) => {
        warn!("Server {server_query} failed to onboard | failed to receive Server public key | {e:#}");
        return;
      }
    };

    let server_id = match create_server_maybe_builder(
      server_query,
      public_key.into_inner(),
      onboarding_key.copy_server,
      onboarding_key.tags,
      onboarding_key.create_builder
    ).await {
      Ok(server_id) => server_id,
      Err(e) => {
        warn!("{e:#}");
        if let Err(e) = socket
          .send_login_error(&e)
          .await
          .context("Failed to send Server creation failed to client")
        {
          // Log additional error
          warn!("{e:#}");
        }
        return;
      }
    };

    if let Err(e) = socket
      .send_message(LoginMessage::Success)
      .await
      .context("Failed to send Login Onboarding Successful message")
    {
      // Log additional error
      warn!("{e:#}");
    }

    // Server created, close and trigger reconnect
    // and handling using existing server handler.
    let _ = socket.close().await;

    // Add the server to onboarding key "Onboarded"
    let res = db_client()
      .onboarding_keys
      .update_one(
        doc! { "public_key": &onboarding_key.public_key },
        doc! { "$push": { "onboarded": server_id } },
      ).await;
    if let Err(e) = res {
      warn!("Failed to update onboarding key 'onboarded' | {e:?}");
    }
  }))
}

async fn create_server_maybe_builder(
  server_query: String,
  public_key: String,
  copy_server: String,
  tags: Vec<String>,
  create_builder: bool,
) -> anyhow::Result<String> {
  let config = if copy_server.is_empty() {
    PartialServerConfig {
      enabled: Some(true),
      ..Default::default()
    }
  } else {
    let config = match db_client()
      .servers
      .find_one(id_or_name_filter(&copy_server))
      .await
    {
      Ok(Some(server)) => server.config,
      Ok(None) => {
        warn!(
          "Server onboarding: Failed to find Server {}",
          copy_server
        );
        Default::default()
      }
      Err(e) => {
        warn!(
          "Failed to query database for onboarding key 'copy_server' | {e:?}"
        );
        Default::default()
      }
    };
    PartialServerConfig {
      enabled: Some(true),
      address: None,
      ..config.into()
    }
  };

  let args = WriteArgs {
    user: system_user().to_owned(),
  };

  let server = CreateServer {
    name: server_query.clone(),
    config,
    public_key: Some(public_key),
  }
  .resolve(&args)
  .await
  .map_err(|e| e.error)
  .context("Server onboarding flow failed at Server creation")?;

  // Don't need to fail, only warn on this
  if let Err(e) = (UpdateResourceMeta {
    target: (&server).into(),
    tags: Some(tags),
    description: None,
    template: None,
  })
  .resolve(&args)
  .await
  .map_err(|e| e.error)
  .context("Server onboarding flow failed at Server creation")
  {
    warn!("{e:#}");
  };

  if create_builder {
    // Don't need to fail, only warn on this
    if let Err(e) = (CreateBuilder {
      name: server_query,
      config: PartialBuilderConfig::Server(
        PartialServerBuilderConfig {
          server_id: Some(server.id.clone()),
        },
      ),
    })
    .resolve(&args)
    .await
    .map_err(|e| e.error)
    .context("Server onboarding flow failed at Builder creation")
    {
      warn!("{e:#}");
    };
  }

  Ok(server.id)
}

struct OnboardingKeyValidator {
  privileged_required: bool,
}

impl PublicKeyValidator for OnboardingKeyValidator {
  type ValidationResult = OnboardingKey;
  async fn validate(
    &self,
    public_key: String,
  ) -> anyhow::Result<Self::ValidationResult> {
    let onboarding_key = db_client()
      .onboarding_keys
      .find_one(doc! { "public_key": &public_key })
      .await
      .context("Failed to query database for Server onboarding keys")?
      .context("Matching Server onboarding key not found")?;
    // Check enabled and not expired.
    if !onboarding_key.enabled
      || (onboarding_key.expires != 0
        && onboarding_key.expires <= komodo_timestamp())
    {
      return Err(anyhow!("Onboarding key is invalid"));
    }
    if self.privileged_required && !onboarding_key.privileged {
      return Err(anyhow!(
        "Onboarding key not able to fix existing servers"
      ));
    }
    Ok(onboarding_key)
  }
}
