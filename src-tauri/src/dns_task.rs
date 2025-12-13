use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::network_info::get_all_network_interfaces;
use crate::db::Database;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsTask {
    pub id: String,
    pub name: String,
    pub interface_pattern: String,  // 网卡名称匹配规则（支持通配符）
    pub target_dns: Vec<String>,    // 目标DNS服务器
    pub enabled: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatus {
    pub task_id: String,
    pub interface_name: String,
    pub current_dns: Vec<String>,
    pub target_dns: Vec<String>,
    pub status: String,  // "matched", "dns_mismatch", "applied"
    pub last_check: i64,
}

pub struct DnsTaskManager {
    tasks: Arc<Mutex<Vec<DnsTask>>>,
    running: Arc<Mutex<bool>>,
    task_statuses: Arc<Mutex<Vec<TaskStatus>>>,
    db: Arc<Mutex<Option<Database>>>,
    monitoring_enabled: Arc<Mutex<bool>>,
}

impl DnsTaskManager {
    pub fn new() -> Self {
        // 简单初始化，不在这里加载数据库
        DnsTaskManager {
            tasks: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(Mutex::new(false)),
            task_statuses: Arc::new(Mutex::new(Vec::new())),
            db: Arc::new(Mutex::new(None)),
            monitoring_enabled: Arc::new(Mutex::new(false)),
        }
    }

    pub fn init_database(&self) -> Result<(), String> {
        // 初始化数据库
        match Database::new() {
            Ok(database) => {
                // 加载任务
                if let Ok(loaded_tasks) = database.get_all_tasks() {
                    if let Ok(mut tasks) = self.tasks.lock() {
                        *tasks = loaded_tasks;
                    }
                }
                // 加载监控状态
                if let Ok(state) = database.get_monitoring_state() {
                    if let Ok(mut enabled) = self.monitoring_enabled.lock() {
                        *enabled = state;
                    }
                }
                // 保存数据库实例
                if let Ok(mut db) = self.db.lock() {
                    *db = Some(database);
                }
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to initialize database: {}", e);
                Ok(()) // 即使数据库初始化失败，也继续运行
            }
        }
    }

    pub fn restore_monitoring_state(&self) -> Result<(), String> {
        // 等待一下，确保数据库初始化完成
        std::thread::sleep(Duration::from_millis(100));
        
        // 如果之前启用了监控，自动启动
        if let Ok(enabled) = self.monitoring_enabled.lock() {
            if *enabled {
                drop(enabled); // 释放锁
                return self.start_monitoring();
            }
        }
        Ok(())
    }

    pub fn add_task(&self, task: DnsTask) -> Result<(), String> {
        // 保存到数据库
        if let Ok(db_lock) = self.db.lock() {
            if let Some(ref db) = *db_lock {
                db.add_task(&task).map_err(|e| e.to_string())?;
            }
        }

        // 保存到内存
        let mut tasks = self.tasks.lock().map_err(|e| e.to_string())?;
        tasks.push(task);
        Ok(())
    }

    pub fn remove_task(&self, task_id: &str) -> Result<(), String> {
        // 从数据库删除
        if let Ok(db_lock) = self.db.lock() {
            if let Some(ref db) = *db_lock {
                db.remove_task(task_id).map_err(|e| e.to_string())?;
            }
        }

        // 从内存删除
        let mut tasks = self.tasks.lock().map_err(|e| e.to_string())?;
        tasks.retain(|t| t.id != task_id);
        Ok(())
    }

    pub fn get_tasks(&self) -> Result<Vec<DnsTask>, String> {
        let tasks = self.tasks.lock().map_err(|e| e.to_string())?;
        Ok(tasks.clone())
    }

    pub fn update_task(&self, task: DnsTask) -> Result<(), String> {
        // 更新数据库
        if let Ok(db_lock) = self.db.lock() {
            if let Some(ref db) = *db_lock {
                db.update_task(&task).map_err(|e| e.to_string())?;
            }
        }

        // 更新内存
        let mut tasks = self.tasks.lock().map_err(|e| e.to_string())?;
        if let Some(pos) = tasks.iter().position(|t| t.id == task.id) {
            tasks[pos] = task;
            Ok(())
        } else {
            Err("Task not found".to_string())
        }
    }

    pub fn get_task_statuses(&self) -> Result<Vec<TaskStatus>, String> {
        let statuses = self.task_statuses.lock().map_err(|e| e.to_string())?;
        Ok(statuses.clone())
    }

    pub fn start_monitoring(&self) -> Result<(), String> {
        let mut running = self.running.lock().map_err(|e| e.to_string())?;
        if *running {
            return Err("Already running".to_string());
        }
        *running = true;

        // 保存监控状态到数据库
        if let Ok(db_lock) = self.db.lock() {
            if let Some(ref db) = *db_lock {
                let _ = db.save_monitoring_state(true);
            }
        }

        // 保存监控状态到内存
        if let Ok(mut enabled) = self.monitoring_enabled.lock() {
            *enabled = true;
        }

        let tasks = Arc::clone(&self.tasks);
        let task_statuses = Arc::clone(&self.task_statuses);
        let running_flag = Arc::clone(&self.running);

        thread::spawn(move || {
            loop {
                // 检查是否应该继续运行
                let should_continue = match running_flag.lock() {
                    Ok(flag) => *flag,
                    Err(_) => break, // 如果锁被poisoned，退出线程
                };

                if !should_continue {
                    break;
                }

                // 使用catch_unwind保护get_all_network_interfaces调用
                let interfaces_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    get_all_network_interfaces()
                }));

                let interfaces = match interfaces_result {
                    Ok(Ok(ifaces)) => ifaces,
                    Ok(Err(_)) => {
                        thread::sleep(Duration::from_secs(1));
                        continue;
                    }
                    Err(_) => {
                        eprintln!("Panic caught in get_all_network_interfaces");
                        thread::sleep(Duration::from_secs(1));
                        continue;
                    }
                };

                // 安全地获取任务列表
                let tasks_list = match tasks.lock() {
                    Ok(list) => list.clone(),
                    Err(_) => {
                        // 如果锁被poisoned，跳过这次迭代
                        thread::sleep(Duration::from_secs(1));
                        continue;
                    }
                };

                let mut statuses = Vec::new();

                    for task in tasks_list.iter() {
                        if !task.enabled {
                            continue;
                        }

                        for iface in &interfaces {
                            if !iface.enabled {
                                continue;
                            }

                            // 匹配网卡名称
                            if matches_pattern(&iface.name, &task.interface_pattern) {
                                let current_dns = iface.dns_servers.clone();
                                let target_dns = task.target_dns.clone();

                                let status_str = if current_dns == target_dns {
                                    "matched".to_string()
                                } else {
                                    // DNS不匹配，尝试设置
                                    // 使用catch_unwind来防止panic导致线程崩溃
                                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                        set_interface_dns(&iface.name, &target_dns)
                                    }));

                                    match result {
                                        Ok(Ok(_)) => "applied".to_string(),
                                        Ok(Err(_)) => "dns_mismatch".to_string(),
                                        Err(_) => {
                                            eprintln!("Panic caught in set_interface_dns");
                                            "dns_mismatch".to_string()
                                        }
                                    }
                                };

                                statuses.push(TaskStatus {
                                    task_id: task.id.clone(),
                                    interface_name: iface.name.clone(),
                                    current_dns,
                                    target_dns,
                                    status: status_str,
                                    last_check: chrono::Local::now().timestamp(),
                                });
                            }
                        }
                    }

                // 安全地更新状态
                if let Ok(mut status_lock) = task_statuses.lock() {
                    *status_lock = statuses;
                }

                thread::sleep(Duration::from_secs(1));
            }
        });

        Ok(())
    }

    pub fn stop_monitoring(&self) -> Result<(), String> {
        let mut running = self.running.lock().map_err(|e| e.to_string())?;
        *running = false;

        // 保存监控状态到数据库
        if let Ok(db_lock) = self.db.lock() {
            if let Some(ref db) = *db_lock {
                let _ = db.save_monitoring_state(false);
            }
        }

        // 保存监控状态到内存
        if let Ok(mut enabled) = self.monitoring_enabled.lock() {
            *enabled = false;
        }

        Ok(())
    }

    pub fn is_running(&self) -> Result<bool, String> {
        let running = self.running.lock().map_err(|e| e.to_string())?;
        Ok(*running)
    }
}

fn matches_pattern(name: &str, pattern: &str) -> bool {
    // 简单的通配符匹配
    if pattern == "*" {
        return true;
    }

    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        let mut pos = 0;

        for (i, part) in parts.iter().enumerate() {
            if i == 0 {
                if !name.starts_with(part) {
                    return false;
                }
                pos += part.len();
            } else if i == parts.len() - 1 {
                if !name.ends_with(part) {
                    return false;
                }
            } else {
                if let Some(found_pos) = name[pos..].find(part) {
                    pos += found_pos + part.len();
                } else {
                    return false;
                }
            }
        }
        true
    } else {
        name == pattern
    }
}

fn set_interface_dns(interface_name: &str, dns_servers: &[String]) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    return set_dns_windows_internal(interface_name, dns_servers);

    #[cfg(target_os = "linux")]
    return set_dns_linux_internal(interface_name, dns_servers);

    #[cfg(target_os = "macos")]
    return set_dns_macos_internal(interface_name, dns_servers);

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    Err("Unsupported platform".to_string())
}

#[cfg(target_os = "windows")]
fn set_dns_windows_internal(interface_name: &str, dns_servers: &[String]) -> Result<(), String> {
    if dns_servers.is_empty() {
        return Err("DNS servers list is empty".to_string());
    }

    let mut cmd = format!("netsh interface ip set dns \"{}\" static {}", 
        interface_name, dns_servers[0]);

    for dns in &dns_servers[1..] {
        cmd.push_str(&format!(" & netsh interface ip add dns \"{}\" {}", 
            interface_name, dns));
    }

    let output = Command::new("cmd")
        .args(&["/C", &cmd])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[cfg(target_os = "linux")]
fn set_dns_linux_internal(interface_name: &str, dns_servers: &[String]) -> Result<(), String> {
    let dns_list = dns_servers.join(" ");
    let cmd = format!("echo 'nameserver {}' | sudo tee /etc/resolv.conf > /dev/null", 
        dns_list.replace(" ", "\nnameserver "));

    let output = Command::new("sh")
        .args(&["-c", &cmd])
        .output()
        .map_err(|e| format!("Failed to set DNS: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[cfg(target_os = "macos")]
fn set_dns_macos_internal(interface_name: &str, dns_servers: &[String]) -> Result<(), String> {
    let dns_list = dns_servers.join(" ");
    let output = Command::new("sudo")
        .args(&["networksetup", "-setdnsservers", interface_name])
        .arg(&dns_list)
        .output()
        .map_err(|e| format!("Failed to set DNS: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

// 全局任务管理器实例
lazy_static::lazy_static! {
    pub static ref TASK_MANAGER: DnsTaskManager = DnsTaskManager::new();
}
