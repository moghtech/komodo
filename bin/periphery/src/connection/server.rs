use std::{
  net::{IpAddr, SocketAddr},
  str::FromStr,
  sync::{
    OnceLock,
    atomic::{self, AtomicBool},
  },
};

use anyhow::{Context, anyhow};
use axum::{
  Router,
  body::Body,
  extract::{ConnectInfo, Query, WebSocketUpgrade},
  http::{HeaderMap, Request, StatusCode},
  middleware::{self, Next},
  response::Response,
  routing::get,
};
use axum_server::tls_rustls::RustlsConfig;
use periphery_client::{
  api::CoreConnectionQuery, transport::LoginMessage,
};
use serror::{AddStatusCode, AddStatusCodeError};
use transport::{
  auth::{
    ConnectionIdentifiers, HeaderConnectionIdentifiers,
    ServerLoginFlow,
  },
  websocket::{
    Websocket, WebsocketExt, axum::AxumWebsocket,
    login::LoginWebsocketExt,
  },
};

use crate::{config::periphery_config, state::core_connections};

#[instrument("RunCoreConnectionServer")]
pub async fn run()
-> anyhow::Result<tokio::task::JoinHandle<anyhow::Result<()>>> {
  let config = periphery_config();

  let addr = format!("{}:{}", config.bind_ip, config.port);

  let socket_addr = SocketAddr::from_str(&addr)
    .context("failed to parse listen address")?;

  let app = Router::new()
    .route("/version", get(|| async { env!("CARGO_PKG_VERSION") }))
    .route("/", get(crate::connection::server::handler))
    .layer(middleware::from_fn(guard_request_by_ip))
    .into_make_service_with_connect_info::<SocketAddr>();

  let handle = if config.ssl_enabled {
    info!("🔒 Periphery SSL Enabled");
    crate::helpers::ensure_ssl_certs().await;
    info!("Komodo Periphery starting on wss://{}", socket_addr);
    let ssl_config = RustlsConfig::from_pem_file(
      config.ssl_cert_file(),
      config.ssl_key_file(),
    )
    .await
    .context("Invalid ssl cert / key")?;
    tokio::spawn(async move {
      axum_server::bind_rustls(socket_addr, ssl_config)
        .serve(app)
        .await
        .context("Server crashed")
    })
  } else {
    info!("🔓 Periphery SSL Disabled");
    info!("Komodo Periphery starting on ws://{}", socket_addr);
    tokio::spawn(async move {
      axum_server::bind(socket_addr)
        .serve(app)
        .await
        .context("Server crashed")
    })
  };

  Ok(handle)
}

fn already_logged_login_error() -> &'static AtomicBool {
  static ALREADY_LOGGED: OnceLock<AtomicBool> = OnceLock::new();
  ALREADY_LOGGED.get_or_init(|| AtomicBool::new(false))
}

async fn handler(
  Query(CoreConnectionQuery { core }): Query<CoreConnectionQuery>,
  mut headers: HeaderMap,
  ws: WebSocketUpgrade,
) -> serror::Result<Response> {
  let identifiers =
    HeaderConnectionIdentifiers::extract(&mut headers)
      .status_code(StatusCode::UNAUTHORIZED)?;

  let channel = core_connections().get_or_insert_default(&core).await;

  // Ensure the receiver is free before upgrading connection.
  // Due to ownership, it needs to be re-locked inside the ws handler,
  // opening up a possible timing edge case, but should be good enough.
  drop(
    channel
      .receiver()
      .with_context(|| {
        format!("Connection for {core} is already active")
      })
      .inspect_err(|e| warn!("{e:#}"))?,
  );

  Ok(ws.on_upgrade(|socket| async move {
    let mut socket = AxumWebsocket(socket);

    // Make sure receiver locked over the login.
    let mut receiver = match channel.receiver() {
      Ok(receiver) => receiver,
      Err(e) => {
        warn!("Failed to forward connection | {e:#}");

        if let Err(e) = socket
          .send_login_error(&e)
          .await
          .context("Failed to send forward failed to client")
        {
          // Log additional error
          warn!("{e:#}");
        }

        // Close socket
        let _ = socket.close().await;

        return;
      }
    };

    let query = format!("core={}", urlencoding::encode(&core));

    if let Err(e) =
      handle_login(&mut socket, identifiers.build(query.as_bytes()))
        .await
    {
      let already_logged = already_logged_login_error();
      if !already_logged.load(atomic::Ordering::Relaxed) {
        warn!("Core failed to login to connection | {e:#}");
        already_logged.store(true, atomic::Ordering::Relaxed);
      }
      // End the connection
      return;
    }

    already_logged_login_error()
      .store(false, atomic::Ordering::Relaxed);

    super::handle_socket(
      socket,
      &core,
      &channel.sender,
      &mut receiver,
    )
    .await
  }))
}

/// Custom Core -> Periphery side only login wrapper
/// to implement passkey support for backward compatibility
#[instrument(
  "CoreLogin",
  skip(socket, identifiers),
  fields(direction = "CoreToPeriphery")
)]
async fn handle_login(
  socket: &mut AxumWebsocket,
  identifiers: ConnectionIdentifiers<'_>,
) -> anyhow::Result<()> {
  let config = periphery_config();
  match (&config.core_public_keys, &config.passkeys) {
    (Some(_), _) | (_, None) => {
      socket
        .send_message(LoginMessage::V1PasskeyFlow(false))
        .await
        .context("Failed to send Login V1PasskeyFlow message")?;
      super::handle_login::<_, ServerLoginFlow>(
        socket,
        identifiers,
        true,
      )
      .await
    }
    (None, Some(passkeys)) => {
      handle_passkey_login(socket, passkeys).await
    }
  }
}

#[instrument("V1PasskeyCoreLoginFlow", skip(socket, passkeys))]
async fn handle_passkey_login(
  socket: &mut AxumWebsocket,
  passkeys: &[String],
) -> anyhow::Result<()> {
  if !already_logged_login_error().load(atomic::Ordering::Relaxed) {
    warn!(
      "Authenticating using Passkeys. Set 'core_public_key' (PERIPHERY_CORE_PUBLIC_KEY) instead to enhance security."
    );
  };
  let res = async {
    socket
      .send_message(LoginMessage::V1PasskeyFlow(true))
      .await
      .context("Failed to send login type indicator")?;

    // Receieve passkey
    let passkey = socket
      .recv_login_v1_passkey()
      .await
      .context("Failed to receive Login V1Passkey from Core")?;

    if passkeys
      .iter()
      .any(|expected_passkey| expected_passkey.as_bytes() == passkey)
    {
      socket
        .send_message(LoginMessage::Success)
        .await
        .context("Failed to send login type indicator")?;
      Ok(())
    } else {
      let e = anyhow!("Invalid passkey");
      if let Err(e) = socket
        .send_login_error(&e)
        .await
        .context("Failed to send login failed")
      {
        // Log additional error
        warn!("{e:#}");
        // Close socket
        let _ = socket.close().await;
      }
      // Return the original error
      Err(e)
    }
  }
  .await;
  if let Err(e) = res {
    if let Err(e) = socket
      .send_login_error(&e)
      .await
      .context("Failed to send login failed to Core")
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

async fn guard_request_by_ip(
  req: Request<Body>,
  next: Next,
) -> serror::Result<Response> {
  if periphery_config().allowed_ips.is_empty() {
    return Ok(next.run(req).await);
  }
  let ConnectInfo(socket_addr) = req
    .extensions()
    .get::<ConnectInfo<SocketAddr>>()
    .context("could not get ConnectionInfo of request")
    .status_code(StatusCode::UNAUTHORIZED)?;
  let ip = socket_addr.ip();

  let ip_match = periphery_config().allowed_ips.iter().any(|net| {
    net.contains(ip)
      || match ip {
        IpAddr::V4(ipv4) => {
          net.contains(IpAddr::V6(ipv4.to_ipv6_mapped()))
        }
        IpAddr::V6(_) => net.contains(ip.to_canonical()),
      }
  });

  if ip_match {
    Ok(next.run(req).await)
  } else {
    Err(
      anyhow!("requesting ip {ip} not allowed")
        .status_code(StatusCode::UNAUTHORIZED),
    )
  }
}
