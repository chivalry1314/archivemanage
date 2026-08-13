use crate::db::{db, models::*};
use rusqlite::Result;

const UNKNOWN_MEMBER_ID: i64 = 999_999;

fn ensure_not_system_member(conn: &rusqlite::Connection, id: i64) -> Result<(), String> {
    let is_system: i32 = conn
        .query_row(
            "SELECT COALESCE(is_system, 0) FROM members WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if is_system != 0 || id == UNKNOWN_MEMBER_ID {
        return Err("系统初始化人员（未知保管人）不能修改或删除。".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn create_member(req: CreateMemberRequest) -> Result<Member, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO members (name, phone, email, note) VALUES (?1, ?2, ?3, ?4)",
        (&req.name, &req.phone, &req.email, &req.note),
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    let member = conn
        .query_row(
            "SELECT id, name, phone, email, note, created_at FROM members WHERE id = ?1",
            [id],
            |row| {
                Ok(Member {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    phone: row.get(2)?,
                    email: row.get(3)?,
                    note: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(member)
}

#[tauri::command]
pub fn update_member(req: UpdateMemberRequest) -> Result<Member, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    ensure_not_system_member(&conn, req.id)?;

    conn.execute(
        "UPDATE members SET name = ?1, phone = ?2, email = ?3, note = ?4 WHERE id = ?5",
        (&req.name, &req.phone, &req.email, &req.note, req.id),
    )
    .map_err(|e| e.to_string())?;

    let member = conn
        .query_row(
            "SELECT id, name, phone, email, note, created_at FROM members WHERE id = ?1",
            [req.id],
            |row| {
                Ok(Member {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    phone: row.get(2)?,
                    email: row.get(3)?,
                    note: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(member)
}

#[tauri::command]
pub fn delete_member(id: i64) -> Result<(), String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    ensure_not_system_member(&conn, id)?;

    let keeper_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM archives WHERE keeper_id = ?1", [id], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let borrower_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM archive_borrows WHERE borrower_id = ?1", [id], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let approver_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM archive_borrows WHERE approver_id = ?1", [id], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let task_assignee_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM task_assignees WHERE member_id = ?1", [id], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    if keeper_count > 0 || borrower_count > 0 || approver_count > 0 || task_assignee_count > 0 {
        let mut reasons = Vec::new();
        if keeper_count > 0 {
            reasons.push(format!("{} 个档案的保管人", keeper_count));
        }
        if borrower_count > 0 {
            reasons.push(format!("{} 条借还记录的借阅人", borrower_count));
        }
        if approver_count > 0 {
            reasons.push(format!("{} 条借还记录的审批人", approver_count));
        }
        if task_assignee_count > 0 {
            reasons.push(format!("{} 个任务的被指派人", task_assignee_count));
        }
        return Err(format!(
            "该人员仍被以下数据引用，无法删除：{}。请先在对应页面解除关联。",
            reasons.join("、")
        ));
    }

    conn.execute("DELETE FROM members WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn list_members() -> Result<Vec<Member>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, name, phone, email, note, created_at FROM members ORDER BY name")
        .map_err(|e| e.to_string())?;

    let members = stmt
        .query_map([], |row| {
            Ok(Member {
                id: row.get(0)?,
                name: row.get(1)?,
                phone: row.get(2)?,
                email: row.get(3)?,
                note: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    members.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_members_paged(
    page: i64,
    per_page: i64,
    search: Option<String>,
) -> Result<Paginated<Member>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let pattern = search.as_ref().map(|s| format!("%{}%", s));

    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM members
             WHERE (?1 IS NULL OR name LIKE ?1 OR phone LIKE ?1 OR email LIKE ?1 OR note LIKE ?1)",
            [pattern.as_deref()],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let offset = (page - 1).max(0) * per_page;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, phone, email, note, created_at
             FROM members
             WHERE (?1 IS NULL OR name LIKE ?1 OR phone LIKE ?1 OR email LIKE ?1 OR note LIKE ?1)
             ORDER BY name LIMIT ?2 OFFSET ?3",
        )
        .map_err(|e| e.to_string())?;

    let members = stmt
        .query_map(rusqlite::params![pattern.as_deref(), per_page, offset], |row| {
            Ok(Member {
                id: row.get(0)?,
                name: row.get(1)?,
                phone: row.get(2)?,
                email: row.get(3)?,
                note: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let items = members.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(Paginated {
        items,
        total,
        page,
        per_page,
    })
}
