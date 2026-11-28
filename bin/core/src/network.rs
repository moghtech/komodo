//! # Network Configuration Module
//!
//! This module provides manual network interface configuration for multi-NIC Docker environments.
//! It allows Komodo Core to specify which network interface should be used as the default route
//! for internet traffic, which is particularly useful in complex networking setups with multiple
//! network interfaces.
//!
//! ## Features
//! - Automatic container environment detection
//! - Interface validation (existence and UP state)
//! - Gateway discovery from routing tables or network configuration
//! - Atomic default route replacement (never leaves the container without a default route)
//! - Comprehensive error handling and logging

use std::net::Ipv4Addr;

use anyhow::{Context, anyhow};
use tokio::process::Command;
use tracing::{debug, info, trace, warn};

/// Container environment detection files
const DOCKERENV_FILE: &str = "/.dockerenv";
const CONTAINERENV_FILE: &str = "/run/.containerenv";
const CGROUP_FILE: &str = "/proc/1/cgroup";

/// Check if running in container environment
fn is_container_environment() -> bool {
  // Check for Docker / Podman specific indicators
  if std::path::Path::new(DOCKERENV_FILE).exists()
    || std::path::Path::new(CONTAINERENV_FILE).exists()
  {
    return true;
  }

  // Check container environment variable
  if std::env::var("container").is_ok() {
    return true;
  }

  // Check cgroup for container runtime indicators.
  // Only reliable on cgroup v1 - on cgroup v2 the file
  // is usually just `0::/`, so the checks above carry detection.
  if let Ok(content) = std::fs::read_to_string(CGROUP_FILE)
    && (content.contains("docker") || content.contains("containerd"))
  {
    return true;
  }

  false
}

/// Configure internet gateway for specified interface
pub async fn configure_internet_gateway() {
  use crate::config::core_config;

  let config = core_config();

  if config.internet_interface.is_empty() {
    debug!("No interface specified, using default routing");
    return;
  }

  if !is_container_environment() {
    debug!("Not in container, skipping network configuration");
    return;
  }

  debug!(
    "Configuring internet interface: {}",
    config.internet_interface
  );
  if let Err(e) =
    configure_manual_interface(&config.internet_interface).await
  {
    warn!("Failed to configure internet gateway: {e:#}");
  }
}

/// Configure interface as default route
async fn configure_manual_interface(
  interface_name: &str,
) -> anyhow::Result<()> {
  // Verify interface exists and is up
  let interface_check = Command::new("ip")
    .args(["addr", "show", interface_name])
    .output()
    .await
    .context("Failed to check interface status")?;

  if !interface_check.status.success() {
    return Err(anyhow!(
      "Interface '{}' does not exist or is not accessible. Available interfaces can be listed with 'ip addr show'",
      interface_name
    ));
  }

  let interface_info =
    String::from_utf8_lossy(&interface_check.stdout);
  // tun / wireguard style interfaces report `state UNKNOWN` while operational
  if !interface_info.contains("state UP")
    && !interface_info.contains("state UNKNOWN")
  {
    return Err(anyhow!(
      "Interface '{}' is not UP. Please ensure the interface is enabled and connected",
      interface_name
    ));
  }

  debug!("Interface {} is UP", interface_name);

  let gateway = find_gateway(interface_name).await?;
  debug!("Found gateway {} for {}", gateway, interface_name);

  set_default_gateway(&gateway, interface_name).await?;
  info!(
    "🌐 Configured {} as default gateway via {}",
    interface_name, gateway
  );
  Ok(())
}

/// Find gateway for interface
async fn find_gateway(
  interface_name: &str,
) -> anyhow::Result<String> {
  // Get interface IP address
  let addr_output = Command::new("ip")
    .args(["addr", "show", interface_name])
    .output()
    .await
    .context("Failed to get interface address")?;

  let addr_info = String::from_utf8_lossy(&addr_output.stdout);
  let mut ip_cidr = None;

  // Extract IP/CIDR from interface info
  for line in addr_info.lines() {
    if line.trim().starts_with("inet ") && !line.contains("127.0.0.1")
    {
      let parts: Vec<&str> = line.split_whitespace().collect();
      if let Some(found_ip_cidr) = parts.get(1) {
        debug!(
          "Interface {} has IP {}",
          interface_name, found_ip_cidr
        );
        ip_cidr = Some(*found_ip_cidr);
        break;
      }
    }
  }

  let ip_cidr = ip_cidr.ok_or_else(|| anyhow!(
        "Could not find IP address for interface '{}'. Ensure interface has a valid IPv4 address",
        interface_name
    ))?;

  trace!(
    "Finding gateway for interface {} in network {}",
    interface_name, ip_cidr
  );

  // Try to find gateway from routing table
  let route_output = Command::new("ip")
    .args(["route", "show", "dev", interface_name])
    .output()
    .await
    .context("Failed to get routes for interface")?;

  if route_output.status.success() {
    let routes = String::from_utf8_lossy(&route_output.stdout);
    trace!("Routes for {}: {}", interface_name, routes.trim());

    // Look for routes with gateway
    for line in routes.lines() {
      if line.contains("via") {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if let Some(via_idx) = parts.iter().position(|&x| x == "via")
          && let Some(&gateway) = parts.get(via_idx + 1)
        {
          trace!(
            "Found gateway {} for {} from routing table",
            gateway, interface_name
          );
          return Ok(gateway.to_string());
        }
      }
    }
  }

  // Derive candidates from the interface subnet
  let candidates = derive_gateway_candidates(ip_cidr)
    .with_context(|| {
      format!(
        "Could not determine gateway for interface '{interface_name}' in network '{ip_cidr}'"
      )
    })?;

  for gateway in &candidates {
    trace!(
      "Testing potential gateway {} for {}",
      gateway, interface_name
    );

    // Note: `ip route get` only confirms the address is on-link
    // for this interface, not that a router actually answers there.
    let route_test = Command::new("ip")
      .args(["route", "get", gateway, "dev", interface_name])
      .output()
      .await;

    if let Ok(output) = route_test
      && output.status.success()
    {
      trace!("Gateway {} is on-link via {}", gateway, interface_name);
      return Ok(gateway.clone());
    }
  }

  // Fall back to the first host address (Docker standard)
  let gateway = candidates
    .into_iter()
    .next()
    .context("No gateway candidates derived")?;
  trace!(
    "Assuming Docker gateway {} for {}",
    gateway, interface_name
  );
  Ok(gateway)
}

/// Derive candidate gateway addresses for the interface subnet:
/// the first and last usable host addresses. Docker assigns the
/// gateway the first host address of the subnet regardless of prefix length.
fn derive_gateway_candidates(
  ip_cidr: &str,
) -> anyhow::Result<Vec<String>> {
  let (ip, prefix) = ip_cidr
    .split_once('/')
    .context("Address is not in CIDR (ip/prefix) format")?;
  let ip: Ipv4Addr =
    ip.parse().context("Failed to parse IPv4 address")?;
  let prefix: u32 =
    prefix.parse().context("Failed to parse CIDR prefix")?;
  if !(1..=30).contains(&prefix) {
    return Err(anyhow!(
      "Subnet /{prefix} cannot contain a distinct gateway address"
    ));
  }

  let mask = u32::MAX << (32 - prefix);
  let network = u32::from(ip) & mask;
  let first_host = network + 1;
  let last_host = (network | !mask) - 1;

  Ok(vec![
    Ipv4Addr::from(first_host).to_string(),
    Ipv4Addr::from(last_host).to_string(),
  ])
}

/// Set default gateway to use specified interface
async fn set_default_gateway(
  gateway: &str,
  interface_name: &str,
) -> anyhow::Result<()> {
  trace!(
    "Setting default gateway to {} via {}",
    gateway, interface_name
  );

  // `replace` swaps the default route atomically, so a failure
  // never leaves the container without any default route.
  let replace = Command::new("ip")
    .args([
      "route",
      "replace",
      "default",
      "via",
      gateway,
      "dev",
      interface_name,
    ])
    .output()
    .await
    .context("Failed to run ip route replace")?;

  if !replace.status.success() {
    let error =
      String::from_utf8_lossy(&replace.stderr).trim().to_string();
    if error.contains("Operation not permitted") {
      warn!(
        "⚠️  Container lacks network privileges (NET_ADMIN capability required)"
      );
      warn!(
        "Add 'cap_add: [\"NET_ADMIN\"]' to your docker-compose.yaml"
      );
    }
    return Err(anyhow!(
      "❌ Failed to set default gateway via '{}': {}. \
            Verify interface configuration and network permissions",
      interface_name,
      error
    ));
  }

  trace!("Default gateway set to {} via {}", gateway, interface_name);
  Ok(())
}
