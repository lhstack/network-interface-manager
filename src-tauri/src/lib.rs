use serde::{Deserialize, Serialize};
use std::process::Command;
use tauri::tray::{MouseButton, MouseButtonState};
use tauri::Manager;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

mod network_info;
mod dns_task;
mod db;

use dns_task::{DnsTask, TASK_MANAGER};
use network_info::{get_all_network_interfaces, NetworkInterface};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    pub interface_name: String,
    pub dns_servers: Vec<String>,
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
fn set_autostart(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        set_autostart_windows(enabled)
    }
    
    #[cfg(target_os = "linux")]
    {
        set_autostart_linux(enabled)
    }
    
    #[cfg(target_os = "macos")]
    {
        set_autostart_macos(enabled)
    }
}

#[cfg(target_os = "windows")]
fn set_autostart_windows(enabled: bool) -> Result<(), String> {
    use std::fs;
    
    let app_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get app path: {}", e))?;
    
    let startup_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("Microsoft\\Windows\\Start Menu\\Programs\\Startup");
    
    let shortcut_path = startup_dir.join("NetworkInterfaceManager.lnk");
    
    if enabled {
        // 创建快捷方式
        let cmd = format!(
            "powershell -Command \"$WshShell = New-Object -ComObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut('{}'); $Shortcut.TargetPath = '{}'; $Shortcut.Save()\"",
            shortcut_path.display(),
            app_path.display()
        );
        
        let output = Command::new("cmd")
            .args(&["/C", &cmd])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output()
            .map_err(|e| format!("Failed to create shortcut: {}", e))?;
        
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
    } else {
        // 删除快捷方式
        if shortcut_path.exists() {
            fs::remove_file(&shortcut_path)
                .map_err(|e| format!("Failed to remove shortcut: {}", e))?;
        }
    }
    
    Ok(())
}

#[cfg(target_os = "linux")]
fn set_autostart_linux(enabled: bool) -> Result<(), String> {
    use std::path::PathBuf;
    use std::fs;
    
    let app_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get app path: {}", e))?;
    
    let autostart_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("autostart");
    
    let desktop_file = autostart_dir.join("network-interface-manager.desktop");
    
    if enabled {
        fs::create_dir_all(&autostart_dir)
            .map_err(|e| format!("Failed to create autostart directory: {}", e))?;
        
        let content = format!(
            "[Desktop Entry]\nType=Application\nName=Network Interface Manager\nExec={}\nX-GNOME-Autostart-enabled=true\n",
            app_path.display()
        );
        
        fs::write(&desktop_file, content)
            .map_err(|e| format!("Failed to write desktop file: {}", e))?;
    } else {
        if desktop_file.exists() {
            fs::remove_file(&desktop_file)
                .map_err(|e| format!("Failed to remove desktop file: {}", e))?;
        }
    }
    
    Ok(())
}

#[cfg(target_os = "macos")]
fn set_autostart_macos(enabled: bool) -> Result<(), String> {
    use std::path::PathBuf;
    use std::fs;
    
    let app_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get app path: {}", e))?;
    
    let launch_agents_dir = dirs::home_dir()
        .ok_or("Failed to get home directory")?
        .join("Library/LaunchAgents");
    
    let plist_file = launch_agents_dir.join("com.network-interface-manager.plist");
    
    if enabled {
        fs::create_dir_all(&launch_agents_dir)
            .map_err(|e| format!("Failed to create LaunchAgents directory: {}", e))?;
        
        let content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.network-interface-manager</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>"#,
            app_path.display()
        );
        
        fs::write(&plist_file, content)
            .map_err(|e| format!("Failed to write plist file: {}", e))?;
    } else {
        if plist_file.exists() {
            fs::remove_file(&plist_file)
                .map_err(|e| format!("Failed to remove plist file: {}", e))?;
        }
    }
    
    Ok(())
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

    let mut cmd = format!("netsh interface ip set dns \"{}\" static {}", 
        config.interface_name, config.dns_servers[0]);

    for dns in &config.dns_servers[1..] {
        cmd.push_str(&format!(" & netsh interface ip add dns \"{}\" {}", 
            config.interface_name, dns));
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
    let cmd = format!("echo 'nameserver {}' | sudo tee /etc/resolv.conf > /dev/null", dns_list.replace(" ", "\nnameserver "));

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
    
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;
    
    let exe_path_str = exe_path.to_string_lossy().to_string();
    let exe_path_cstr = CString::new(exe_path_str)
        .map_err(|e| format!("Failed to create CString: {}", e))?;
    
    let operation = CString::new("runas")
        .map_err(|e| format!("Failed to create operation string: {}", e))?;
    
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
    let tray = TrayIconBuilder::new()
        .menu(&menu)
        .icon(app.default_window_icon().unwrap().clone())
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            if event.id.as_ref() == quit.id().as_ref() {
                app.exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| {
            use tauri::tray::TrayIconEvent;
            match event {
                TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } => {
                    // 左键点击托盘图标切换显示/隐藏
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                _ => {}
            }
        })
        .build(app)?;
    
    // 保持托盘图标活跃
    std::mem::forget(tray);
    
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
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_all_network_interface,
            set_dns_servers,
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
            set_autostart
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
                    window.hide().ok();
                    api.prevent_close();
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

