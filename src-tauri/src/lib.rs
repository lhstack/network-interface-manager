use serde::{Deserialize, Serialize};
use std::process::Command;
use tauri::tray::{MouseButton, MouseButtonState};
use tauri::Manager;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

mod db;
mod dns_task;
mod network_info;

use dns_task::{DnsTask, TASK_MANAGER};
use network_info::{get_all_network_interfaces, NetworkInterface};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    pub interface_name: String,
    pub dns_servers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub interface_name: String,
    pub dhcp: bool,
    #[serde(default)]
    pub ip_address: String,
    #[serde(default)]
    pub subnet_mask: String,
    #[serde(default)]
    pub gateway: String,
    #[serde(default)]
    pub dns: Vec<String>,
}

#[tauri::command]
fn get_all_network_interface() -> Result<Vec<NetworkInterface>, String> {
    get_all_network_interfaces()
}

#[tauri::command]
fn add_dns_task(task: DnsTask) -> Result<(), String> {
    TASK_MANAGER.add_task(task)
}

#[tauri::command]
#[allow(non_snake_case)]
fn remove_dns_task(taskId: String) -> Result<(), String> {
    TASK_MANAGER.remove_task(&taskId)
}

#[tauri::command]
fn get_dns_tasks() -> Result<Vec<DnsTask>, String> {
    TASK_MANAGER.get_tasks()
}

#[tauri::command]
fn update_dns_task(task: DnsTask) -> Result<(), String> {
    TASK_MANAGER.update_task(task)
}

#[tauri::command]
fn get_task_statuses() -> Result<Vec<dns_task::TaskStatus>, String> {
    TASK_MANAGER.get_task_statuses()
}

#[tauri::command]
fn start_dns_monitoring() -> Result<(), String> {
    TASK_MANAGER.start_monitoring()
}

#[tauri::command]
fn stop_dns_monitoring() -> Result<(), String> {
    TASK_MANAGER.stop_monitoring()
}

#[tauri::command]
fn is_dns_monitoring_running() -> Result<bool, String> {
    TASK_MANAGER.is_running()
}

#[tauri::command]
fn restore_monitoring_state() -> Result<(), String> {
    TASK_MANAGER.restore_monitoring_state()
}

#[tauri::command]
fn init_app() -> Result<(), String> {
    TASK_MANAGER.init_database()?;
    TASK_MANAGER.restore_monitoring_state()
}

#[tauri::command]
fn get_logs() -> Result<Vec<dns_task::LogEntry>, String> {
    TASK_MANAGER.get_logs()
}

#[tauri::command]
fn clear_logs() -> Result<(), String> {
    TASK_MANAGER.clear_logs()
}

#[tauri::command]
fn set_network_config(config: NetworkConfig) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    return set_network_config_windows(&config);

    #[cfg(target_os = "linux")]
    return set_network_config_linux(&config);

    #[cfg(target_os = "macos")]
    return set_network_config_macos(&config);
}

#[cfg(target_os = "windows")]
fn set_network_config_windows(config: &NetworkConfig) -> Result<String, String> {
    if config.dhcp {
        enable_dhcp_windows(&config.interface_name, &config.dns)
    } else {
        set_static_ip_windows(config)
    }
}

#[cfg(target_os = "windows")]
fn enable_dhcp_windows(interface_name: &str, dns: &[String]) -> Result<String, String> {
    // 启用DHCP获取IP
    let cmd = format!(
        "netsh interface ip set address name=\"{}\" source=dhcp",
        interface_name
    );
    
    let output = Command::new("cmd")
        .args(&["/C", &cmd])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // 忽略"已经是DHCP"的错误
        if !stderr.is_empty() && !stderr.contains("DHCP") {
            return Err(stderr.to_string());
        }
    }

    // 如果指定了DNS，设置静态DNS；否则使用DHCP获取DNS
    if !dns.is_empty() {
        set_dns_windows_internal(interface_name, dns)?;
    } else {
        let dns_cmd = format!(
            "netsh interface ip set dns name=\"{}\" source=dhcp",
            interface_name
        );
        Command::new("cmd")
            .args(&["/C", &dns_cmd])
            .creation_flags(0x08000000)
            .output()
            .ok();
    }

    // 刷新DNS缓存
    flush_dns_cache_windows();

    Ok(format!("DHCP enabled for {}", interface_name))
}

#[cfg(target_os = "windows")]
fn set_static_ip_windows(config: &NetworkConfig) -> Result<String, String> {
    if config.ip_address.is_empty() || config.subnet_mask.is_empty() {
        return Err("IP address and subnet mask are required for static configuration".to_string());
    }

    // 设置静态IP
    let cmd = if config.gateway.is_empty() {
        format!(
            "netsh interface ip set address name=\"{}\" static {} {}",
            config.interface_name, config.ip_address, config.subnet_mask
        )
    } else {
        format!(
            "netsh interface ip set address name=\"{}\" static {} {} {}",
            config.interface_name, config.ip_address, config.subnet_mask, config.gateway
        )
    };

    let output = Command::new("cmd")
        .args(&["/C", &cmd])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            return Err(stderr.to_string());
        }
    }

    // 设置DNS
    if !config.dns.is_empty() {
        set_dns_windows_internal(&config.interface_name, &config.dns)?;
    }

    // 刷新DNS缓存
    flush_dns_cache_windows();

    Ok(format!("Static IP configured for {}", config.interface_name))
}

#[cfg(target_os = "windows")]
fn set_dns_windows_internal(interface_name: &str, dns_servers: &[String]) -> Result<(), String> {
    if dns_servers.is_empty() {
        return Ok(());
    }

    // 设置主DNS
    let cmd = format!(
        "netsh interface ip set dns name=\"{}\" static {}",
        interface_name, dns_servers[0]
    );

    let output = Command::new("cmd")
        .args(&["/C", &cmd])
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            return Err(stderr.to_string());
        }
    }

    // 添加备用DNS
    for (i, dns) in dns_servers.iter().skip(1).enumerate() {
        let add_cmd = format!(
            "netsh interface ip add dns name=\"{}\" {} index={}",
            interface_name, dns, i + 2
        );
        Command::new("cmd")
            .args(&["/C", &add_cmd])
            .creation_flags(0x08000000)
            .output()
            .ok();
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn flush_dns_cache_windows() {
    Command::new("cmd")
        .args(&["/C", "ipconfig /flushdns"])
        .creation_flags(0x08000000)
        .output()
        .ok();
}

#[cfg(target_os = "linux")]
#[allow(dead_code)]
fn set_network_config_linux(config: &NetworkConfig) -> Result<String, String> {
    if config.dhcp {
        enable_dhcp_linux(&config.interface_name)
    } else {
        set_static_ip_linux(config)
    }
}

#[cfg(target_os = "linux")]
fn enable_dhcp_linux(interface_name: &str) -> Result<String, String> {
    // 尝试使用nmcli
    let output = Command::new("nmcli")
        .args(&["con", "mod", interface_name, "ipv4.method", "auto"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            // 重新激活连接
            Command::new("nmcli")
                .args(&["con", "up", interface_name])
                .output()
                .ok();
            return Ok(format!("DHCP enabled for {}", interface_name));
        }
    }

    // 回退到dhclient
    let output = Command::new("dhclient")
        .arg(interface_name)
        .output()
        .map_err(|e| format!("Failed to run dhclient: {}", e))?;

    if output.status.success() {
        Ok(format!("DHCP enabled for {}", interface_name))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[cfg(target_os = "linux")]
fn set_static_ip_linux(config: &NetworkConfig) -> Result<String, String> {
    if config.ip_address.is_empty() || config.subnet_mask.is_empty() {
        return Err("IP address and subnet mask are required".to_string());
    }

    // 计算CIDR前缀
    let prefix = subnet_mask_to_prefix(&config.subnet_mask);

    // 使用ip命令设置
    let cmd = format!(
        "ip addr flush dev {} && ip addr add {}/{} dev {}",
        config.interface_name, config.ip_address, prefix, config.interface_name
    );

    let output = Command::new("sh")
        .args(&["-c", &cmd])
        .output()
        .map_err(|e| format!("Failed to set IP: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    // 设置网关
    if !config.gateway.is_empty() {
        let gw_cmd = format!("ip route add default via {}", config.gateway);
        Command::new("sh")
            .args(&["-c", &gw_cmd])
            .output()
            .ok();
    }

    // 设置DNS
    if !config.dns.is_empty() {
        let dns_content = config.dns.iter()
            .map(|d| format!("nameserver {}", d))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write("/etc/resolv.conf", dns_content).ok();
    }

    Ok(format!("Static IP configured for {}", config.interface_name))
}

#[cfg(target_os = "linux")]
fn subnet_mask_to_prefix(mask: &str) -> u32 {
    let parts: Vec<u8> = mask.split('.')
        .filter_map(|p| p.parse().ok())
        .collect();
    
    if parts.len() != 4 {
        return 24; // 默认
    }

    let mask_val: u32 = ((parts[0] as u32) << 24)
        | ((parts[1] as u32) << 16)
        | ((parts[2] as u32) << 8)
        | (parts[3] as u32);
    
    mask_val.count_ones()
}

#[cfg(target_os = "macos")]
#[allow(dead_code)]
fn set_network_config_macos(config: &NetworkConfig) -> Result<String, String> {
    if config.dhcp {
        enable_dhcp_macos(&config.interface_name)
    } else {
        set_static_ip_macos(config)
    }
}

#[cfg(target_os = "macos")]
fn enable_dhcp_macos(interface_name: &str) -> Result<String, String> {
    let output = Command::new("networksetup")
        .args(&["-setdhcp", interface_name])
        .output()
        .map_err(|e| format!("Failed to enable DHCP: {}", e))?;

    if output.status.success() {
        Ok(format!("DHCP enabled for {}", interface_name))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[cfg(target_os = "macos")]
fn set_static_ip_macos(config: &NetworkConfig) -> Result<String, String> {
    if config.ip_address.is_empty() || config.subnet_mask.is_empty() {
        return Err("IP address and subnet mask are required".to_string());
    }

    let router = if config.gateway.is_empty() { "empty" } else { &config.gateway };

    let output = Command::new("networksetup")
        .args(&[
            "-setmanual",
            &config.interface_name,
            &config.ip_address,
            &config.subnet_mask,
            router,
        ])
        .output()
        .map_err(|e| format!("Failed to set static IP: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    // 设置DNS
    if !config.dns.is_empty() {
        let dns_args: Vec<&str> = std::iter::once("-setdnsservers")
            .chain(std::iter::once(config.interface_name.as_str()))
            .chain(config.dns.iter().map(|s| s.as_str()))
            .collect();
        
        Command::new("networksetup")
            .args(&dns_args)
            .output()
            .ok();
    }

    Ok(format!("Static IP configured for {}", config.interface_name))
}

#[tauri::command]
fn is_admin() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        Ok(is_elevated::is_elevated())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(unsafe { libc::geteuid() == 0 })
    }
}

#[tauri::command]
fn set_dns_servers(config: DnsConfig) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    return set_dns_windows(&config);

    #[cfg(target_os = "linux")]
    return set_dns_linux(&config);

    #[cfg(target_os = "macos")]
    return set_dns_macos(&config);
}

#[cfg(target_os = "windows")]
#[allow(dead_code)]
fn set_dns_windows(config: &DnsConfig) -> Result<String, String> {
    if config.dns_servers.is_empty() {
        return Err("DNS servers list is empty".to_string());
    }

    let mut cmd = format!(
        "netsh interface ip set dns \"{}\" static {}",
        config.interface_name, config.dns_servers[0]
    );

    for dns in &config.dns_servers[1..] {
        cmd.push_str(&format!(
            " & netsh interface ip add dns \"{}\" {}",
            config.interface_name, dns
        ));
    }

    let output = Command::new("cmd")
        .args(&["/C", &cmd])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        Ok(format!("DNS servers set for {}", config.interface_name))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[cfg(target_os = "linux")]
#[allow(dead_code)]
fn set_dns_linux(config: &DnsConfig) -> Result<String, String> {
    let dns_list = config.dns_servers.join(" ");
    let cmd = format!(
        "echo 'nameserver {}' | sudo tee /etc/resolv.conf > /dev/null",
        dns_list.replace(" ", "\nnameserver ")
    );

    let output = Command::new("sh")
        .args(&["-c", &cmd])
        .output()
        .map_err(|e| format!("Failed to set DNS: {}", e))?;

    if output.status.success() {
        Ok(format!("DNS servers set for {}", config.interface_name))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[cfg(target_os = "macos")]
#[allow(dead_code)]
fn set_dns_macos(config: &DnsConfig) -> Result<String, String> {
    let dns_list = config.dns_servers.join(" ");
    let output = Command::new("sudo")
        .args(&["networksetup", "-setdnsservers", &config.interface_name])
        .arg(&dns_list)
        .output()
        .map_err(|e| format!("Failed to set DNS: {}", e))?;

    if output.status.success() {
        Ok(format!("DNS servers set for {}", config.interface_name))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[cfg(target_os = "windows")]
fn request_admin_privileges() -> Result<(), String> {
    use std::ffi::CString;
    use winapi::um::shellapi::ShellExecuteA;
    use winapi::um::winuser::SW_SHOW;

    let exe_path =
        std::env::current_exe().map_err(|e| format!("Failed to get executable path: {}", e))?;

    let exe_path_str = exe_path.to_string_lossy().to_string();
    let exe_path_cstr =
        CString::new(exe_path_str).map_err(|e| format!("Failed to create CString: {}", e))?;

    let operation =
        CString::new("runas").map_err(|e| format!("Failed to create operation string: {}", e))?;

    unsafe {
        let result = ShellExecuteA(
            std::ptr::null_mut(),
            operation.as_ptr(),
            exe_path_cstr.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOW,
        );

        if result as i32 <= 32 {
            return Err("Failed to request administrator privileges".to_string());
        }
    }

    Ok(())
}

fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::TrayIconBuilder;

    // 创建托盘菜单
    let menu = Menu::new(app)?;

    // 添加"退出"菜单项
    let quit = MenuItem::new(app, "退出", true, Some("quit"))?;
    menu.append(&quit)?;
    // 创建托盘图标并关联菜单
    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .icon(app.default_window_icon().unwrap().clone())
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            if event.id.as_ref() == quit.id().as_ref() {
                app.exit(0);
                std::process::exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| {
            use tauri::tray::TrayIconEvent;
            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } => {
                    // 左键点击托盘图标切换显示/隐藏
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            // 如果窗口可见，隐藏它
                            let _ = window.set_skip_taskbar(true);
                            let _ = window.hide();
                        } else {
                            // 如果窗口隐藏，显示它
                            let _ = window.set_skip_taskbar(false);
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
                _ => {}
            }
        })
        .build(app)?;

    // 保持托盘图标活跃
    // std::mem::forget(tray);

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "windows")]
    {
        // 检查管理员权限
        if !is_elevated::is_elevated() {
            // 尝试以管理员身份重新启动应用
            request_admin_privileges().expect("Failed to request administrator privileges");
            std::process::exit(0);
        }
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_all_network_interface,
            set_dns_servers,
            set_network_config,
            add_dns_task,
            remove_dns_task,
            get_dns_tasks,
            update_dns_task,
            get_task_statuses,
            start_dns_monitoring,
            stop_dns_monitoring,
            is_dns_monitoring_running,
            restore_monitoring_state,
            init_app,
            is_admin,
            get_logs,
            clear_logs
        ])
        .setup(|app| {
            #[cfg(target_os = "linux")]
            {
                // 检查root权限
                if unsafe { libc::geteuid() != 0 } {
                    eprintln!("This application requires root privileges. Please run with sudo.");
                    std::process::exit(1);
                }
            }

            #[cfg(target_os = "macos")]
            {
                // macOS通常不需要root权限来修改DNS，但可以检查
                // 如果需要，可以在这里添加权限检查
            }

            // 设置托盘菜单
            setup_tray(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    // 阻止窗口关闭，改为最小化到托盘
                    // 隐藏时从任务栏移除
                    let _ = window.set_skip_taskbar(true);
                    window.hide().ok();
                    api.prevent_close();
                }
                tauri::WindowEvent::Focused(focused) => {
                    // 窗口获得焦点时，确保显示在任务栏
                    if *focused {
                        let _ = window.set_skip_taskbar(false);
                    }
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
