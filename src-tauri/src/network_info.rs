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
    pub dhcp: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet_mask: Option<String>,
}

#[cfg(target_os = "windows")]
pub fn get_all_network_interfaces() -> Result<Vec<NetworkInterface>, String> {
    use ipconfig::{IfType, OperStatus};

    let adapters =
        ipconfig::get_adapters().map_err(|e| format!("Failed to get adapters: {}", e))?;
    
    // 使用Windows API获取DHCP信息
    let dhcp_info = get_dhcp_info_via_api();
    
    let mut interfaces = Vec::new();

    for adapter in adapters {
        if adapter.oper_status() != OperStatus::IfOperStatusUp
            && adapter.oper_status() != OperStatus::IfOperStatusDown
        {
            continue;
        }
        if adapter.if_type() == IfType::SoftwareLoopback {
            continue;
        }

        let mut dns_set = HashSet::new();
        for dns in adapter.dns_servers() {
            if dns.is_ipv4() {
                dns_set.insert(dns.to_string());
            }
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

        // 从前缀计算子网掩码
        let subnet_mask = adapter.prefixes().iter()
            .find(|(ip, _)| ip.is_ipv4())
            .map(|(_, prefix_len)| prefix_to_subnet_mask(*prefix_len));

        // 从API获取DHCP状态，使用adapter_name (GUID)作为key
        let adapter_name = adapter.adapter_name();
        // 尝试多种格式匹配
        let dhcp_enabled = dhcp_info.get(adapter_name)
            .or_else(|| {
                // 尝试带花括号的格式
                let with_braces = format!("{{{}}}", adapter_name);
                dhcp_info.get(&with_braces)
            })
            .or_else(|| {
                // 尝试去掉花括号的格式
                let without_braces = adapter_name
                    .trim_start_matches('{')
                    .trim_end_matches('}');
                dhcp_info.get(without_braces)
            })
            .copied()
            .unwrap_or(false);

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
            gateways: adapter
                .gateways()
                .iter()
                .filter(|ip| ip.is_ipv4())
                .map(|item| item.to_string())
                .collect(),
            guid: Some(adapter_name.to_string()),
            mask: adapter
                .prefixes()
                .iter()
                .map(|item| format!("{}/{}", item.0.to_string(), item.1))
                .collect(),
            receive_link_speed: adapter.receive_link_speed(),
            transmit_link_speed: adapter.transmit_link_speed(),
            dhcp: dhcp_enabled,
            subnet_mask,
        };

        interfaces.push(iface);
    }
    
    // 按GUID排序
    interfaces.sort_by(|a, b| {
        let guid_a = a.guid.as_deref().unwrap_or("");
        let guid_b = b.guid.as_deref().unwrap_or("");
        guid_a.cmp(guid_b)
    });
    
    Ok(interfaces)
}

/// 使用Windows API GetAdaptersInfo获取DHCP状态
/// 返回 HashMap<AdapterName(GUID), DhcpEnabled>
#[cfg(target_os = "windows")]
fn get_dhcp_info_via_api() -> std::collections::HashMap<String, bool> {
    use std::collections::HashMap;
    use std::ptr;
    
    let mut result = HashMap::new();
    
    // IP_ADAPTER_INFO结构体大小
    const MAX_ADAPTER_NAME_LENGTH: usize = 256;
    const MAX_ADAPTER_DESCRIPTION_LENGTH: usize = 128;
    const MAX_ADAPTER_ADDRESS_LENGTH: usize = 8;
    
    #[repr(C)]
    struct IP_ADDR_STRING {
        next: *mut IP_ADDR_STRING,
        ip_address: [u8; 16],
        ip_mask: [u8; 16],
        context: u32,
    }
    
    #[repr(C)]
    struct IP_ADAPTER_INFO {
        next: *mut IP_ADAPTER_INFO,
        combo_index: u32,
        adapter_name: [u8; MAX_ADAPTER_NAME_LENGTH + 4],
        description: [u8; MAX_ADAPTER_DESCRIPTION_LENGTH + 4],
        address_length: u32,
        address: [u8; MAX_ADAPTER_ADDRESS_LENGTH],
        index: u32,
        type_: u32,
        dhcp_enabled: u32,
        current_ip_address: *mut IP_ADDR_STRING,
        ip_address_list: IP_ADDR_STRING,
        gateway_list: IP_ADDR_STRING,
        dhcp_server: IP_ADDR_STRING,
        have_wins: u32,
        primary_wins_server: IP_ADDR_STRING,
        secondary_wins_server: IP_ADDR_STRING,
        lease_obtained: i64,
        lease_expires: i64,
    }
    
    #[link(name = "iphlpapi")]
    extern "system" {
        fn GetAdaptersInfo(adapter_info: *mut IP_ADAPTER_INFO, size: *mut u32) -> u32;
    }
    
    unsafe {
        // 首先获取需要的缓冲区大小
        let mut size: u32 = 0;
        let ret = GetAdaptersInfo(ptr::null_mut(), &mut size);
        
        // ERROR_BUFFER_OVERFLOW = 111
        if ret != 111 && ret != 0 {
            return result;
        }
        
        if size == 0 {
            return result;
        }
        
        // 分配缓冲区
        let layout = std::alloc::Layout::from_size_align(size as usize, 8).unwrap();
        let buffer = std::alloc::alloc(layout);
        if buffer.is_null() {
            return result;
        }
        
        let ret = GetAdaptersInfo(buffer as *mut IP_ADAPTER_INFO, &mut size);
        
        if ret == 0 {
            let mut info = buffer as *mut IP_ADAPTER_INFO;
            while !info.is_null() {
                let adapter_info = &*info;
                
                // 获取adapter_name (GUID) - 格式可能是 {GUID} 或 GUID
                let name_bytes = &adapter_info.adapter_name;
                let name_end = name_bytes.iter().position(|&b| b == 0).unwrap_or(name_bytes.len());
                let adapter_name = String::from_utf8_lossy(&name_bytes[..name_end]).to_string();
                
                // dhcp_enabled: 1 = enabled, 0 = disabled
                let dhcp_enabled = adapter_info.dhcp_enabled == 1;
                
                // 存储原始名称
                result.insert(adapter_name.clone(), dhcp_enabled);
                
                // 同时存储去掉花括号的版本（ipconfig crate可能返回不带花括号的GUID）
                let normalized_name = adapter_name
                    .trim_start_matches('{')
                    .trim_end_matches('}')
                    .to_string();
                if normalized_name != adapter_name {
                    result.insert(normalized_name, dhcp_enabled);
                }
                
                info = adapter_info.next;
            }
        }
        
        std::alloc::dealloc(buffer, layout);
    }
    
    result
}

#[cfg(target_os = "windows")]
fn prefix_to_subnet_mask(prefix_len: u32) -> String {
    if prefix_len > 32 {
        return "255.255.255.0".to_string();
    }
    let mask: u32 = if prefix_len == 0 {
        0
    } else {
        !0u32 << (32 - prefix_len)
    };
    format!(
        "{}.{}.{}.{}",
        (mask >> 24) & 0xFF,
        (mask >> 16) & 0xFF,
        (mask >> 8) & 0xFF,
        mask & 0xFF
    )
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
                    dhcp: false,
                    subnet_mask: None,
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
                        // 从CIDR提取子网掩码
                        if let Some(prefix) = addr.split('/').nth(1) {
                            if let Ok(prefix_len) = prefix.parse::<u32>() {
                                iface.subnet_mask = Some(prefix_to_subnet_mask_linux(prefix_len));
                            }
                        }
                        iface.ipv4.push(addr.split('/').next().unwrap_or(&addr).to_string());
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

    // 检测DHCP状态
    for iface in &mut interfaces {
        iface.dhcp = check_dhcp_linux(&iface.name);
    }

    Ok(interfaces)
}

#[cfg(target_os = "linux")]
fn check_dhcp_linux(interface_name: &str) -> bool {
    // 检查NetworkManager
    if let Ok(output) = Command::new("nmcli")
        .args(&["-t", "-f", "IP4.METHOD", "device", "show", interface_name])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("auto") {
            return true;
        }
    }
    false
}

#[cfg(target_os = "linux")]
fn prefix_to_subnet_mask_linux(prefix_len: u32) -> String {
    if prefix_len > 32 {
        return "255.255.255.0".to_string();
    }
    let mask: u32 = if prefix_len == 0 {
        0
    } else {
        !0u32 << (32 - prefix_len)
    };
    format!(
        "{}.{}.{}.{}",
        (mask >> 24) & 0xFF,
        (mask >> 16) & 0xFF,
        (mask >> 8) & 0xFF,
        mask & 0xFF
    )
}

#[cfg(target_os = "macos")]
pub fn get_all_network_interfaces() -> Result<Vec<NetworkInterface>, String> {
    let mut interfaces = Vec::new();

    let output = Command::new("ifconfig")
        .output()
        .map_err(|e| format!("Failed to run ifconfig: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut current_iface: Option<NetworkInterface> = None;

    for line in stdout.lines() {
        if !line.starts_with('\t') && !line.starts_with(' ') && !line.is_empty() {
            if let Some(iface) = current_iface.take() {
                if !iface.name.starts_with("lo") {
                    interfaces.push(iface);
                }
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                let name = parts[0].trim_end_matches(':').to_string();
                let enabled = line.contains("UP");

                current_iface = Some(NetworkInterface {
                    name,
                    enabled,
                    dhcp: false,
                    subnet_mask: None,
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
                if let Some(mask_idx) = parts.iter().position(|&p| p == "netmask") {
                    if parts.len() > mask_idx + 1 {
                        let hex_mask = parts[mask_idx + 1].trim_start_matches("0x");
                        if let Ok(mask_val) = u32::from_str_radix(hex_mask, 16) {
                            iface.subnet_mask = Some(format!(
                                "{}.{}.{}.{}",
                                (mask_val >> 24) & 0xFF,
                                (mask_val >> 16) & 0xFF,
                                (mask_val >> 8) & 0xFF,
                                mask_val & 0xFF
                            ));
                        }
                    }
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
        .args(&["--dns"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut dns_servers = Vec::new();
        for line in stdout.lines() {
            if line.contains("nameserver") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    dns_servers.push(parts[2].to_string());
                }
            }
        }
        for iface in &mut interfaces {
            iface.dns_servers = dns_servers.clone();
        }
    }

    // 检测DHCP状态
    for iface in &mut interfaces {
        iface.dhcp = check_dhcp_macos(&iface.name);
    }

    Ok(interfaces)
}

#[cfg(target_os = "macos")]
fn check_dhcp_macos(interface_name: &str) -> bool {
    if let Ok(output) = Command::new("networksetup")
        .args(&["-getinfo", interface_name])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout.contains("DHCP Configuration");
    }
    false
}
