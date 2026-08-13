use crate::db::db;
use chrono::{Duration, Local};
use std::thread;
use std::time::Duration as StdDuration;
use tauri::Emitter;
use tauri_plugin_notification::NotificationExt;

pub fn start_scheduler(app_handle: tauri::AppHandle) {
    thread::spawn(move || {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("Failed to create tokio runtime: {}", e);
                return;
            }
        };

        rt.block_on(async move {
            let mut ticker = tokio::time::interval(StdDuration::from_secs(60));
            loop {
                ticker.tick().await;
                if let Err(e) = check_and_notify(&app_handle).await {
                    eprintln!("Scheduler error: {}", e);
                }
            }
        });
    });
}

async fn check_and_notify(app_handle: &tauri::AppHandle) -> Result<(), String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let now = Local::now();
    let now_naive = now.naive_local();
    let today = now.date_naive();

    // Update overdue task instances
    conn.execute(
        "UPDATE task_instances SET status = 'overdue'
         WHERE status = 'pending' AND due_date < ?1",
        [today],
    )
    .map_err(|e| e.to_string())?;

    // Notify task instances
    notify_task_instances(app_handle, &conn, now_naive)?;

    // Update overdue archive borrows
    conn.execute(
        "UPDATE archive_borrows SET status = 'overdue'
         WHERE status = 'borrowed' AND due_date < ?1",
        [today],
    )
    .map_err(|e| e.to_string())?;

    // Notify archive borrows
    notify_archive_borrows(app_handle, &conn, today)?;

    Ok(())
}

fn notify_task_instances(
    app_handle: &tauri::AppHandle,
    conn: &rusqlite::Connection,
    now_naive: chrono::NaiveDateTime,
) -> Result<(), String> {
    let mut stmt = conn
        .prepare(
            "SELECT ti.id, ti.task_id, ti.due_date, t.title, t.reminder_minutes, t.sound_enabled
             FROM task_instances ti
             INNER JOIN tasks t ON t.id = ti.task_id
             WHERE ti.status = 'pending' AND ti.reminded = 0
               AND datetime(ti.due_date || ' 00:00:00', '-' || t.reminder_minutes || ' minutes') <= ?1",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([now_naive], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i32>(4)?,
                row.get::<_, i32>(5)? != 0,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut to_notify: Vec<(i64, i64, String, String, i32, bool)> = Vec::new();
    for row in rows {
        to_notify.push(row.map_err(|e| e.to_string())?);
    }

    for (instance_id, task_id, due_date_str, title, reminder_minutes, sound_enabled) in to_notify {
        let assignees: Vec<String> = {
            let mut stmt = conn
                .prepare(
                    "SELECT m.name FROM members m
                     INNER JOIN task_assignees ta ON ta.member_id = m.id
                     WHERE ta.task_id = ?1",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([task_id], |row| row.get(0))
                .map_err(|e| e.to_string())?;
            rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
        };

        let names = if assignees.is_empty() {
            "（未指派）".to_string()
        } else {
            assignees.join("、")
        };

        let body = format!(
            "负责人：{}，截止日期：{}，提前 {} 分钟提醒",
            names, due_date_str, reminder_minutes
        );

        let _ = app_handle
            .notification()
            .builder()
            .title(&title)
            .body(&body)
            .show();

        let _ = app_handle.emit(
            "task-reminder",
            serde_json::json!({
                "instance_id": instance_id,
                "title": title,
                "body": body,
                "sound_enabled": sound_enabled,
            }),
        );

        conn.execute(
            "UPDATE task_instances SET reminded = 1 WHERE id = ?1",
            [instance_id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn notify_archive_borrows(
    app_handle: &tauri::AppHandle,
    conn: &rusqlite::Connection,
    today: chrono::NaiveDate,
) -> Result<(), String> {
    let tomorrow = today + Duration::days(1);

    let mut stmt = conn
        .prepare(
            "SELECT ab.id, ab.archive_id, a.code, a.title, ab.due_date, m.name, ab.status
             FROM archive_borrows ab
             INNER JOIN archives a ON a.id = ab.archive_id
             INNER JOIN members m ON m.id = ab.borrower_id
             WHERE ab.status IN ('borrowed', 'overdue') AND ab.reminded = 0
               AND ab.due_date IN (?1, ?2)",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([today, tomorrow], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut to_notify: Vec<(i64, i64, String, String, String, String, String)> = Vec::new();
    for row in rows {
        to_notify.push(row.map_err(|e| e.to_string())?);
    }

    for (borrow_id, _archive_id, code, title, due_date, borrower, status) in to_notify {
        let (title_msg, body) = if status == "overdue" {
            (
                "档案已逾期未还".to_string(),
                format!(
                    "档案 {}《{}》已于 {} 到期，借阅人：{}，请尽快归还",
                    code, title, due_date, borrower
                ),
            )
        } else if due_date == today.format("%Y-%m-%d").to_string() {
            (
                "档案今日到期".to_string(),
                format!(
                    "档案 {}《{}》今日（{}）到期，借阅人：{}，请及时归还",
                    code, title, due_date, borrower
                ),
            )
        } else {
            (
                "档案即将到期".to_string(),
                format!(
                    "档案 {}《{}》将于 {} 到期，借阅人：{}，请提前准备",
                    code, title, due_date, borrower
                ),
            )
        };

        let _ = app_handle
            .notification()
            .builder()
            .title(&title_msg)
            .body(&body)
            .show();

        let _ = app_handle.emit(
            "archive-reminder",
            serde_json::json!({
                "borrow_id": borrow_id,
                "title": title_msg,
                "body": body,
                "sound_enabled": true,
            }),
        );

        conn.execute(
            "UPDATE archive_borrows SET reminded = 1 WHERE id = ?1",
            [borrow_id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}
