use crate::db::{current_db_path, db, models::*};
use chrono::{Datelike, Local, NaiveDate};
use rusqlite::Result;
use std::fs;
use std::path::{Path, PathBuf};

const UNKNOWN_CATEGORY_ID: i64 = 999_999;

fn archive_storage_root() -> Result<PathBuf, String> {
    let db_path = current_db_path().ok_or("数据库路径未初始化")?;
    let root = db_path
        .parent()
        .ok_or("无法获取数据库所在目录")?
        .join("electronic_archives");
    fs::create_dir_all(&root).map_err(|e| format!("创建电子档案目录失败：{}", e))?;
    Ok(root)
}

fn store_electronic_file(
    box_name: Option<&str>,
    source_path: &str,
) -> Result<String, String> {
    let root = archive_storage_root()?;
    let folder_name = box_name
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("未分类盒");
    let safe_folder = sanitize_filename(folder_name);
    let folder = root.join(&safe_folder);
    fs::create_dir_all(&folder).map_err(|e| format!("创建档案盒目录失败：{}", e))?;

    let src = Path::new(source_path);
    let original_name = src
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("无法获取上传文件名")?;
    let safe_name = sanitize_filename(original_name);

    let mut dest = folder.join(&safe_name);
    let mut counter = 1;
    while dest.exists() {
        let stem = Path::new(&safe_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&safe_name);
        let ext = Path::new(&safe_name)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .unwrap_or_default();
        dest = folder.join(format!("{}_{}{}", stem, counter, ext));
        counter += 1;
    }

    fs::copy(src, &dest).map_err(|e| format!("复制电子档案文件失败：{}", e))?;

    // Return relative path from storage root
    let relative = Path::new(&safe_folder)
        .join(dest.file_name().unwrap_or_default())
        .to_string_lossy()
        .replace("\\", "/");
    Ok(relative)
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

fn resolve_archive_file_path(relative: &str) -> Result<PathBuf, String> {
    let root = archive_storage_root()?;
    Ok(root.join(relative.replace('/', "\\")))
}

fn ensure_not_system_category(conn: &rusqlite::Connection, id: i64) -> Result<(), String> {
    let is_system: i32 = conn
        .query_row(
            "SELECT COALESCE(is_system, 0) FROM archive_categories WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if is_system != 0 || id == UNKNOWN_CATEGORY_ID {
        return Err("系统初始化分类（未知分类）不能修改或删除。".to_string());
    }
    Ok(())
}

// ===== Archive Categories =====

#[tauri::command]
pub fn create_archive_category(req: CreateArchiveCategoryRequest) -> Result<ArchiveCategory, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO archive_categories (name, code_prefix, note) VALUES (?1, ?2, ?3)",
        rusqlite::params![req.name, req.code_prefix, req.note],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    let category = conn
        .query_row(
            "SELECT id, name, code_prefix, note, created_at FROM archive_categories WHERE id = ?1",
            [id],
            |row| {
                Ok(ArchiveCategory {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    code_prefix: row.get(2)?,
                    note: row.get(3)?,
                    created_at: row.get(4)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(category)
}

#[tauri::command]
pub fn update_archive_category(req: UpdateArchiveCategoryRequest) -> Result<ArchiveCategory, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    ensure_not_system_category(&conn, req.id)?;

    conn.execute(
        "UPDATE archive_categories SET name = ?1, code_prefix = ?2, note = ?3 WHERE id = ?4",
        rusqlite::params![req.name, req.code_prefix, req.note, req.id],
    )
    .map_err(|e| e.to_string())?;

    let category = conn
        .query_row(
            "SELECT id, name, code_prefix, note, created_at FROM archive_categories WHERE id = ?1",
            [req.id],
            |row| {
                Ok(ArchiveCategory {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    code_prefix: row.get(2)?,
                    note: row.get(3)?,
                    created_at: row.get(4)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(category)
}

#[tauri::command]
pub fn delete_archive_category(id: i64) -> Result<(), String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    ensure_not_system_category(&conn, id)?;

    let archive_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM archives WHERE category_id = ?1", [id], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    if archive_count > 0 {
        return Err(format!(
            "该分类下仍有 {} 个档案，无法删除。请先将档案移至其他分类。",
            archive_count
        ));
    }

    conn.execute("DELETE FROM archive_categories WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_archive_categories() -> Result<Vec<ArchiveCategory>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, name, code_prefix, note, created_at FROM archive_categories ORDER BY id")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ArchiveCategory {
                id: row.get(0)?,
                name: row.get(1)?,
                code_prefix: row.get(2)?,
                note: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_archive_categories_paged(
    page: i64,
    per_page: i64,
    search: Option<String>,
) -> Result<Paginated<ArchiveCategory>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let pattern = search.as_ref().map(|s| format!("%{}%", s));

    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM archive_categories
             WHERE (?1 IS NULL OR name LIKE ?1 OR code_prefix LIKE ?1 OR note LIKE ?1)",
            [pattern.as_deref()],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let offset = (page - 1).max(0) * per_page;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, code_prefix, note, created_at
             FROM archive_categories
             WHERE (?1 IS NULL OR name LIKE ?1 OR code_prefix LIKE ?1 OR note LIKE ?1)
             ORDER BY id LIMIT ?2 OFFSET ?3",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(rusqlite::params![pattern.as_deref(), per_page, offset], |row| {
            Ok(ArchiveCategory {
                id: row.get(0)?,
                name: row.get(1)?,
                code_prefix: row.get(2)?,
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

// ===== Archive Tags =====

#[tauri::command]
pub fn create_archive_tag(req: CreateArchiveTagRequest) -> Result<ArchiveTag, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO archive_tags (name, parent_id, note) VALUES (?1, ?2, ?3)",
        rusqlite::params![req.name, req.parent_id, req.note],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    let tag = conn
        .query_row(
            "SELECT id, name, parent_id, note, created_at FROM archive_tags WHERE id = ?1",
            [id],
            |row| {
                Ok(ArchiveTag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    parent_id: row.get(2)?,
                    note: row.get(3)?,
                    created_at: row.get(4)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(tag)
}

#[tauri::command]
pub fn update_archive_tag(req: UpdateArchiveTagRequest) -> Result<ArchiveTag, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE archive_tags SET name = ?1, parent_id = ?2, note = ?3 WHERE id = ?4",
        rusqlite::params![req.name, req.parent_id, req.note, req.id],
    )
    .map_err(|e| e.to_string())?;

    let tag = conn
        .query_row(
            "SELECT id, name, parent_id, note, created_at FROM archive_tags WHERE id = ?1",
            [req.id],
            |row| {
                Ok(ArchiveTag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    parent_id: row.get(2)?,
                    note: row.get(3)?,
                    created_at: row.get(4)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(tag)
}

#[tauri::command]
pub fn delete_archive_tag(id: i64) -> Result<(), String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let archive_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM archive_tag_relations WHERE tag_id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let child_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM archive_tags WHERE parent_id = ?1", [id], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    if archive_count > 0 || child_count > 0 {
        let mut reasons = Vec::new();
        if archive_count > 0 {
            reasons.push(format!("{} 个档案已关联此标签", archive_count));
        }
        if child_count > 0 {
            reasons.push(format!("{} 个子标签", child_count));
        }
        return Err(format!(
            "该标签仍被以下数据引用，无法删除：{}。请先在对应页面解除关联。",
            reasons.join("、")
        ));
    }

    conn.execute("DELETE FROM archive_tags WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_archive_tags() -> Result<Vec<ArchiveTag>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, name, parent_id, note, created_at FROM archive_tags ORDER BY name")
        .map_err(|e| e.to_string())?;

    let tags = stmt
        .query_map([], |row| {
            Ok(ArchiveTag {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                note: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    tags.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_archive_tags_paged(
    page: i64,
    per_page: i64,
    search: Option<String>,
) -> Result<Paginated<ArchiveTag>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let pattern = search.as_ref().map(|s| format!("%{}%", s));

    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM archive_tags
             WHERE (?1 IS NULL OR name LIKE ?1 OR note LIKE ?1)",
            [pattern.as_deref()],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let offset = (page - 1).max(0) * per_page;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, parent_id, note, created_at
             FROM archive_tags
             WHERE (?1 IS NULL OR name LIKE ?1 OR note LIKE ?1)
             ORDER BY name LIMIT ?2 OFFSET ?3",
        )
        .map_err(|e| e.to_string())?;

    let tags = stmt
        .query_map(rusqlite::params![pattern.as_deref(), per_page, offset], |row| {
            Ok(ArchiveTag {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                note: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let items = tags.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(Paginated {
        items,
        total,
        page,
        per_page,
    })
}

#[tauri::command]
pub fn list_archives_by_tag(
    tag_id: i64,
    page: i64,
    per_page: i64,
) -> Result<Paginated<ArchiveDetail>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM archive_tag_relations WHERE tag_id = ?1",
            [tag_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let offset = (page - 1).max(0) * per_page;
    let mut stmt = conn
        .prepare(
            "SELECT a.id FROM archives a
             INNER JOIN archive_tag_relations r ON r.archive_id = a.id
             WHERE r.tag_id = ?1
             ORDER BY a.created_at DESC
             LIMIT ?2 OFFSET ?3",
        )
        .map_err(|e| e.to_string())?;

    let ids: Vec<i64> = stmt
        .query_map(rusqlite::params![tag_id, per_page, offset], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for id in ids {
        results.push(archive_detail(&conn, id)?);
    }

    Ok(Paginated {
        items: results,
        total,
        page,
        per_page,
    })
}

// ===== Archives =====

fn generate_archive_code(conn: &rusqlite::Connection, category_id: i64) -> Result<String, String> {
    let (prefix, current_year) = {
        let prefix: String = conn
            .query_row(
                "SELECT code_prefix FROM archive_categories WHERE id = ?1",
                [category_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        (prefix, Local::now().year())
    };

    let pattern = format!("{}-{}-%", prefix, current_year);
    let max_seq: Option<i64> = conn
        .query_row(
            "SELECT MAX(CAST(SUBSTR(code, LENGTH(?1) + 2) AS INTEGER))
             FROM archives WHERE code LIKE ?2",
            rusqlite::params![format!("{}-{}", prefix, current_year), pattern],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let seq = max_seq.unwrap_or(0) + 1;
    Ok(format!("{}-{}-{:03}", prefix, current_year, seq))
}

fn archive_detail(conn: &rusqlite::Connection, id: i64) -> Result<ArchiveDetail, String> {
    let archive = conn
        .query_row(
            "SELECT id, code, title, category_id, location, keeper_id, status, quantity, description, photos,
                    archive_type, box_name, file_path, created_at
             FROM archives WHERE id = ?1",
            [id],
            |row| {
                Ok(Archive {
                    id: row.get(0)?,
                    code: row.get(1)?,
                    title: row.get(2)?,
                    category_id: row.get(3)?,
                    location: row.get(4)?,
                    keeper_id: row.get(5)?,
                    status: row.get(6)?,
                    quantity: row.get(7)?,
                    description: row.get(8)?,
                    photos: row.get(9)?,
                    archive_type: row.get(10)?,
                    box_name: row.get(11)?,
                    file_path: row.get(12)?,
                    created_at: row.get(13)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    let category = conn.query_row(
        "SELECT id, name, code_prefix, note, created_at FROM archive_categories WHERE id = ?1",
        [archive.category_id],
        |row| {
            Ok(ArchiveCategory {
                id: row.get(0)?,
                name: row.get(1)?,
                code_prefix: row.get(2)?,
                note: row.get(3)?,
                created_at: row.get(4)?,
            })
        },
    ).ok();

    let keeper = if let Some(keeper_id) = archive.keeper_id {
        conn.query_row(
            "SELECT id, name, phone, email, note, created_at FROM members WHERE id = ?1",
            [keeper_id],
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
        .ok()
    } else {
        None
    };

    let tags = load_archive_tags(conn, id)?;

    Ok(ArchiveDetail {
        archive,
        category,
        keeper,
        tags,
    })
}

fn load_archive_tags(conn: &rusqlite::Connection, archive_id: i64) -> Result<Vec<ArchiveTag>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name, t.parent_id, t.note, t.created_at
             FROM archive_tags t
             INNER JOIN archive_tag_relations r ON r.tag_id = t.id
             WHERE r.archive_id = ?1
             ORDER BY t.name",
        )
        .map_err(|e| e.to_string())?;

    let tags = stmt
        .query_map([archive_id], |row| {
            Ok(ArchiveTag {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                note: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    tags.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

fn attach_archive_tags(
    tx: &rusqlite::Transaction,
    archive_id: i64,
    tag_ids: &[i64],
) -> Result<(), String> {
    tx.execute(
        "DELETE FROM archive_tag_relations WHERE archive_id = ?1",
        [archive_id],
    )
    .map_err(|e| e.to_string())?;

    let mut stmt = tx
        .prepare("INSERT INTO archive_tag_relations (archive_id, tag_id) VALUES (?1, ?2)")
        .map_err(|e| e.to_string())?;
    for tag_id in tag_ids {
        stmt.execute(rusqlite::params![archive_id, tag_id])
            .map_err(|e| e.to_string())?;
    }
    drop(stmt);
    Ok(())
}

#[tauri::command]
pub fn create_archive(req: CreateArchiveRequest) -> Result<ArchiveDetail, String> {
    let db = db();
    let mut conn = db.lock().map_err(|e| e.to_string())?;

    let code = generate_archive_code(&conn, req.category_id)?;

    let archive_type = req
        .archive_type
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("paper");

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO archives (code, title, category_id, location, keeper_id, status, quantity, description, photos,
                               archive_type, box_name, file_path)
         VALUES (?1, ?2, ?3, ?4, ?5, 'in_stock', ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            code,
            req.title,
            req.category_id,
            req.location,
            req.keeper_id,
            req.quantity,
            req.description,
            req.photos,
            archive_type,
            req.box_name,
            req.file_path,
        ],
    )
    .map_err(|e| e.to_string())?;

    let id = tx.last_insert_rowid();
    attach_archive_tags(&tx, id, &req.tag_ids)?;

    if let Some(source) = req.source_file_path.as_deref().filter(|s| !s.is_empty()) {
        if req.box_name.as_deref().map(|s| s.trim()).unwrap_or("").is_empty() {
            return Err("上传电子文件前必须先填写档案盒名称。".to_string());
        }
        let relative = store_electronic_file(req.box_name.as_deref(), source)?;
        tx.execute(
            "UPDATE archives SET file_path = ?1 WHERE id = ?2",
            rusqlite::params![&relative, id],
        )
        .map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;

    let detail = archive_detail(&conn, id)?;
    Ok(detail)
}

#[tauri::command]
pub fn update_archive(req: UpdateArchiveRequest) -> Result<ArchiveDetail, String> {
    let db = db();
    let mut conn = db.lock().map_err(|e| e.to_string())?;

    let archive_type = req
        .archive_type
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("paper");

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "UPDATE archives SET title = ?1, category_id = ?2, location = ?3, keeper_id = ?4,
                            quantity = ?5, description = ?6, photos = ?7,
                            archive_type = ?8, box_name = ?9, file_path = ?10
         WHERE id = ?11",
        rusqlite::params![
            req.title,
            req.category_id,
            req.location,
            req.keeper_id,
            req.quantity,
            req.description,
            req.photos,
            archive_type,
            req.box_name,
            req.file_path,
            req.id,
        ],
    )
    .map_err(|e| e.to_string())?;

    attach_archive_tags(&tx, req.id, &req.tag_ids)?;

    if let Some(source) = req.source_file_path.as_deref().filter(|s| !s.is_empty()) {
        if req.box_name.as_deref().map(|s| s.trim()).unwrap_or("").is_empty() {
            return Err("上传电子文件前必须先填写档案盒名称。".to_string());
        }
        let relative = store_electronic_file(req.box_name.as_deref(), source)?;
        tx.execute(
            "UPDATE archives SET file_path = ?1 WHERE id = ?2",
            rusqlite::params![&relative, req.id],
        )
        .map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;

    let detail = archive_detail(&conn, req.id)?;
    Ok(detail)
}

#[tauri::command]
pub fn delete_archive(id: i64) -> Result<(), String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let active_borrow_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM archive_borrows WHERE archive_id = ?1 AND status != 'returned'",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if active_borrow_count > 0 {
        return Err(format!(
            "该档案仍有 {} 条未归还的借出记录，无法删除。请先归还后再试。",
            active_borrow_count
        ));
    }

    let file_path: Option<String> = conn
        .query_row("SELECT file_path FROM archives WHERE id = ?1", [id], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM archive_tag_relations WHERE archive_id = ?1", [id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM archive_borrows WHERE archive_id = ?1", [id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM archives WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;

    if let Some(relative) = file_path {
        if let Ok(path) = resolve_archive_file_path(&relative) {
            let _ = fs::remove_file(&path);
        }
    }

    Ok(())
}

#[tauri::command]
pub fn list_archives(
    category_id: Option<i64>,
    status: Option<String>,
    search: Option<String>,
    page: i64,
    per_page: i64,
) -> Result<Paginated<ArchiveDetail>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut where_sql = String::from("WHERE 1=1");
    let mut filter_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(cid) = category_id {
        where_sql.push_str(" AND category_id = ?");
        filter_params.push(Box::new(cid));
    }
    if let Some(s) = status {
        where_sql.push_str(" AND status = ?");
        filter_params.push(Box::new(s));
    }
    if let Some(search) = search {
        where_sql.push_str(" AND (code LIKE ? OR title LIKE ? OR location LIKE ?)");
        let pattern = format!("%{}%", search);
        filter_params.push(Box::new(pattern.clone()));
        filter_params.push(Box::new(pattern.clone()));
        filter_params.push(Box::new(pattern));
    }

    let total: i64 = {
        let sql = format!("SELECT COUNT(*) FROM archives {}", where_sql);
        let refs: Vec<&dyn rusqlite::ToSql> = filter_params.iter().map(|p| p.as_ref()).collect();
        conn.query_row(&sql, refs.as_slice(), |row| row.get(0))
            .map_err(|e| e.to_string())?
    };

    let offset = (page - 1).max(0) * per_page;
    let sql = format!(
        "SELECT id FROM archives {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        where_sql
    );
    let mut params = filter_params;
    params.push(Box::new(per_page));
    params.push(Box::new(offset));
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(param_refs.as_slice(), |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let ids: Vec<i64> = rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for id in ids {
        results.push(archive_detail(&conn, id)?);
    }

    Ok(Paginated {
        items: results,
        total,
        page,
        per_page,
    })
}

#[tauri::command]
pub fn get_archive(id: i64) -> Result<ArchiveDetail, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;
    archive_detail(&conn, id)
}

#[tauri::command]
pub fn update_archive_status(id: i64, status: String) -> Result<ArchiveDetail, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE archives SET status = ?1 WHERE id = ?2", rusqlite::params![status, id])
        .map_err(|e| e.to_string())?;
    archive_detail(&conn, id)
}

#[tauri::command]
pub fn get_archive_file_path(id: i64) -> Result<String, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let relative: Option<String> = conn
        .query_row("SELECT file_path FROM archives WHERE id = ?1", [id], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let relative = relative.ok_or("该档案没有关联的电子文件")?;
    let path = resolve_archive_file_path(&relative)?;
    if !path.exists() {
        return Err("电子文件不存在，可能已被移动或删除".to_string());
    }
    Ok(path.to_string_lossy().to_string())
}

// ===== Archive Borrows =====

fn borrow_detail(conn: &rusqlite::Connection, id: i64) -> Result<ArchiveBorrowDetail, String> {
    let borrow = conn
        .query_row(
            "SELECT id, archive_id, borrower_id, purpose, borrow_date, due_date, return_date,
                    status, approver_id, note, reminded, created_at
             FROM archive_borrows WHERE id = ?1",
            [id],
            |row| {
                Ok(ArchiveBorrow {
                    id: row.get(0)?,
                    archive_id: row.get(1)?,
                    borrower_id: row.get(2)?,
                    purpose: row.get(3)?,
                    borrow_date: row.get(4)?,
                    due_date: row.get(5)?,
                    return_date: row.get(6)?,
                    status: row.get(7)?,
                    approver_id: row.get(8)?,
                    note: row.get(9)?,
                    reminded: row.get::<_, i32>(10)? != 0,
                    created_at: row.get(11)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    let archive = archive_detail(conn, borrow.archive_id)?;

    let borrower = conn
        .query_row(
            "SELECT id, name, phone, email, note, created_at FROM members WHERE id = ?1",
            [borrow.borrower_id],
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

    let approver = if let Some(aid) = borrow.approver_id {
        conn.query_row(
            "SELECT id, name, phone, email, note, created_at FROM members WHERE id = ?1",
            [aid],
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
        .ok()
    } else {
        None
    };

    Ok(ArchiveBorrowDetail {
        borrow,
        archive,
        borrower,
        approver,
    })
}

#[tauri::command]
pub fn create_archive_borrow(req: CreateArchiveBorrowRequest) -> Result<ArchiveBorrowDetail, String> {
    let db = db();
    let mut conn = db.lock().map_err(|e| e.to_string())?;

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO archive_borrows (archive_id, borrower_id, purpose, borrow_date, due_date, approver_id, note)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            req.archive_id,
            req.borrower_id,
            req.purpose,
            req.borrow_date,
            req.due_date,
            req.approver_id,
            req.note,
        ],
    )
    .map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE archives SET status = 'borrowed' WHERE id = ?1 AND status = 'in_stock'",
        [req.archive_id],
    )
    .map_err(|e| e.to_string())?;

    let id = tx.last_insert_rowid();
    tx.commit().map_err(|e| e.to_string())?;

    let detail = borrow_detail(&conn, id)?;
    Ok(detail)
}

#[tauri::command]
pub fn return_archive_borrow(id: i64, return_date: NaiveDate) -> Result<ArchiveBorrowDetail, String> {
    let db = db();
    let mut conn = db.lock().map_err(|e| e.to_string())?;

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE archive_borrows SET status = 'returned', return_date = ?1 WHERE id = ?2",
        rusqlite::params![return_date, id],
    )
    .map_err(|e| e.to_string())?;

    let archive_id: i64 = tx
        .query_row(
            "SELECT archive_id FROM archive_borrows WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let active_count: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM archive_borrows WHERE archive_id = ?1 AND status = 'borrowed'",
            [archive_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if active_count == 0 {
        tx.execute(
            "UPDATE archives SET status = 'in_stock' WHERE id = ?1",
            [archive_id],
        )
        .map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;

    let detail = borrow_detail(&conn, id)?;
    Ok(detail)
}

#[tauri::command]
pub fn update_archive_borrow(req: UpdateArchiveBorrowRequest) -> Result<ArchiveBorrowDetail, String> {
    let db = db();
    let mut conn = db.lock().map_err(|e| e.to_string())?;

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE archive_borrows
         SET borrower_id = ?1, purpose = ?2, borrow_date = ?3, due_date = ?4,
             return_date = ?5, status = ?6, approver_id = ?7, note = ?8
         WHERE id = ?9",
        rusqlite::params![
            req.borrower_id,
            req.purpose,
            req.borrow_date,
            req.due_date,
            req.return_date,
            req.status,
            req.approver_id,
            req.note,
            req.id,
        ],
    )
    .map_err(|e| e.to_string())?;

    let archive_id: i64 = tx
        .query_row(
            "SELECT archive_id FROM archive_borrows WHERE id = ?1",
            [req.id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let active_count: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM archive_borrows WHERE archive_id = ?1 AND status != 'returned'",
            [archive_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let archive_status = if active_count > 0 { "borrowed" } else { "in_stock" };
    tx.execute(
        "UPDATE archives SET status = ?1 WHERE id = ?2",
        rusqlite::params![archive_status, archive_id],
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    let detail = borrow_detail(&conn, req.id)?;
    Ok(detail)
}

#[tauri::command]
pub fn delete_archive_borrow(id: i64) -> Result<(), String> {
    let db = db();
    let mut conn = db.lock().map_err(|e| e.to_string())?;

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let status: String = tx
        .query_row(
            "SELECT status FROM archive_borrows WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if status != "returned" {
        return Err("只能删除已归还的借还记录，请先归还该档案。".to_string());
    }

    let archive_id: i64 = tx
        .query_row(
            "SELECT archive_id FROM archive_borrows WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    tx.execute("DELETE FROM archive_borrows WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;

    let active_count: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM archive_borrows WHERE archive_id = ?1 AND status != 'returned'",
            [archive_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if active_count == 0 {
        tx.execute(
            "UPDATE archives SET status = 'in_stock' WHERE id = ?1",
            [archive_id],
        )
        .map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_archive_borrows(
    status: Option<String>,
    archive_id: Option<i64>,
    borrower_id: Option<i64>,
    page: i64,
    per_page: i64,
) -> Result<Paginated<ArchiveBorrowDetail>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut where_sql = String::from("WHERE 1=1");
    let mut filter_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(s) = status {
        where_sql.push_str(" AND status = ?");
        filter_params.push(Box::new(s));
    }
    if let Some(aid) = archive_id {
        where_sql.push_str(" AND archive_id = ?");
        filter_params.push(Box::new(aid));
    }
    if let Some(bid) = borrower_id {
        where_sql.push_str(" AND borrower_id = ?");
        filter_params.push(Box::new(bid));
    }

    let total: i64 = {
        let sql = format!("SELECT COUNT(*) FROM archive_borrows {}", where_sql);
        let refs: Vec<&dyn rusqlite::ToSql> = filter_params.iter().map(|p| p.as_ref()).collect();
        conn.query_row(&sql, refs.as_slice(), |row| row.get(0))
            .map_err(|e| e.to_string())?
    };

    let offset = (page - 1).max(0) * per_page;
    let sql = format!(
        "SELECT id FROM archive_borrows {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        where_sql
    );
    let mut params = filter_params;
    params.push(Box::new(per_page));
    params.push(Box::new(offset));
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(param_refs.as_slice(), |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let ids: Vec<i64> = rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for id in ids {
        results.push(borrow_detail(&conn, id)?);
    }

    Ok(Paginated {
        items: results,
        total,
        page,
        per_page,
    })
}

#[tauri::command]
pub fn get_archive_borrow(id: i64) -> Result<ArchiveBorrowDetail, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;
    borrow_detail(&conn, id)
}

#[tauri::command]
pub fn list_active_archive_borrows(
    page: i64,
    per_page: i64,
) -> Result<Paginated<ArchiveBorrowDetail>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM archive_borrows WHERE status != 'returned'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let offset = (page - 1).max(0) * per_page;
    let mut stmt = conn
        .prepare(
            "SELECT id FROM archive_borrows
             WHERE status != 'returned'
             ORDER BY created_at DESC
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
        results.push(borrow_detail(&conn, id)?);
    }

    Ok(Paginated {
        items: results,
        total,
        page,
        per_page,
    })
}

#[tauri::command]
pub fn get_archive_stats() -> Result<ArchiveStats, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let today = Local::now().date_naive();

    let total_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM archives", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let in_stock_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM archives WHERE status = 'in_stock'", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let borrowed_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM archives WHERE status = 'borrowed'", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let damaged_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM archives WHERE status = 'damaged'", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let destroyed_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM archives WHERE status = 'destroyed'", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE archive_borrows SET status = 'overdue'
         WHERE status = 'borrowed' AND due_date < ?1",
        [today],
    )
    .map_err(|e| e.to_string())?;

    let overdue_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM archive_borrows WHERE status = 'overdue'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(ArchiveStats {
        total_count,
        in_stock_count,
        borrowed_count,
        overdue_count,
        damaged_count,
        destroyed_count,
    })
}

fn ensure_unknown_defaults(tx: &mut rusqlite::Transaction) -> Result<(i64, i64), String> {

    let category_id: i64 = tx
        .query_row(
            "SELECT id FROM archive_categories WHERE name = ?1",
            ["未知分类"],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| {
            tx.execute(
                "INSERT INTO archive_categories (name, code_prefix, note, is_system) VALUES (?1, ?2, ?3, 1)",
                ["未知分类", "UNKNOWN", "系统默认分类，导入时未指定分类的档案会归到这里"],
            )
            .expect("Failed to create unknown category");
            tx.last_insert_rowid()
        });

    let keeper_id: i64 = tx
        .query_row(
            "SELECT id FROM members WHERE name = ?1",
            ["未知保管人"],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| {
            tx.execute(
                "INSERT INTO members (name, phone, email, note, is_system) VALUES (?1, NULL, NULL, ?2, 1)",
                ["未知保管人", "系统默认人员，导入时未指定保管人的档案会归到这里"],
            )
            .expect("Failed to create unknown keeper");
            tx.last_insert_rowid()
        });

    Ok((category_id, keeper_id))
}

#[tauri::command]
pub fn import_archives_from_excel(path: String) -> Result<(usize, usize), String> {
    use calamine::{open_workbook, DataType, Reader, Xlsx};

    let mut workbook: Xlsx<_> = open_workbook(&path).map_err(|e| format!("无法打开 Excel 文件：{}", e))?;
    let range = workbook
        .worksheet_range("Sheet1")
        .map_err(|e| format!("读取 Sheet1 失败：{}", e))?;

    if range.height() < 2 {
        return Err("Excel 数据行数不足，至少需要表头和一行数据".to_string());
    }

    let header: Vec<String> = range
        .rows()
        .next()
        .unwrap_or(&[])
        .iter()
        .map(|c| c.as_string().unwrap_or_default().trim().to_string())
        .collect();
    if header.len() < 3
        || header[0] != "具体材料"
        || header[1] != "档案盒名称"
        || header[2] != "标签"
    {
        return Err(format!(
            "表头格式不符合规范，前三列必须是：具体材料、档案盒名称、标签，当前为：{:?}",
            header
        ));
    }

    let db = db();
    let mut conn = db.lock().map_err(|e| e.to_string())?;
    let mut tx = conn.transaction().map_err(|e| e.to_string())?;

    let (unknown_category_id, unknown_keeper_id) = ensure_unknown_defaults(&mut tx)?;

    let mut archive_count = 0usize;
    let mut tag_count = 0usize;

    use std::collections::HashMap;
    use std::collections::HashSet;

    // material -> (box_name, tags)
    let mut materials: HashMap<String, (String, Vec<String>)> = HashMap::new();
    let mut tag_names: HashSet<String> = HashSet::new();

    for row in range.rows().skip(1) {
        let material: String = row
            .get(0)
            .map(|c| c.as_string().unwrap_or_default().trim().to_string())
            .unwrap_or_default();
        let box_name: String = row
            .get(1)
            .map(|c| c.as_string().unwrap_or_default().trim().to_string())
            .unwrap_or_default();
        let tag_name: String = row
            .get(2)
            .map(|c| c.as_string().unwrap_or_default().trim().to_string())
            .unwrap_or_default();

        if tag_name.is_empty() && material.is_empty() {
            continue;
        }

        if !tag_name.is_empty() {
            tag_names.insert(tag_name.clone());
        }

        if !material.is_empty() {
            let entry = materials
                .entry(material)
                .or_insert_with(|| (box_name.clone(), Vec::new()));
            if entry.0.is_empty() && !box_name.is_empty() {
                entry.0 = box_name;
            }
            if !tag_name.is_empty() && !entry.1.contains(&tag_name) {
                entry.1.push(tag_name);
            }
        }
    }

    let mut tag_id_map: HashMap<String, i64> = HashMap::new();
    for tag_name in tag_names {
        let existing: Option<i64> = tx
            .query_row(
                "SELECT id FROM archive_tags WHERE name = ?1",
                [&tag_name],
                |row| row.get(0),
            )
            .ok();
        let tag_id = match existing {
            Some(id) => id,
            None => {
                tx.execute(
                    "INSERT INTO archive_tags (name, parent_id, note) VALUES (?1, NULL, NULL)",
                    [&tag_name],
                )
                .map_err(|e| e.to_string())?;
                tag_count += 1;
                tx.last_insert_rowid()
            }
        };
        tag_id_map.insert(tag_name, tag_id);
    }

    for (material, (box_name, tags)) in materials {
        let code = generate_archive_code(&tx, unknown_category_id)?;
        tx.execute(
            "INSERT INTO archives (code, title, category_id, location, keeper_id, status, quantity, description, photos,
                                   archive_type, box_name, file_path)
             VALUES (?1, ?2, ?3, ?4, ?5, 'in_stock', 1, NULL, NULL, 'paper', ?6, NULL)",
            rusqlite::params![code, material, unknown_category_id, box_name, unknown_keeper_id, box_name],
        )
        .map_err(|e| e.to_string())?;
        let archive_id = tx.last_insert_rowid();

        for tag_name in tags {
            if let Some(&tid) = tag_id_map.get(&tag_name) {
                tx.execute(
                    "INSERT OR IGNORE INTO archive_tag_relations (archive_id, tag_id) VALUES (?1, ?2)",
                    rusqlite::params![archive_id, tid],
                )
                .map_err(|e| e.to_string())?;
            }
        }
        archive_count += 1;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok((archive_count, tag_count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::members::create_member;
    use crate::db::init_db;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn list_archives_returns_nested_archive_detail() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let dir = std::env::temp_dir().join(format!("task_reminder_test_{}", ts));
        std::fs::create_dir_all(&dir).unwrap();
        init_db(dir).unwrap();

        let member = create_member(CreateMemberRequest {
            name: "测试员工".to_string(),
            phone: None,
            email: None,
            note: None,
        })
        .unwrap();

        let detail = create_archive(CreateArchiveRequest {
            title: "测试档案".to_string(),
            category_id: 1,
            location: Some("A柜".to_string()),
            keeper_id: Some(member.id),
            quantity: 1,
            description: None,
            photos: None,
            archive_type: Some("paper".to_string()),
            box_name: None,
            file_path: None,
            source_file_path: None,
            tag_ids: vec![],
        })
        .unwrap();

        assert!(detail.archive.code.starts_with("YZ"));
        assert_eq!(detail.archive.title, "测试档案");

        let list = list_archives(None, None, None, 1, 10).unwrap();
        assert_eq!(list.items.len(), 1);
        assert_eq!(list.items[0].archive.id, detail.archive.id);
        assert_eq!(list.total, 1);

        // Make sure serialization keeps a nested "archive" object,
        // which the Vue frontend expects.
        let json = serde_json::to_value(&list.items[0]).unwrap();
        assert!(json.get("archive").is_some());
        assert_eq!(json["archive"]["title"], "测试档案");
    }
}
