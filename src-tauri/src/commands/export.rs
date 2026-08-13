use crate::db::{current_db_path, db, instance_detail, set_db_path};
use chrono::{NaiveDate, NaiveDateTime};
use serde_json::json;
use std::collections::HashMap;

#[tauri::command]
pub fn export_instances_csv() -> Result<String, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id FROM task_instances
             ORDER BY due_date DESC, id DESC",
        )
        .map_err(|e| e.to_string())?;

    let ids: Vec<i64> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut csv = String::from("\u{FEFF}");
    csv.push_str("任务名称,周期类型,截止日期,状态,被指派人,确认完成时间\n");

    for id in ids {
        let detail = instance_detail(&conn, id).map_err(|e| e.to_string())?;
        let assignees = detail
            .assignees
            .iter()
            .map(|m| m.name.as_str())
            .collect::<Vec<_>>()
            .join("、");

        let cycle_type = match detail.task.cycle_type.as_str() {
            "monthly" => "每月",
            "quarterly" => "每季度",
            "halfyearly" => "每半年",
            "yearly" => "每年",
            _ => &detail.task.cycle_type,
        };

        let status = match detail.instance.status.as_str() {
            "pending" => "待办",
            "completed" => "已完成",
            "overdue" => "逾期",
            _ => &detail.instance.status,
        };

        let confirmed_at = detail
            .instance
            .confirmed_at
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_default();

        csv.push_str(&format!(
            "{},{},{},{},{},{}\n",
            escape_csv(&detail.task.title),
            cycle_type,
            detail.instance.due_date.format("%Y-%m-%d"),
            status,
            escape_csv(&assignees),
            confirmed_at,
        ));
    }

    Ok(csv)
}

#[tauri::command]
pub fn export_instances_json() -> Result<String, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id FROM task_instances
             ORDER BY due_date DESC, id DESC",
        )
        .map_err(|e| e.to_string())?;

    let ids: Vec<i64> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut data = Vec::new();
    for id in ids {
        let detail = instance_detail(&conn, id).map_err(|e| e.to_string())?;
        let assignees = detail
            .assignees
            .iter()
            .map(|m| json!(m.name))
            .collect::<Vec<_>>();

        data.push(json!({
            "task_title": detail.task.title,
            "cycle_type": detail.task.cycle_type,
            "due_date": detail.instance.due_date.format("%Y-%m-%d").to_string(),
            "status": detail.instance.status,
            "assignees": assignees,
            "confirmed_at": detail.instance.confirmed_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }));
    }

    serde_json::to_string_pretty(&data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_member_stats_csv() -> Result<String, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT ti.id FROM task_instances ti
             JOIN task_assignees ta ON ta.task_id = ti.task_id
             WHERE ti.status = 'completed'
             ORDER BY ti.due_date DESC",
        )
        .map_err(|e| e.to_string())?;

    let ids: Vec<i64> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut stats: HashMap<String, (i64, Vec<String>)> = HashMap::new();
    for id in ids {
        let detail = instance_detail(&conn, id).map_err(|e| e.to_string())?;
        let due = detail.instance.due_date.format("%Y-%m-%d").to_string();
        for m in &detail.assignees {
            let entry = stats.entry(m.name.clone()).or_insert((0, Vec::new()));
            entry.0 += 1;
            entry.1.push(due.clone());
        }
    }

    let mut csv = String::from("\u{FEFF}");
    csv.push_str("员工姓名,已完成次数,完成日期列表\n");

    let mut names: Vec<_> = stats.keys().collect();
    names.sort();
    for name in names {
        let (count, dates) = stats.get(name).unwrap();
        let dates_str = dates.join("、");
        csv.push_str(&format!(
            "{},{},{}\n",
            escape_csv(name),
            count,
            escape_csv(&dates_str)
        ));
    }

    Ok(csv)
}

#[tauri::command]
pub fn get_db_path() -> Result<String, String> {
    current_db_path()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or("数据库路径未初始化".to_string())
}

#[tauri::command]
pub fn set_db_path_command(path: String, migrate: bool) -> Result<String, String> {
    let new_path = std::path::PathBuf::from(path);
    set_db_path(new_path, migrate)?;
    get_db_path()
}

#[tauri::command]
pub fn export_archives_csv() -> Result<String, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut csv = String::from("\u{FEFF}");
    csv.push_str("档案编号,档案名称,分类,存放位置,保管人,状态,数量,创建时间\n");

    let mut stmt = conn
        .prepare(
            "SELECT a.code, a.title, ac.name, a.location, m.name, a.status, a.quantity, a.created_at
             FROM archives a
             LEFT JOIN archive_categories ac ON ac.id = a.category_id
             LEFT JOIN members m ON m.id = a.keeper_id
             ORDER BY a.created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, i32>(6)?,
                row.get::<_, NaiveDateTime>(7)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    for row in rows {
        let (code, title, category, location, keeper, status, quantity, created_at) =
            row.map_err(|e| e.to_string())?;

        let status_label = match status.as_str() {
            "in_stock" => "在库",
            "borrowed" => "借出",
            "damaged" => "损坏",
            "destroyed" => "销毁",
            _ => &status,
        };

        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            escape_csv(&code),
            escape_csv(&title),
            escape_csv(&category.unwrap_or_default()),
            escape_csv(&location.unwrap_or_default()),
            escape_csv(&keeper.unwrap_or_default()),
            status_label,
            quantity,
            created_at.format("%Y-%m-%d %H:%M"),
        ));
    }

    Ok(csv)
}

#[tauri::command]
pub fn export_archive_borrows_csv() -> Result<String, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut csv = String::from("\u{FEFF}");
    csv.push_str("档案编号,档案名称,借阅人,借阅日期,应还日期,归还日期,状态,审批人,备注\n");

    let mut stmt = conn
        .prepare(
            "SELECT a.code, a.title, mb.name, ab.borrow_date, ab.due_date, ab.return_date,
                    ab.status, ma.name, ab.note
             FROM archive_borrows ab
             INNER JOIN archives a ON a.id = ab.archive_id
             INNER JOIN members mb ON mb.id = ab.borrower_id
             LEFT JOIN members ma ON ma.id = ab.approver_id
             ORDER BY ab.created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, NaiveDate>(3)?,
                row.get::<_, NaiveDate>(4)?,
                row.get::<_, Option<NaiveDate>>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    for row in rows {
        let (code, title, borrower, borrow_date, due_date, return_date, status, approver, note) =
            row.map_err(|e| e.to_string())?;

        let status_label = match status.as_str() {
            "borrowed" => "借阅中",
            "returned" => "已归还",
            "overdue" => "逾期",
            _ => &status,
        };

        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            escape_csv(&code),
            escape_csv(&title),
            escape_csv(&borrower),
            borrow_date.format("%Y-%m-%d"),
            due_date.format("%Y-%m-%d"),
            return_date.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default(),
            status_label,
            escape_csv(&approver.unwrap_or_default()),
            escape_csv(&note.unwrap_or_default()),
        ));
    }

    Ok(csv)
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
