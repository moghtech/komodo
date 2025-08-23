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
//! - Safe default route modification with privilege checking
//! - Comprehensive error handling and logging

use anyhow::{Context, anyhow};
use tokio::process::Command;
use tracing::{debug, info, trace, warn};

/// Standard gateway addresses to test for Docker networks
const DOCKER_GATEWAY_CANDIDATES: &[&str] = &[".1", ".254"];

/// Container environment detection files
const DOCKERENV_FILE: &str = "/.dockerenv";
const CGROUP_FILE: &str = "/proc/1/cgroup";

/// Check if running in container environment
fn is_container_environment() -> bool {
  // Check for Docker-specific indicators
  if std::path::Path::new(DOCKERENV_FILE).exists() {
    return true;
  }

  // Check container environment variable
  if std::env::var("container").is_ok() {
    return true;
  }

  // Check cgroup for container runtime indicators
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

  if !is_container_environment() {
    debug!("Not in container, skipping network configuration");
    return;
  }

  if !config.internet_interface.is_empty() {
    debug!(
      "Configuring internet interface: {}",
      config.internet_interface
    );
    if let Err(e) =
      configure_manual_interface(&config.internet_interface).await
    {
      warn!("Failed to configure internet gateway: {e:#}");
    }
  } else {
    debug!("No interface specified, using default routing");
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
  if !interface_info.contains("state UP") {
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
    let routes = String::from_utf8(route_output.stdout)?;
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

  // Derive gateway from network configuration (Docker standard: .1)
  if let Some(network_base) = ip_cidr.split('/').next() {
    let ip_parts: Vec<&str> = network_base.split('.').collect();
    if ip_parts.len() == 4 {
      let potential_gateways: Vec<String> = DOCKER_GATEWAY_CANDIDATES
        .iter()
        .map(|suffix| {
          format!(
            "{}.{}.{}{}",
            ip_parts[0], ip_parts[1], ip_parts[2], suffix
          )
        })
        .collect();

      for gateway in potential_gateways {
        trace!(
          "Testing potential gateway {} for {}",
          gateway, interface_name
        );

        // Check if gateway is reachable
        let route_test = Command::new("ip")
          .args(["route", "get", &gateway, "dev", interface_name])
          .output()
          .await;

        if let Ok(output) = route_test
          && output.status.success()
        {
          trace!(
            "Gateway {} is reachable via {}",
            gateway, interface_name
          );
          return Ok(gateway.to_string());
        }

        // Fallback: assume .1 is gateway (Docker standard)
        if gateway.ends_with(".1") {
          trace!(
            "Assuming Docker gateway {} for {}",
            gateway, interface_name
          );
          return Ok(gateway.to_string());
        }
      }
    }
  }

  Err(anyhow!(
    "Could not determine gateway for interface '{}' in network '{}'. \
        Ensure the interface is properly configured with a valid gateway",
    interface_name,
    ip_cidr
  ))
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

  // Check if we have network privileges
  if !check_network_privileges().await {
    warn!(
      "⚠️  Container lacks network privileges (NET_ADMIN capability required)"
    );
    warn!(
      "Add 'cap_add: [\"NET_ADMIN\"]' to your docker-compose.yaml"
    );
    return Err(anyhow!(
      "Insufficient network privileges to modify routing table. \
            Container needs NET_ADMIN capability to configure network interfaces"
    ));
  }

  // Remove existing default routes
  let remove_default = Command::new("sh")
    .args(["-c", "ip route del default 2>/dev/null || true"])
    .output()
    .await;

  if let Ok(output) = remove_default
    && output.status.success()
  {
    trace!("Removed existing default routes");
  }

  // Add new default route
  let add_default_cmd = format!(
    "ip route add default via {gateway} dev {interface_name}"
  );
  trace!("Adding default route: {}", add_default_cmd);

  let add_default = Command::new("sh")
    .args(["-c", &add_default_cmd])
    .output()
    .await
    .context("Failed to add default route")?;

  if !add_default.status.success() {
    let error = String::from_utf8_lossy(&add_default.stderr)
      .trim()
      .to_string();
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

/// Check if we have sufficient network privileges
async fn check_network_privileges() -> bool {
  // Try to test NET_ADMIN capability with a harmless route operation
  let capability_test = Command::new("sh")
        .args(["-c", "ip route add 198.51.100.1/32 dev lo 2>/dev/null && ip route del 198.51.100.1/32 dev lo 2>/dev/null"])
        .output()
        .await;

  matches!(capability_test, Ok(output) if output.status.success())
}
