use crate::db::{db, models::*};

#[tauri::command]
pub fn create_archive_box(req: CreateArchiveBoxRequest) -> Result<ArchiveBox, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let name = req.name.trim();
    if name.is_empty() {
        return Err("档案盒名称不能为空".to_string());
    }

    conn.execute(
        "INSERT INTO archive_boxes (name, location, note) VALUES (?1, ?2, ?3)",
        rusqlite::params![name, req.location, req.note],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    let box_record = conn
        .query_row(
            "SELECT id, name, location, note, created_at FROM archive_boxes WHERE id = ?1",
            [id],
            |row| {
                Ok(ArchiveBox {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    location: row.get(2)?,
                    note: row.get(3)?,
                    created_at: row.get(4)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(box_record)
}

#[tauri::command]
pub fn update_archive_box(req: UpdateArchiveBoxRequest) -> Result<ArchiveBox, String> {
    let db = db();
    let mut conn = db.lock().map_err(|e| e.to_string())?;

    let name = req.name.trim();
    if name.is_empty() {
        return Err("档案盒名称不能为空".to_string());
    }

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE archive_boxes SET name = ?1, location = ?2, note = ?3 WHERE id = ?4",
        rusqlite::params![name, req.location, req.note, req.id],
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    let box_record = conn
        .query_row(
            "SELECT id, name, location, note, created_at FROM archive_boxes WHERE id = ?1",
            [req.id],
            |row| {
                Ok(ArchiveBox {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    location: row.get(2)?,
                    note: row.get(3)?,
                    created_at: row.get(4)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(box_record)
}

#[tauri::command]
pub fn delete_archive_box(id: i64) -> Result<(), String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let archive_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM archives WHERE archive_box_id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if archive_count > 0 {
        return Err(format!(
            "该档案盒下仍有 {} 个档案，无法删除。请先将档案移至其他档案盒。",
            archive_count
        ));
    }

    conn.execute("DELETE FROM archive_boxes WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_archive_boxes() -> Result<Vec<ArchiveBox>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, name, location, note, created_at FROM archive_boxes ORDER BY name")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ArchiveBox {
                id: row.get(0)?,
                name: row.get(1)?,
                location: row.get(2)?,
                note: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_archive_boxes_paged(
    page: i64,
    per_page: i64,
    search: Option<String>,
) -> Result<Paginated<ArchiveBox>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let pattern = search.as_ref().map(|s| format!("%{}%", s));

    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM archive_boxes
             WHERE (?1 IS NULL OR name LIKE ?1 OR location LIKE ?1 OR note LIKE ?1)",
            [pattern.as_deref()],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let offset = (page - 1).max(0) * per_page;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, location, note, created_at
             FROM archive_boxes
             WHERE (?1 IS NULL OR name LIKE ?1 OR location LIKE ?1 OR note LIKE ?1)
             ORDER BY name LIMIT ?2 OFFSET ?3",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![pattern.as_deref(), per_page, offset], |row| {
            Ok(ArchiveBox {
                id: row.get(0)?,
                name: row.get(1)?,
                location: row.get(2)?,
                note: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let items = rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(Paginated {
        items,
        total,
        page,
        per_page,
    })
}

#[tauri::command]
pub fn get_archive_box(id: i64) -> Result<ArchiveBox, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let box_record = conn
        .query_row(
            "SELECT id, name, location, note, created_at FROM archive_boxes WHERE id = ?1",
            [id],
            |row| {
                Ok(ArchiveBox {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    location: row.get(2)?,
                    note: row.get(3)?,
                    created_at: row.get(4)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(box_record)
}
