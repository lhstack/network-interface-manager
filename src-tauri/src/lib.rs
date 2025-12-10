use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use ipconfig::{IfType, OperStatus};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
#[cfg(target_os = "linux")]
fn greet(dns: &str) -> Result<String, String> {
    Ok(dns.to_string())
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkInterface {
    // 网卡名称
    pub name: String,
    // 网卡描述（例如："Realtek PCIe GbE Family Controller"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    // 物理地址（MAC 地址）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_address: Option<String>,
    // 网卡 IPv4 地址列表（一个网卡可能有多个）
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ipv4: Vec<String>,
    // 网卡 IPv6 地址列表
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ipv6: Vec<String>,
    // DNS 服务器地址列表
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub dns_servers: Vec<String>,
    // 是否启用
    pub enabled: bool,
    // 接口类型（例如：以太网、Wifi）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_type: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub gateways:Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guid:Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub mask:Vec<String>,
    pub receive_link_speed: u64,
    pub transmit_link_speed:u64,
}
#[tauri::command]
#[cfg(target_os = "windows")]
fn get_all_network_interface() -> Result<Vec<NetworkInterface>, String> {
    // 获取计算机所有网络适配器
    let adapters = ipconfig::get_adapters().unwrap();
    let mut interfaces = Vec::new();

    for adapter in adapters {
        if adapter.oper_status() != OperStatus::IfOperStatusUp && adapter.oper_status() != OperStatus::IfOperStatusDown {
            continue;
        }
        if adapter.if_type() == IfType::SoftwareLoopback {
            continue;
        }
        // 初始化一个空的 DNS 服务器集合（使用集合去重）
        let mut dns_set = HashSet::new();

        // 收集所有 DNS 服务器地址（包括IPv4和IPv6）
        for dns in adapter.dns_servers() {
            dns_set.insert(dns.to_string());
        }
        // 收集 IPv4 地址信息
        let mut ipv4_list = Vec::new();
        for ip in adapter.ip_addresses() {
            if ip.is_ipv4() {
                ipv4_list.push(ip.to_string());
            }
        }
        ipv4_list.sort();
        // 收集 IPv6 地址信息
        let mut ipv6_list = Vec::new();
        for ip in adapter.ip_addresses() {
            if ip.is_ipv6() && !ip.is_loopback() {
                // 通常过滤掉环回地址
                ipv6_list.push(ip.to_string());
            }
        }
        ipv6_list.sort();
        let mut dns_list: Vec<String> = dns_set.into_iter().collect();
        dns_list.sort();
        // 构建 NetworkInterface 实例
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
            gateways: adapter.gateways().iter().map(|item|{
                item.to_string()
            }).collect(),
            guid: Some(adapter.adapter_name().to_string()),
            mask: adapter.prefixes().iter().map(|item|{
                format!("{}/{}",item.0.to_string(),item.1)
            }).collect(),
            receive_link_speed: adapter.receive_link_speed(),
            transmit_link_speed: adapter.transmit_link_speed(),
        };

        interfaces.push(iface);
    }
    Ok(interfaces)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_all_network_interface])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
