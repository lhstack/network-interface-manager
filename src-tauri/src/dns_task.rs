use crate::db::Database;
use crate::network_info::get_all_network_interfaces;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsTask {
    pub id: String,
    pub name: String,
    pub interface_pattern: String, // 网卡名称匹配规则（支持通配符）
    pub target_dns: Vec<String>,   // 目标DNS服务器
    pub enabled: bool,
    pub created_at: i64,
    #[serde(default = "default_interval")]
    pub interval: u64,             // 检查间隔（秒），默认1秒
}

fn default_interval() -> u64 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatus {
    pub task_id: String,
    pub task_name: String,
    pub interface_name: String,
    pub current_dns: Vec<String>,
    pub target_dns: Vec<String>,
    pub status: String, // "matched", "dns_mismatch", "applied", "running", "stopped"
    pub last_check: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub time: String,
    pub task_id: String,
    pub task_name: String,
    pub message: String,
}

pub struct DnsTaskManager {
    tasks: Arc<Mutex<Vec<DnsTask>>>,
    running: Arc<Mutex<bool>>,
    task_statuses: Arc<Mutex<Vec<TaskStatus>>>,
    db: Arc<Mutex<Option<Database>>>,
    monitoring_enabled: Arc<Mutex<bool>>,
    logs: Arc<Mutex<Vec<LogEntry>>>,
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
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    // 获取日志
    pub fn get_logs(&self) -> Result<Vec<LogEntry>, String> {
        let logs = self.logs.lock().map_err(|e| e.to_string())?;
        Ok(logs.clone())
    }
    
    // 清空日志
    pub fn clear_logs(&self) -> Result<(), String> {
        let mut logs = self.logs.lock().map_err(|e| e.to_string())?;
        logs.clear();
        Ok(())
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
        let logs = Arc::clone(&self.logs);

        thread::spawn(move || {
            let mut last_check_times: std::collections::HashMap<String, std::time::Instant> = std::collections::HashMap::new();
            
            loop {
                // 检查是否应该继续运行
                let should_continue = match running_flag.lock() {
                    Ok(flag) => *flag,
                    Err(_) => break,
                };

                if !should_continue {
                    break;
                }

                // 使用catch_unwind保护get_all_network_interfaces调用
                let interfaces_result =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        get_all_network_interfaces()
                    }));

                let interfaces = match interfaces_result {
                    Ok(Ok(ifaces)) => ifaces,
                    Ok(Err(_)) => {
                        thread::sleep(Duration::from_millis(500));
                        continue;
                    }
                    Err(_) => {
                        thread::sleep(Duration::from_millis(500));
                        continue;
                    }
                };

                // 安全地获取任务列表
                let tasks_list = match tasks.lock() {
                    Ok(list) => list.clone(),
                    Err(_) => {
                        thread::sleep(Duration::from_millis(500));
                        continue;
                    }
                };

                let mut statuses = Vec::new();
                let now = std::time::Instant::now();

                for task in tasks_list.iter() {
                    if !task.enabled {
                        statuses.push(TaskStatus {
                            task_id: task.id.clone(),
                            task_name: task.name.clone(),
                            interface_name: task.interface_pattern.clone(),
                            current_dns: vec![],
                            target_dns: task.target_dns.clone(),
                            status: "stopped".to_string(),
                            last_check: "-".to_string(),
                            message: "任务已禁用".to_string(),
                        });
                        continue;
                    }

                    // 检查是否到达检查间隔
                    let interval = if task.interval < 1 { 1 } else { task.interval };
                    let should_check = match last_check_times.get(&task.id) {
                        Some(last_time) => now.duration_since(*last_time).as_secs() >= interval,
                        None => true,
                    };

                    for iface in &interfaces {
                        if !iface.enabled {
                            continue;
                        }

                        // 匹配网卡名称
                        if matches_pattern(&iface.name, &task.interface_pattern) {
                            let current_dns = iface.dns_servers.clone();
                            let target_dns = task.target_dns.clone();
                            let last_check_str = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

                            let (status_str, message) = if !should_check {
                                // 还没到检查时间
                                ("running".to_string(), "等待下次检查".to_string())
                            } else if dns_equal(&current_dns, &target_dns) {
                                ("matched".to_string(), "DNS配置正确".to_string())
                            } else {
                                // DNS不匹配，尝试设置
                                let result =
                                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                        set_interface_dns(&iface.name, &target_dns)
                                    }));

                                // 更新最后检查时间
                                last_check_times.insert(task.id.clone(), now);

                                match result {
                                    Ok(Ok(_)) => {
                                        // 添加日志
                                        if let Ok(mut log_lock) = logs.lock() {
                                            log_lock.insert(0, LogEntry {
                                                time: last_check_str.clone(),
                                                task_id: task.id.clone(),
                                                task_name: task.name.clone(),
                                                message: format!("DNS已设置: {} -> {:?}", iface.name, target_dns),
                                            });
                                            if log_lock.len() > 100 {
                                                log_lock.truncate(100);
                                            }
                                        }
                                        // 刷新DNS缓存
                                        #[cfg(target_os = "windows")]
                                        flush_dns_cache();
                                        ("applied".to_string(), "DNS已自动设置".to_string())
                                    }
                                    Ok(Err(e)) => {
                                        if let Ok(mut log_lock) = logs.lock() {
                                            log_lock.insert(0, LogEntry {
                                                time: last_check_str.clone(),
                                                task_id: task.id.clone(),
                                                task_name: task.name.clone(),
                                                message: format!("设置DNS失败: {}", e),
                                            });
                                            if log_lock.len() > 100 {
                                                log_lock.truncate(100);
                                            }
                                        }
                                        ("dns_mismatch".to_string(), format!("设置失败: {}", e))
                                    }
                                    Err(_) => {
                                        ("dns_mismatch".to_string(), "设置DNS时发生错误".to_string())
                                    }
                                }
                            };

                            statuses.push(TaskStatus {
                                task_id: task.id.clone(),
                                task_name: task.name.clone(),
                                interface_name: iface.name.clone(),
                                current_dns,
                                target_dns,
                                status: status_str,
                                last_check: last_check_str,
                                message,
                            });
                        }
                    }
                }

                // 安全地更新状态
                if let Ok(mut status_lock) = task_statuses.lock() {
                    *status_lock = statuses;
                }

                thread::sleep(Duration::from_millis(500));
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

// DNS比较函数（忽略顺序）
fn dns_equal(a: &[String], b: &[String]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut a_set: std::collections::HashSet<&String> = a.iter().collect();
    for dns in b {
        if !a_set.remove(dns) {
            return false;
        }
    }
    a_set.is_empty()
}

// 刷新DNS缓存
#[cfg(target_os = "windows")]
fn flush_dns_cache() {
    let _ = Command::new("cmd")
        .args(&["/C", "ipconfig /flushdns"])
        .creation_flags(0x08000000)
        .output();
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

    let mut cmd = format!(
        "netsh interface ip set dns \"{}\" static {}",
        interface_name, dns_servers[0]
    );

    for dns in &dns_servers[1..] {
        cmd.push_str(&format!(
            " & netsh interface ip add dns \"{}\" {}",
            interface_name, dns
        ));
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
    let cmd = format!(
        "echo 'nameserver {}' | sudo tee /etc/resolv.conf > /dev/null",
        dns_list.replace(" ", "\nnameserver ")
    );

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
