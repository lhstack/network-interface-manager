use crate::dns_task::DnsTask;
use rusqlite::{params, Connection, Result as SqliteResult};
use std::fs;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // 获取用户主目录
        let home_dir = dirs::home_dir().ok_or("Failed to get home directory")?;

        // 创建.network interface manager目录
        let data_dir = home_dir.join(".network interface manager");
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir)?;
        }

        let db_path = data_dir.join("tasks.db");
        let conn = Connection::open(&db_path)?;

        let db = Database { conn };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> SqliteResult<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS dns_tasks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                interface_pattern TEXT NOT NULL,
                target_dns TEXT NOT NULL,
                enabled INTEGER NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS monitoring_state (
                id INTEGER PRIMARY KEY,
                enabled INTEGER NOT NULL
            )",
            [],
        )?;

        // 初始化监控状态
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM monitoring_state", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);

        if count == 0 {
            self.conn.execute(
                "INSERT INTO monitoring_state (id, enabled) VALUES (1, 0)",
                [],
            )?;
        }

        Ok(())
    }

    pub fn add_task(&self, task: &DnsTask) -> Result<(), Box<dyn std::error::Error>> {
        let target_dns_json = serde_json::to_string(&task.target_dns)?;

        self.conn.execute(
            "INSERT INTO dns_tasks (id, name, interface_pattern, target_dns, enabled, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &task.id,
                &task.name,
                &task.interface_pattern,
                &target_dns_json,
                task.enabled as i32,
                task.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn remove_task(&self, task_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.conn
            .execute("DELETE FROM dns_tasks WHERE id = ?1", params![task_id])?;
        Ok(())
    }

    pub fn update_task(&self, task: &DnsTask) -> Result<(), Box<dyn std::error::Error>> {
        let target_dns_json = serde_json::to_string(&task.target_dns)?;

        self.conn.execute(
            "UPDATE dns_tasks SET name = ?1, interface_pattern = ?2, target_dns = ?3, enabled = ?4
             WHERE id = ?5",
            params![
                &task.name,
                &task.interface_pattern,
                &target_dns_json,
                task.enabled as i32,
                &task.id,
            ],
        )?;
        Ok(())
    }

    pub fn get_all_tasks(&self) -> Result<Vec<DnsTask>, Box<dyn std::error::Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, interface_pattern, target_dns, enabled, created_at FROM dns_tasks",
        )?;

        let tasks = stmt
            .query_map([], |row| {
                let target_dns_json: String = row.get(3)?;
                let target_dns: Vec<String> =
                    serde_json::from_str(&target_dns_json).unwrap_or_default();

                Ok(DnsTask {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    interface_pattern: row.get(2)?,
                    target_dns,
                    enabled: row.get::<_, i32>(4)? != 0,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(tasks)
    }

    pub fn save_monitoring_state(&self, enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute(
            "UPDATE monitoring_state SET enabled = ?1 WHERE id = 1",
            params![enabled as i32],
        )?;
        Ok(())
    }

    pub fn get_monitoring_state(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let enabled: i32 = self
            .conn
            .query_row(
                "SELECT enabled FROM monitoring_state WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        Ok(enabled != 0)
    }
}
