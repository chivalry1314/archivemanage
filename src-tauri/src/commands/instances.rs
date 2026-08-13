use crate::db::{db, instance_detail, models::*};
use chrono::Local;
use rusqlite::Result;

#[tauri::command]
pub fn list_instances(
    filter: Option<String>,
    page: i64,
    per_page: i64,
) -> Result<Paginated<TaskInstanceDetail>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM task_instances WHERE (?1 IS NULL OR status = ?1)",
            [filter.as_deref()],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let offset = (page - 1).max(0) * per_page;
    let mut stmt = conn
        .prepare(
            "SELECT id FROM task_instances
             WHERE (?1 IS NULL OR status = ?1)
             ORDER BY due_date DESC, id DESC
             LIMIT ?2 OFFSET ?3",
        )
        .map_err(|e| e.to_string())?;

    let ids: Vec<i64> = stmt
        .query_map(rusqlite::params![filter.as_deref(), per_page, offset], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for id in ids {
        let detail = instance_detail(&conn, id).map_err(|e| e.to_string())?;
        results.push(detail);
    }

    Ok(Paginated {
        items: results,
        total,
        page,
        per_page,
    })
}

#[tauri::command]
pub fn list_task_instances(
    task_id: i64,
    page: i64,
    per_page: i64,
) -> Result<Paginated<TaskInstanceDetail>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM task_instances WHERE task_id = ?1",
            [task_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let offset = (page - 1).max(0) * per_page;
    let mut stmt = conn
        .prepare(
            "SELECT id FROM task_instances
             WHERE task_id = ?1
             ORDER BY due_date DESC, id DESC
             LIMIT ?2 OFFSET ?3",
        )
        .map_err(|e| e.to_string())?;

    let ids: Vec<i64> = stmt
        .query_map(rusqlite::params![task_id, per_page, offset], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for id in ids {
        let detail = instance_detail(&conn, id).map_err(|e| e.to_string())?;
        results.push(detail);
    }

    Ok(Paginated {
        items: results,
        total,
        page,
        per_page,
    })
}

#[tauri::command]
pub fn get_today_instances(
    page: i64,
    per_page: i64,
) -> Result<Paginated<TaskInstanceDetail>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let today = Local::now().date_naive();

    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM task_instances WHERE due_date = ?1 AND status != 'completed'",
            [today],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let offset = (page - 1).max(0) * per_page;
    let mut stmt = conn
        .prepare(
            "SELECT id FROM task_instances
             WHERE due_date = ?1 AND status != 'completed'
             ORDER BY id DESC
             LIMIT ?2 OFFSET ?3",
        )
        .map_err(|e| e.to_string())?;

    let ids: Vec<i64> = stmt
        .query_map(rusqlite::params![today, per_page, offset], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for id in ids {
        let detail = instance_detail(&conn, id).map_err(|e| e.to_string())?;
        results.push(detail);
    }

    Ok(Paginated {
        items: results,
        total,
        page,
        per_page,
    })
}

#[tauri::command]
pub fn get_pending_instances(
    page: i64,
    per_page: i64,
) -> Result<Paginated<TaskInstanceDetail>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let today = Local::now().date_naive();

    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM task_instances WHERE status = 'pending' AND due_date >= ?1",
            [today],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let offset = (page - 1).max(0) * per_page;
    let mut stmt = conn
        .prepare(
            "SELECT id FROM task_instances
             WHERE status = 'pending' AND due_date >= ?1
             ORDER BY due_date ASC, id ASC
             LIMIT ?2 OFFSET ?3",
        )
        .map_err(|e| e.to_string())?;

    let ids: Vec<i64> = stmt
        .query_map(rusqlite::params![today, per_page, offset], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for id in ids {
        let detail = instance_detail(&conn, id).map_err(|e| e.to_string())?;
        results.push(detail);
    }

    Ok(Paginated {
        items: results,
        total,
        page,
        per_page,
    })
}

#[tauri::command]
pub fn get_overdue_instances(
    page: i64,
    per_page: i64,
) -> Result<Paginated<TaskInstanceDetail>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let today = Local::now().date_naive();

    conn.execute(
        "UPDATE task_instances SET status = 'overdue'
         WHERE status = 'pending' AND due_date < ?1",
        [today],
    )
    .map_err(|e| e.to_string())?;

    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM task_instances WHERE status = 'overdue'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let offset = (page - 1).max(0) * per_page;
    let mut stmt = conn
        .prepare(
            "SELECT id FROM task_instances
             WHERE status = 'overdue'
             ORDER BY due_date DESC, id DESC
             LIMIT ?1 OFFSET ?2",
        )
        .map_err(|e| e.to_string())?;

    let ids: Vec<i64> = stmt
        .query_map([per_page, offset], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for id in ids {
        let detail = instance_detail(&conn, id).map_err(|e| e.to_string())?;
        results.push(detail);
    }

    Ok(Paginated {
        items: results,
        total,
        page,
        per_page,
    })
}

#[tauri::command]
pub fn complete_instance(id: i64) -> Result<TaskInstanceDetail, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let now = Local::now().naive_local();
    conn.execute(
        "UPDATE task_instances SET status = 'completed', confirmed_at = ?1 WHERE id = ?2",
        (now, id),
    )
    .map_err(|e| e.to_string())?;

    let detail = instance_detail(&conn, id).map_err(|e| e.to_string())?;
    Ok(detail)
}

#[tauri::command]
pub fn uncomplete_instance(id: i64) -> Result<TaskInstanceDetail, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE task_instances SET status = 'pending', confirmed_at = NULL WHERE id = ?1",
        [id],
    )
    .map_err(|e| e.to_string())?;

    let detail = instance_detail(&conn, id).map_err(|e| e.to_string())?;
    Ok(detail)
}

#[tauri::command]
pub fn get_dashboard_stats() -> Result<DashboardStats, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    // Mark overdue
    let today = Local::now().date_naive();
    conn.execute(
        "UPDATE task_instances SET status = 'overdue'
         WHERE status = 'pending' AND due_date < ?1",
        [today],
    )
    .map_err(|e| e.to_string())?;

    let pending_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM task_instances WHERE status = 'pending'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let overdue_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM task_instances WHERE status = 'overdue'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let completed_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM task_instances WHERE status = 'completed'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let today_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM task_instances WHERE due_date = ?1 AND status != 'completed'",
            [today],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(DashboardStats {
        pending_count,
        overdue_count,
        completed_count,
        today_count,
    })
}
