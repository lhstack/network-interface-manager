use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[cfg(not(target_os = "windows"))]
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkInterface {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_address: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ipv4: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ipv6: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub dns_servers: Vec<String>,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_type: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub gateways: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guid: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub mask: Vec<String>,
    pub receive_link_speed: u64,
    pub transmit_link_speed: u64,
}

#[cfg(target_os = "windows")]
pub fn get_all_network_interfaces() -> Result<Vec<NetworkInterface>, String> {
    use ipconfig::{IfType, OperStatus};
    
    let adapters = ipconfig::get_adapters().map_err(|e| format!("Failed to get adapters: {}", e))?;
    let mut interfaces = Vec::new();

    for adapter in adapters {
        if adapter.oper_status() != OperStatus::IfOperStatusUp && adapter.oper_status() != OperStatus::IfOperStatusDown {
            continue;
        }
        if adapter.if_type() == IfType::SoftwareLoopback {
            continue;
        }

        let mut dns_set = HashSet::new();
        for dns in adapter.dns_servers() {
            dns_set.insert(dns.to_string());
        }

        let mut ipv4_list = Vec::new();
        for ip in adapter.ip_addresses() {
            if ip.is_ipv4() {
                ipv4_list.push(ip.to_string());
            }
        }
        ipv4_list.sort();

        let mut ipv6_list = Vec::new();
        for ip in adapter.ip_addresses() {
            if ip.is_ipv6() && !ip.is_loopback() {
                ipv6_list.push(ip.to_string());
            }
        }
        ipv6_list.sort();

        let mut dns_list: Vec<String> = dns_set.into_iter().collect();
        dns_list.sort();

        let iface = NetworkInterface {
            name: adapter.friendly_name().to_string(),
            description: Some(adapter.description().to_string()),
            mac_address: adapter
                .physical_address()
                .filter(|addr| addr.len() == 6 && addr.iter().any(|&b| b != 0))
                .map(|addr| {
                    format!(
                        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                        addr[0], addr[1], addr[2], addr[3], addr[4], addr[5]
                    )
                }),
            ipv4: ipv4_list,
            ipv6: ipv6_list,
            dns_servers: dns_list,
            enabled: adapter.oper_status() == OperStatus::IfOperStatusUp,
            if_type: Some(format!("{:?}", adapter.if_type())),
            gateways: adapter.gateways().iter().map(|item| item.to_string()).collect(),
            guid: Some(adapter.adapter_name().to_string()),
            mask: adapter.prefixes().iter().map(|item| {
                format!("{}/{}", item.0.to_string(), item.1)
            }).collect(),
            receive_link_speed: adapter.receive_link_speed(),
            transmit_link_speed: adapter.transmit_link_speed(),
        };

        interfaces.push(iface);
    }
    Ok(interfaces)
}

#[cfg(target_os = "linux")]
pub fn get_all_network_interfaces() -> Result<Vec<NetworkInterface>, String> {
    let mut interfaces = Vec::new();

    // 使用 ip link show 命令获取网卡信息
    let output = Command::new("ip")
        .args(&["link", "show"])
        .output()
        .map_err(|e| format!("Failed to run ip command: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut current_iface: Option<NetworkInterface> = None;

    for line in stdout.lines() {
        if line.starts_with(|c: char| c.is_numeric()) {
            // 新的网卡行
            if let Some(iface) = current_iface.take() {
                if !iface.name.starts_with("lo") {
                    interfaces.push(iface);
                }
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[1].trim_end_matches(':').to_string();
                let enabled = line.contains("UP");

                current_iface = Some(NetworkInterface {
                    name,
                    enabled,
                    ..Default::default()
                });
            }
        } else if let Some(ref mut iface) = current_iface {
            if line.contains("link/ether") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    iface.mac_address = Some(parts[1].to_string());
                }
            }
        }
    }

    if let Some(iface) = current_iface {
        if !iface.name.starts_with("lo") {
            interfaces.push(iface);
        }
    }

    // 获取 IP 地址信息
    let output = Command::new("ip")
        .args(&["addr", "show"])
        .output()
        .map_err(|e| format!("Failed to run ip addr command: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut current_name = String::new();

    for line in stdout.lines() {
        if line.starts_with(|c: char| c.is_numeric()) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                current_name = parts[1].trim_end_matches(':').to_string();
            }
        } else if line.contains("inet ") || line.contains("inet6 ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let addr = parts[1].to_string();
                if let Some(iface) = interfaces.iter_mut().find(|i| i.name == current_name) {
                    if line.contains("inet6") {
                        iface.ipv6.push(addr);
                    } else {
                        iface.ipv4.push(addr);
                    }
                }
            }
        }
    }

    // 获取 DNS 信息
    if let Ok(content) = std::fs::read_to_string("/etc/resolv.conf") {
        let mut dns_servers = Vec::new();
        for line in content.lines() {
            if line.starts_with("nameserver ") {
                if let Some(dns) = line.strip_prefix("nameserver ") {
                    dns_servers.push(dns.trim().to_string());
                }
            }
        }
        for iface in &mut interfaces {
            iface.dns_servers = dns_servers.clone();
        }
    }

    Ok(interfaces)
}

#[cfg(target_os = "macos")]
pub fn get_all_network_interfaces() -> Result<Vec<NetworkInterface>, String> {
    let mut interfaces = Vec::new();

    // 使用 ifconfig 命令获取网卡信息
    let output = Command::new("ifconfig")
        .output()
        .map_err(|e| format!("Failed to run ifconfig: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut current_iface: Option<NetworkInterface> = None;

    for line in stdout.lines() {
        if !line.starts_with('\t') && !line.starts_with(' ') && !line.is_empty() {
            // 新的网卡行
            if let Some(iface) = current_iface.take() {
                if !iface.name.starts_with("lo") {
                    interfaces.push(iface);
                }
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                let name = parts[0].to_string();
                let enabled = line.contains("UP");

                current_iface = Some(NetworkInterface {
                    name,
                    enabled,
                    ..Default::default()
                });
            }
        } else if let Some(ref mut iface) = current_iface {
            let trimmed = line.trim();
            if trimmed.starts_with("inet ") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    iface.ipv4.push(parts[1].to_string());
                }
            } else if trimmed.starts_with("inet6 ") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    iface.ipv6.push(parts[1].to_string());
                }
            } else if trimmed.starts_with("ether ") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    iface.mac_address = Some(parts[1].to_string());
                }
            }
        }
    }

    if let Some(iface) = current_iface {
        if !iface.name.starts_with("lo") {
            interfaces.push(iface);
        }
    }

    // 获取 DNS 信息
    let output = Command::new("scutil")
        .args(&["-r", "State:/Network/Global/DNS"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("ServerAddresses") {
                // 简单解析 DNS 服务器
                if let Some(start) = line.find('{') {
                    if let Some(end) = line.find('}') {
                        let dns_str = &line[start + 1..end];
                        let dns_list: Vec<String> = dns_str
                            .split(',')
                            .map(|s| s.trim().trim_matches('"').to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        for iface in &mut interfaces {
                            iface.dns_servers = dns_list.clone();
                        }
                    }
                }
            }
        }
    }

    Ok(interfaces)
}
