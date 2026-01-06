use std::{
  net::{IpAddr, SocketAddr},
  str::FromStr,
};

pub mod auth;
pub mod channel;
pub mod timeout;
pub mod websocket;

/// - Fixes ws addresses:
///   - `11.11.11.11:9120` => `ws://11.11.11.11:9120`
///   - `server.domain` => `wss://server.domain`
///   - `http://server.domain` => `ws://server.domain`
///   - `https://server.domain` => `wss://server.domain`
pub fn fix_ws_address(address: &str) -> String {
  if address.starts_with("ws://") || address.starts_with("wss://") {
    return address.to_string();
  }
  if address.starts_with("http://") {
    return address.replace("http://", "ws://");
  }
  if address.starts_with("https://") {
    return address.replace("https://", "wss://");
  }
  // When using direct IPs, always use ws://
  if SocketAddr::from_str(address).is_ok()
    || IpAddr::from_str(address).is_ok()
  {
    return format!("ws://{address}");
  }
  format!("wss://{address}")
}
