use once_cell::sync::OnceCell;
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub mod migrations;
pub mod models;

use models::*;

static DB: Mutex<Option<Arc<Mutex<Connection>>>> = Mutex::new(None);
static DB_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);
static APP_DIR: OnceCell<PathBuf> = OnceCell::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppConfig {
    #[serde(default)]
    db_path: Option<PathBuf>,
    #[serde(default)]
    ai_config: AiConfig,
}

fn config_path() -> Option<PathBuf> {
    APP_DIR.get().map(|d| d.join("config.json"))
}

fn load_config() -> AppConfig {
    if let Some(path) = config_path() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                return config;
            }
        }
    }
    AppConfig {
        db_path: None,
        ai_config: AiConfig::default(),
    }
}

fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = config_path().ok_or("App dir not initialized")?;
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(())
}

fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS migrations_applied (id TEXT PRIMARY KEY)",
        [],
    )?;

    for (id, sql) in migrations::MIGRATIONS {
        let applied: bool = conn
            .query_row(
                "SELECT 1 FROM migrations_applied WHERE id = ?1",
                [id],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if !applied {
            conn.execute_batch(sql)?;
            conn.execute("INSERT INTO migrations_applied (id) VALUES (?1)", [id])?;
        }
    }

    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    Ok(())
}

pub fn is_db_configured(app_dir: &Path) -> bool {
    // Explicitly configured via the settings page or the onboarding wizard
    let config_path = app_dir.join("config.json");
    if let Ok(content) = fs::read_to_string(config_path) {
        if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
            if config.db_path.is_some() {
                return true;
            }
        }
    }
    // Existing install without a config file: a default database already
    // exists in the app data dir, so skip the onboarding wizard.
    app_dir.join("archivemanage.db").exists() || app_dir.join("task_reminder.db").exists()
}

pub fn set_app_dir(app_dir: PathBuf) -> Result<(), String> {
    APP_DIR.set(app_dir).map_err(|_| {
        "App dir already initialized".to_string()
    })?;
    Ok(())
}

fn open_db_at(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建数据库目录失败：{}", e))?;
    }
    let conn = Connection::open(path).map_err(|e| format!("打开数据库失败：{}", e))?;
    run_migrations(&conn).map_err(|e| format!("执行数据库迁移失败：{}", e))?;
    Ok(conn)
}

pub fn init_db(app_dir: PathBuf) -> Result<(), String> {
    if APP_DIR.get().is_none() {
        APP_DIR.set(app_dir.clone()).map_err(|_| {
            "App dir already initialized".to_string()
        })?;
    }

    let config = load_config();
    let db_path = config
        .db_path
        .clone()
        .unwrap_or_else(|| app_dir.join("archivemanage.db"));

    // Migrate from old default database name
    if !db_path.exists() {
        let old_path = app_dir.join("task_reminder.db");
        if old_path.exists() {
            let _ = fs::rename(&old_path, &db_path);
        }
    }

    set_active_db(db_path)?;
    Ok(())
}

fn set_active_db(db_path: PathBuf) -> Result<(), String> {
    let conn = open_db_at(&db_path)?;

    let mut db_guard = DB.lock().map_err(|e| e.to_string())?;
    *db_guard = Some(Arc::new(Mutex::new(conn)));

    let mut path_guard = DB_PATH.lock().map_err(|e| e.to_string())?;
    *path_guard = Some(db_path);

    Ok(())
}

pub fn db() -> Arc<Mutex<Connection>> {
    DB.lock()
        .expect("DB mutex poisoned")
        .as_ref()
        .expect("Database not initialized")
        .clone()
}

pub fn current_db_path() -> Option<PathBuf> {
    DB_PATH.lock().ok()?.clone()
}

pub fn default_db_path() -> Option<PathBuf> {
    APP_DIR.get().map(|d| d.join("archivemanage.db"))
}

pub fn get_ai_config() -> Result<AiConfig, String> {
    let config = load_config();
    Ok(config.ai_config)
}

pub fn set_ai_config(ai_config: AiConfig) -> Result<(), String> {
    let mut config = load_config();
    config.ai_config = ai_config;
    save_config(&config)
}

pub fn set_db_path(new_path: PathBuf, migrate: bool) -> Result<(), String> {
    let current_path = current_db_path();

    if migrate && !new_path.exists() {
        if let Some(ref old) = current_path {
            if old.exists() && old != &new_path {
                fs::copy(old, &new_path).map_err(|e| {
                    format!("迁移数据失败：无法从 {} 复制到 {}，错误：{}", old.display(), new_path.display(), e)
                })?;
            }
        }
    }

    set_active_db(new_path.clone())?;

    let mut config = load_config();
    config.db_path = Some(new_path);
    save_config(&config)?;

    Ok(())
}

// Helper to load assignees for a task
pub fn get_assignees(conn: &Connection, task_id: i64) -> Result<Vec<Member>> {
    let mut stmt = conn.prepare(
        "SELECT m.id, m.name, m.phone, m.email, m.note, m.created_at
         FROM members m
         INNER JOIN task_assignees ta ON ta.member_id = m.id
         WHERE ta.task_id = ?1
         ORDER BY m.name"
    )?;

    let members = stmt.query_map([task_id], |row| {
        Ok(Member {
            id: row.get(0)?,
            name: row.get(1)?,
            phone: row.get(2)?,
            email: row.get(3)?,
            note: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;

    members.collect()
}

// Helper to attach assignees to task
pub fn attach_assignees(conn: &mut Connection, task_id: i64, assignee_ids: &[i64]) -> Result<()> {
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM task_assignees WHERE task_id = ?1", [task_id])?;
    {
        let mut stmt = tx.prepare("INSERT INTO task_assignees (task_id, member_id) VALUES (?1, ?2)")?;
        for id in assignee_ids {
            stmt.execute([task_id, *id])?;
        }
    }
    tx.commit()
}

// Build TaskWithAssignees from a task row
pub fn task_with_assignees(conn: &Connection, task_id: i64) -> Result<TaskWithAssignees> {
    let task = conn.query_row(
        "SELECT id, title, description, cycle_type, cycle_day, start_date, end_date,
                reminder_minutes, sound_enabled, created_at
         FROM tasks WHERE id = ?1",
        [task_id],
        |row| {
            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                cycle_type: row.get(3)?,
                cycle_day: row.get(4)?,
                start_date: row.get(5)?,
                end_date: row.get(6)?,
                reminder_minutes: row.get(7)?,
                sound_enabled: row.get::<_, i32>(8)? != 0,
                created_at: row.get(9)?,
            })
        },
    )?;

    let assignees = get_assignees(conn, task_id)?;

    Ok(TaskWithAssignees { task, assignees })
}

// Build instance detail
pub fn instance_detail(conn: &Connection, instance_id: i64) -> Result<TaskInstanceDetail> {
    let instance = conn.query_row(
        "SELECT id, task_id, due_date, status, confirmed_at, reminded, created_at
         FROM task_instances WHERE id = ?1",
        [instance_id],
        |row| {
            Ok(TaskInstance {
                id: row.get(0)?,
                task_id: row.get(1)?,
                due_date: row.get(2)?,
                status: row.get(3)?,
                confirmed_at: row.get(4)?,
                reminded: row.get::<_, i32>(5)? != 0,
                created_at: row.get(6)?,
            })
        },
    )?;

    let task = conn.query_row(
        "SELECT id, title, description, cycle_type, cycle_day, start_date, end_date,
                reminder_minutes, sound_enabled, created_at
         FROM tasks WHERE id = ?1",
        [instance.task_id],
        |row| {
            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                cycle_type: row.get(3)?,
                cycle_day: row.get(4)?,
                start_date: row.get(5)?,
                end_date: row.get(6)?,
                reminder_minutes: row.get(7)?,
                sound_enabled: row.get::<_, i32>(8)? != 0,
                created_at: row.get(9)?,
            })
        },
    )?;

    let assignees = get_assignees(conn, instance.task_id)?;

    Ok(TaskInstanceDetail {
        instance,
        task,
        assignees,
    })
}
