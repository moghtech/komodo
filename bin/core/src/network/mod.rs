use anyhow::{Context, anyhow};
use tokio::process::Command;
use tracing::{info, warn, debug};

/// Multi-NIC Internet Gateway Configuration
/// 
/// This module provides functionality to manually configure the internet gateway
/// when running in multi-network Docker environments. Useful when Docker's default
/// gateway selection doesn't choose the correct internet-connected interface.

/// Check if we're running in a container environment
fn is_container_environment() -> bool {
    std::path::Path::new("/.dockerenv").exists() ||
    std::env::var("container").is_ok() ||
    std::fs::read_to_string("/proc/1/cgroup")
        .map(|content| content.contains("docker") || content.contains("containerd"))
        .unwrap_or(false)
}

/// Main function to configure internet gateway
/// This should be called early in the application startup
pub async fn configure_internet_gateway(internet_interface: Option<String>) -> anyhow::Result<()> {
    if !is_container_environment() {
        debug!("[KOMODO-NETWORK] Not running in container, skipping gateway configuration");
        return Ok(());
    }

    // Check if manual interface configuration is provided
    if let Some(interface_name) = internet_interface {
        info!("[KOMODO-NETWORK] Configuring manual internet interface: {}", interface_name);
        return configure_manual_internet_interface(&interface_name).await;
    }

    debug!("[KOMODO-NETWORK] No manual interface configuration provided, using default system routing");
    Ok(())
}

/// Configure a manually specified internet interface
async fn configure_manual_internet_interface(interface_name: &str) -> anyhow::Result<()> {
    info!("[KOMODO-NETWORK] Setting up internet gateway for interface: {}", interface_name);
    
    // First, verify the interface exists and is up
    let interface_check = Command::new("ip")
        .args(&["addr", "show", interface_name])
        .output()
        .await
        .context("Failed to check interface status")?;
    
    if !interface_check.status.success() {
        return Err(anyhow!("Interface {} does not exist or is not accessible", interface_name));
    }
    
    let interface_info = String::from_utf8_lossy(&interface_check.stdout);
    if !interface_info.contains("state UP") {
        return Err(anyhow!("Interface {} is not in UP state", interface_name));
    }
    
    debug!("[KOMODO-NETWORK] Interface {} is available and UP", interface_name);
    
    // Find the gateway for this interface
    let gateway = find_gateway_for_interface(interface_name).await?;
    info!("[KOMODO-NETWORK] Found gateway {} for interface {}", gateway, interface_name);
    
    // Configure this interface as the primary internet gateway
    set_default_gateway(&gateway, interface_name).await?;
    
    info!("[KOMODO-NETWORK] ✅ Successfully configured {} as primary internet interface", interface_name);
    Ok(())
}

/// Find gateway for a specific interface
async fn find_gateway_for_interface(interface_name: &str) -> anyhow::Result<String> {
    // Get the IP address and network for this interface
    let addr_output = Command::new("ip")
        .args(&["addr", "show", interface_name])
        .output()
        .await
        .context("Failed to get interface address")?;
    
    let addr_info = String::from_utf8_lossy(&addr_output.stdout);
    
    // Extract IP/CIDR from interface info
    for line in addr_info.lines() {
        if line.trim().starts_with("inet ") && !line.contains("127.0.0.1") {
            let parts: Vec<&str> = line.trim().split_whitespace().collect();
            if let Some(ip_cidr) = parts.get(1) {
                debug!("[KOMODO-NETWORK] Interface {} has IP {}", interface_name, ip_cidr);
                return find_gateway_for_interface_network(interface_name, ip_cidr).await;
            }
        }
    }
    
    Err(anyhow!("Could not find IP address for interface {}", interface_name))
}

/// Find the gateway for an interface by analyzing its network
async fn find_gateway_for_interface_network(interface_name: &str, ip_cidr: &str) -> anyhow::Result<String> {
    debug!("[KOMODO-NETWORK] Finding gateway for interface {} in network {}", interface_name, ip_cidr);
    
    // First, try to find gateway from routing table for this specific network
    let route_output = Command::new("ip")
        .args(&["route", "show", "dev", interface_name])
        .output()
        .await
        .context("Failed to get routes for interface")?;

    if route_output.status.success() {
        let routes = String::from_utf8(route_output.stdout)?;
        debug!("[KOMODO-NETWORK] Routes for {}: {}", interface_name, routes);

        // Look for any route with a gateway on this interface
        for line in routes.lines() {
            if line.contains("via") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(via_idx) = parts.iter().position(|&x| x == "via") {
                    if let Some(&gateway) = parts.get(via_idx + 1) {
                        debug!("[KOMODO-NETWORK] Found gateway {} for interface {} from routing table", gateway, interface_name);
                        return Ok(gateway.to_string());
                    }
                }
            }
        }
    }

    // If no explicit gateway found, derive it from network configuration
    // In Docker networks, the gateway is typically the .1 address of the subnet
    if let Some(network_base) = ip_cidr.split('/').next() {
        let ip_parts: Vec<&str> = network_base.split('.').collect();
        if ip_parts.len() == 4 {
            // For Docker networks, try .1 first (most common), then .254
            let potential_gateways = vec![
                format!("{}.{}.{}.1", ip_parts[0], ip_parts[1], ip_parts[2]),
                format!("{}.{}.{}.254", ip_parts[0], ip_parts[1], ip_parts[2]),
            ];

            for gateway in potential_gateways {
                debug!("[KOMODO-NETWORK] Testing potential gateway {} for interface {}", gateway, interface_name);
                
                // Check if we can add a route via this gateway
                let route_test = Command::new("ip")
                    .args(&["route", "get", &gateway, "dev", interface_name])
                    .output()
                    .await;

                if let Ok(output) = route_test {
                    if output.status.success() {
                        debug!("[KOMODO-NETWORK] Gateway {} is reachable via interface {}", gateway, interface_name);
                        return Ok(gateway.to_string());
                    }
                }
                
                // Fallback: If route test fails, just assume .1 is the gateway for Docker networks
                if gateway.ends_with(".1") {
                    debug!("[KOMODO-NETWORK] Assuming Docker gateway {} for interface {} (standard .1 convention)", gateway, interface_name);
                    return Ok(gateway.to_string());
                }
            }
        }
    }

    Err(anyhow!("Could not determine gateway for interface {}", interface_name))
}

/// Set the default gateway to use a specific interface and gateway
async fn set_default_gateway(gateway: &str, interface_name: &str) -> anyhow::Result<()> {
    info!("[KOMODO-NETWORK] Setting default gateway to {} via {}", gateway, interface_name);
    
    // Check if we have network privileges
    if !check_network_privileges().await {
        warn!("[KOMODO-NETWORK] Container lacks network privileges (NET_ADMIN capability required)");
        warn!("[KOMODO-NETWORK] Add 'cap_add: [NET_ADMIN]' to docker-compose.yaml");
        return Err(anyhow!("Insufficient network privileges to modify routing table"));
    }
    
    // Remove existing default routes
    debug!("[KOMODO-NETWORK] Removing existing default routes");
    let remove_default = Command::new("sh")
        .args(&["-c", "ip route del default 2>/dev/null || true"])
        .output()
        .await;
    
    if let Ok(output) = remove_default {
        if output.status.success() {
            debug!("[KOMODO-NETWORK] Removed existing default routes");
        }
    }
    
    // Add new default route via specified gateway and interface
    let add_default_cmd = format!("ip route add default via {} dev {}", gateway, interface_name);
    debug!("[KOMODO-NETWORK] Adding default route: {}", add_default_cmd);
    
    let add_default = Command::new("sh")
        .args(&["-c", &add_default_cmd])
        .output()
        .await
        .context("Failed to add default route")?;
    
    if !add_default.status.success() {
        let error = String::from_utf8_lossy(&add_default.stderr);
        return Err(anyhow!("Failed to set default gateway: {}", error));
    }
    
    info!("[KOMODO-NETWORK] ✅ Default gateway successfully set to {} via {}", gateway, interface_name);
    
    // Verify the new routing
    let verify_route = Command::new("ip")
        .args(&["route", "show", "default"])
        .output()
        .await?;
    
    let route_info = String::from_utf8_lossy(&verify_route.stdout);
    debug!("[KOMODO-NETWORK] Current default routes:\n{}", route_info);
    
    Ok(())
}

/// Check if we have sufficient network privileges
async fn check_network_privileges() -> bool {
    let output = Command::new("ip")
        .args(&["route", "show"])
        .output()
        .await;
    
    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}
