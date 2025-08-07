use anyhow::{Context, anyhow};
use tokio::process::Command;
use tracing::{info, warn, debug};

/// Manual network interface configuration for multi-NIC Docker environments.

/// Check if running in container environment
fn is_container_environment() -> bool {
    std::path::Path::new("/.dockerenv").exists() ||
    std::env::var("container").is_ok() ||
    std::fs::read_to_string("/proc/1/cgroup")
        .map(|content| content.contains("docker") || content.contains("containerd"))
        .unwrap_or(false)
}

/// Configure internet gateway for specified interface
pub async fn configure_internet_gateway(internet_interface: String) -> anyhow::Result<()> {
    if !is_container_environment() {
        debug!("Not in container, skipping network configuration");
        return Ok(());
    }

    if !internet_interface.is_empty() {
        debug!("Configuring internet interface: {}", internet_interface);
        return configure_manual_interface(&internet_interface).await;
    }

    debug!("No interface specified, using default routing");
    Ok(())
}

/// Configure interface as default route
async fn configure_manual_interface(interface_name: &str) -> anyhow::Result<()> {    
    // Verify interface exists and is up
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
        return Err(anyhow!("Interface {} is not UP", interface_name));
    }
    
    debug!("Interface {} is UP", interface_name);
    
    let gateway = find_gateway(interface_name).await?;
    debug!("Found gateway {} for {}", gateway, interface_name);
    
    set_default_gateway(&gateway, interface_name).await?;
    
    info!("🌐 Configured {} as default gateway", interface_name);
    Ok(())
}

/// Find gateway for interface
async fn find_gateway(interface_name: &str) -> anyhow::Result<String> {
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
                debug!("Interface {} has IP {}", interface_name, ip_cidr);
                return find_gateway_for_network(interface_name, ip_cidr).await;
            }
        }
    }
    
    Err(anyhow!("Could not find IP address for interface {}", interface_name))
}

/// Find gateway for interface network
async fn find_gateway_for_network(interface_name: &str, ip_cidr: &str) -> anyhow::Result<String> {
    debug!("Finding gateway for interface {} in network {}", interface_name, ip_cidr);
    
    // Try to find gateway from routing table
    let route_output = Command::new("ip")
        .args(&["route", "show", "dev", interface_name])
        .output()
        .await
        .context("Failed to get routes for interface")?;

    if route_output.status.success() {
        let routes = String::from_utf8(route_output.stdout)?;
        debug!("Routes for {}: {}", interface_name, routes.trim());

        // Look for routes with gateway
        for line in routes.lines() {
            if line.contains("via") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(via_idx) = parts.iter().position(|&x| x == "via") {
                    if let Some(&gateway) = parts.get(via_idx + 1) {
                        debug!("Found gateway {} for {} from routing table", gateway, interface_name);
                        return Ok(gateway.to_string());
                    }
                }
            }
        }
    }

    // Derive gateway from network configuration (Docker standard: .1)
    if let Some(network_base) = ip_cidr.split('/').next() {
        let ip_parts: Vec<&str> = network_base.split('.').collect();
        if ip_parts.len() == 4 {
            let potential_gateways = vec![
                format!("{}.{}.{}.1", ip_parts[0], ip_parts[1], ip_parts[2]),
                format!("{}.{}.{}.254", ip_parts[0], ip_parts[1], ip_parts[2]),
            ];

            for gateway in potential_gateways {
                debug!("Testing potential gateway {} for {}", gateway, interface_name);
                
                // Check if gateway is reachable
                let route_test = Command::new("ip")
                    .args(&["route", "get", &gateway, "dev", interface_name])
                    .output()
                    .await;

                if let Ok(output) = route_test {
                    if output.status.success() {
                        debug!("Gateway {} is reachable via {}", gateway, interface_name);
                        return Ok(gateway.to_string());
                    }
                }
                
                // Fallback: assume .1 is gateway (Docker standard)
                if gateway.ends_with(".1") {
                    debug!("Assuming Docker gateway {} for {}", gateway, interface_name);
                    return Ok(gateway.to_string());
                }
            }
        }
    }

    Err(anyhow!("Could not determine gateway for interface {}", interface_name))
}

/// Set default gateway to use specified interface
async fn set_default_gateway(gateway: &str, interface_name: &str) -> anyhow::Result<()> {
    debug!("Setting default gateway to {} via {}", gateway, interface_name);
    
    // Check if we have network privileges
    if !check_network_privileges().await {
        warn!("Container lacks network privileges (NET_ADMIN capability required)");
        warn!("add 'cap_add: [NET_ADMIN]' to docker-compose.yaml");
        return Err(anyhow!("insufficient network privileges to modify routing table"));
    }
    
    // Remove existing default routes
    let remove_default = Command::new("sh")
        .args(&["-c", "ip route del default 2>/dev/null || true"])
        .output()
        .await;
    
    if let Ok(output) = remove_default {
        if output.status.success() {
            debug!("Removed existing default routes");
        }
    }
    
    // Add new default route
    let add_default_cmd = format!("ip route add default via {} dev {}", gateway, interface_name);
    debug!("Adding default route: {}", add_default_cmd);

    let add_default = Command::new("sh")
        .args(&["-c", &add_default_cmd])
        .output()
        .await
        .context("Failed to add default route")?;
    
    if !add_default.status.success() {
        let error = String::from_utf8_lossy(&add_default.stderr);
        return Err(anyhow!("Failed to set default gateway: {}", error));
    }
    
    debug!("Default gateway set to {} via {}", gateway, interface_name);
    
    // Verify new routing
    let verify_route = Command::new("ip")
        .args(&["route", "show", "default"])
        .output()
        .await?;
    
    let route_info = String::from_utf8_lossy(&verify_route.stdout);
    debug!("Current default routes: {}", route_info.trim());
    
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
