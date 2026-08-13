use crate::db::{
    attach_assignees, db, models::*, task_with_assignees,
};
use chrono::{Datelike, Local, NaiveDate};
use rusqlite::Result;

fn add_months(from: NaiveDate, months: i32) -> NaiveDate {
    let total_months = from.year() * 12 + (from.month() as i32 - 1) + months;
    let year = total_months / 12;
    let month = (total_months % 12) as u32 + 1;
    NaiveDate::from_ymd_opt(year, month, 1).unwrap()
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
    let next_month_year = if month == 12 { year + 1 } else { year };
    let next_month = if month == 12 { 1 } else { month + 1 };
    let first_of_next = NaiveDate::from_ymd_opt(next_month_year, next_month, 1).unwrap();
    first_of_next.pred_opt().unwrap().day()
}

fn next_due_date(from: NaiveDate, cycle_type: &str, cycle_day: u32) -> NaiveDate {
    let next_first = match cycle_type {
        "monthly" => add_months(from, 1),
        "quarterly" => add_months(from, 3),
        "halfyearly" => add_months(from, 6),
        "yearly" => add_months(from, 12),
        _ => from,
    };

    let last_day = last_day_of_month(next_first.year(), next_first.month());
    let day = cycle_day.min(last_day);
    NaiveDate::from_ymd_opt(next_first.year(), next_first.month(), day).unwrap()
}

// Ensure that for a given task, future instances are generated up to a horizon date.
pub fn ensure_instances(
    conn: &mut rusqlite::Connection,
    task_id: i64,
    horizon: NaiveDate,
) -> Result<(), String> {
    let task = conn
        .query_row(
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
        )
        .map_err(|e| e.to_string())?;

    if let Some(end_date) = task.end_date {
        if end_date < task.start_date {
            return Ok(());
        }
    }

    // Find the latest existing instance due date
    let latest_date: Option<NaiveDate> = conn
        .query_row(
            "SELECT MAX(due_date) FROM task_instances WHERE task_id = ?1",
            [task_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let mut current_date = if let Some(d) = latest_date {
        d
    } else {
        let start = task.start_date;
        let last_day = last_day_of_month(start.year(), start.month());
        let day = (task.cycle_day as u32).min(last_day);
        NaiveDate::from_ymd_opt(start.year(), start.month(), day).unwrap()
    };

    let effective_horizon = if let Some(end_date) = task.end_date {
        horizon.min(end_date)
    } else {
        horizon
    };

    if current_date > effective_horizon {
        return Ok(());
    }

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    loop {
        current_date = next_due_date(current_date, &task.cycle_type, task.cycle_day as u32);
        if current_date > effective_horizon {
            break;
        }

        // Check if this date already exists
        let exists: bool = tx
            .query_row(
                "SELECT 1 FROM task_instances WHERE task_id = ?1 AND due_date = ?2",
                rusqlite::params![task_id, current_date],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if !exists {
            tx.execute(
                "INSERT INTO task_instances (task_id, due_date, status, reminded) VALUES (?1, ?2, 'pending', 0)",
                rusqlite::params![task_id, current_date],
            )
            .map_err(|e| e.to_string())?;
        }

        // Safety break to avoid infinite loops
        if current_date.year() > 2100 {
            break;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn create_task(req: CreateTaskRequest) -> Result<TaskWithAssignees, String> {
    let db = db();
    let mut conn = db.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO tasks (title, description, cycle_type, cycle_day, start_date, end_date,
                            reminder_minutes, sound_enabled)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            req.title,
            req.description,
            req.cycle_type,
            req.cycle_day,
            req.start_date,
            req.end_date,
            req.reminder_minutes,
            req.sound_enabled as i32,
        ],
    )
    .map_err(|e| e.to_string())?;

    let task_id = conn.last_insert_rowid();

    if !req.assignee_ids.is_empty() {
        attach_assignees(&mut conn, task_id, &req.assignee_ids).map_err(|e| e.to_string())?;
    }

    // Generate instances for the next 2 years
    let horizon = Local::now().date_naive().checked_add_days(chrono::Days::new(730)).unwrap();
    ensure_instances(&mut conn, task_id, horizon)?;

    let result = task_with_assignees(&conn, task_id).map_err(|e| e.to_string())?;
    Ok(result)
}

#[tauri::command]
pub fn update_task(req: UpdateTaskRequest) -> Result<TaskWithAssignees, String> {
    let db = db();
    let mut conn = db.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE tasks SET title = ?1, description = ?2, cycle_type = ?3, cycle_day = ?4,
                          start_date = ?5, end_date = ?6, reminder_minutes = ?7, sound_enabled = ?8
         WHERE id = ?9",
        rusqlite::params![
            req.title,
            req.description,
            req.cycle_type,
            req.cycle_day,
            req.start_date,
            req.end_date,
            req.reminder_minutes,
            req.sound_enabled as i32,
            req.id,
        ],
    )
    .map_err(|e| e.to_string())?;

    if !req.assignee_ids.is_empty() {
        attach_assignees(&mut conn, req.id, &req.assignee_ids).map_err(|e| e.to_string())?;
    } else {
        conn.execute("DELETE FROM task_assignees WHERE task_id = ?1", [req.id])
            .map_err(|e| e.to_string())?;
    }

    // Regenerate future pending instances after task edit
    conn.execute(
        "DELETE FROM task_instances WHERE task_id = ?1 AND status = 'pending' AND due_date >= date('now')",
        [req.id],
    )
    .map_err(|e| e.to_string())?;

    let horizon = Local::now().date_naive().checked_add_days(chrono::Days::new(730)).unwrap();
    ensure_instances(&mut conn, req.id, horizon)?;

    let result = task_with_assignees(&conn, req.id).map_err(|e| e.to_string())?;
    Ok(result)
}

#[tauri::command]
pub fn delete_task(id: i64) -> Result<(), String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM tasks WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn list_tasks(page: i64, per_page: i64) -> Result<Paginated<TaskWithAssignees>, String> {
    let db = db();
    let mut conn = db.lock().map_err(|e| e.to_string())?;

    // Ensure instances exist for all tasks
    let task_ids: Vec<i64> = {
        let mut stmt = conn
            .prepare("SELECT id FROM tasks")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
    };

    let horizon = Local::now().date_naive().checked_add_days(chrono::Days::new(730)).unwrap();
    for id in task_ids {
        ensure_instances(&mut conn, id, horizon)?;
    }

    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let offset = (page - 1).max(0) * per_page;
    let task_ids: Vec<i64> = {
        let mut stmt = conn
            .prepare("SELECT id FROM tasks ORDER BY created_at DESC LIMIT ?1 OFFSET ?2")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([per_page, offset], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
    };

    let mut results = Vec::new();
    for id in task_ids {
        let task = task_with_assignees(&conn, id).map_err(|e| e.to_string())?;
        results.push(task);
    }

    Ok(Paginated {
        items: results,
        total,
        page,
        per_page,
    })
}

#[tauri::command]
pub fn get_task(id: i64) -> Result<TaskWithAssignees, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;
    let result = task_with_assignees(&conn, id).map_err(|e| e.to_string())?;
    Ok(result)
}
