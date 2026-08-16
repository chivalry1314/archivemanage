use crate::db::{db, models::*};
use chrono::NaiveDate;
use rusqlite::Result;
use std::collections::HashMap;

const CONTRACT_HEADERS: [&str; 19] = [
    "序号",
    "合同编号",
    "合同名称",
    "合同甲方",
    "合同乙方",
    "对方联系人",
    "联系方式",
    "合同总金额（含税）",
    "合同总金额（不含税）",
    "税额",
    "付款周期",
    "每次支付金额（含税）",
    "付款方式",
    "合同生效日期",
    "合同终止日期",
    "合同签订日期",
    "甲方经办人",
    "乙方经办人",
    "备注",
];

fn contract_from_row(row: &rusqlite::Row) -> Result<Contract> {
    Ok(Contract {
        id: row.get(0)?,
        contract_no: row.get(1)?,
        contract_name: row.get(2)?,
        party_a: row.get(3)?,
        party_b: row.get(4)?,
        contact_person: row.get(5)?,
        contact_info: row.get(6)?,
        total_amount_with_tax: row.get(7)?,
        total_amount_without_tax: row.get(8)?,
        tax_amount: row.get(9)?,
        payment_cycle: row.get(10)?,
        payment_amount_with_tax: row.get(11)?,
        payment_method: row.get(12)?,
        effective_date: row.get(13)?,
        end_date: row.get(14)?,
        sign_date: row.get(15)?,
        handler_party_a: row.get(16)?,
        handler_party_b: row.get(17)?,
        remark: row.get(18)?,
        created_at: row.get(19)?,
    })
}

fn contract_by_id(conn: &rusqlite::Connection, id: i64) -> Result<Contract, String> {
    conn.query_row(
        "SELECT id, contract_no, contract_name, party_a, party_b, contact_person, contact_info,
                total_amount_with_tax, total_amount_without_tax, tax_amount, payment_cycle,
                payment_amount_with_tax, payment_method, effective_date, end_date, sign_date,
                handler_party_a, handler_party_b, remark, created_at
         FROM contracts WHERE id = ?1",
        [id],
        contract_from_row,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_contract(req: CreateContractRequest) -> Result<Contract, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO contracts (
            contract_no, contract_name, party_a, party_b, contact_person, contact_info,
            total_amount_with_tax, total_amount_without_tax, tax_amount, payment_cycle,
            payment_amount_with_tax, payment_method, effective_date, end_date, sign_date,
            handler_party_a, handler_party_b, remark
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
        rusqlite::params![
            req.contract_no,
            req.contract_name,
            req.party_a,
            req.party_b,
            req.contact_person,
            req.contact_info,
            req.total_amount_with_tax,
            req.total_amount_without_tax,
            req.tax_amount,
            req.payment_cycle,
            req.payment_amount_with_tax,
            req.payment_method,
            req.effective_date,
            req.end_date,
            req.sign_date,
            req.handler_party_a,
            req.handler_party_b,
            req.remark,
        ],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    contract_by_id(&conn, id)
}

#[tauri::command]
pub fn update_contract(req: UpdateContractRequest) -> Result<Contract, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE contracts SET
            contract_no = ?1, contract_name = ?2, party_a = ?3, party_b = ?4,
            contact_person = ?5, contact_info = ?6, total_amount_with_tax = ?7,
            total_amount_without_tax = ?8, tax_amount = ?9, payment_cycle = ?10,
            payment_amount_with_tax = ?11, payment_method = ?12, effective_date = ?13,
            end_date = ?14, sign_date = ?15, handler_party_a = ?16, handler_party_b = ?17,
            remark = ?18
        WHERE id = ?19",
        rusqlite::params![
            req.contract_no,
            req.contract_name,
            req.party_a,
            req.party_b,
            req.contact_person,
            req.contact_info,
            req.total_amount_with_tax,
            req.total_amount_without_tax,
            req.tax_amount,
            req.payment_cycle,
            req.payment_amount_with_tax,
            req.payment_method,
            req.effective_date,
            req.end_date,
            req.sign_date,
            req.handler_party_a,
            req.handler_party_b,
            req.remark,
            req.id,
        ],
    )
    .map_err(|e| e.to_string())?;

    contract_by_id(&conn, req.id)
}

#[tauri::command]
pub fn delete_contract(id: i64) -> Result<(), String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM contracts WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_contracts_batch(ids: Vec<i64>) -> Result<usize, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;
    let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
    let sql = format!("DELETE FROM contracts WHERE id IN ({})", placeholders.join(","));
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let deleted = stmt
        .execute(rusqlite::params_from_iter(ids.iter()))
        .map_err(|e| e.to_string())?;
    Ok(deleted)
}

#[tauri::command]
pub fn get_contract(id: i64) -> Result<Contract, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;
    contract_by_id(&conn, id)
}

#[tauri::command]
pub fn list_contracts(
    search: Option<String>,
    page: i64,
    per_page: i64,
) -> Result<Paginated<Contract>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let pattern = search.as_ref().map(|s| format!("%{}%", s));

    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM contracts
             WHERE (?1 IS NULL OR contract_no LIKE ?1 OR contract_name LIKE ?1
                    OR party_a LIKE ?1 OR party_b LIKE ?1 OR contact_person LIKE ?1
                    OR payment_cycle LIKE ?1 OR payment_method LIKE ?1
                    OR handler_party_a LIKE ?1 OR handler_party_b LIKE ?1
                    OR remark LIKE ?1)",
            [pattern.as_deref()],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let offset = (page - 1).max(0) * per_page;
    let mut stmt = conn
        .prepare(
            "SELECT id, contract_no, contract_name, party_a, party_b, contact_person, contact_info,
                    total_amount_with_tax, total_amount_without_tax, tax_amount, payment_cycle,
                    payment_amount_with_tax, payment_method, effective_date, end_date, sign_date,
                    handler_party_a, handler_party_b, remark, created_at
             FROM contracts
             WHERE (?1 IS NULL OR contract_no LIKE ?1 OR contract_name LIKE ?1
                    OR party_a LIKE ?1 OR party_b LIKE ?1 OR contact_person LIKE ?1
                    OR payment_cycle LIKE ?1 OR payment_method LIKE ?1
                    OR handler_party_a LIKE ?1 OR handler_party_b LIKE ?1
                    OR remark LIKE ?1)
             ORDER BY created_at DESC LIMIT ?2 OFFSET ?3",
        )
        .map_err(|e| e.to_string())?;

    let items = stmt
        .query_map(rusqlite::params![pattern.as_deref(), per_page, offset], contract_from_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(Paginated {
        items,
        total,
        page,
        per_page,
    })
}

fn parse_date(cell: &calamine::Data) -> Option<NaiveDate> {
    match cell {
        calamine::Data::DateTime(dt) => dt.as_datetime().map(|d| d.date()),
        calamine::Data::Float(n) => {
            chrono::NaiveDate::from_ymd_opt(1899, 12, 30).and_then(|base| {
                let days = *n as i64;
                base.checked_add_signed(chrono::Duration::days(days))
            })
        }
        calamine::Data::String(s) => {
            let s = s.trim();
            if s.is_empty() {
                return None;
            }
            NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .or_else(|_| NaiveDate::parse_from_str(s, "%Y/%m/%d"))
                .or_else(|_| NaiveDate::parse_from_str(s, "%Y年%m月%d日"))
                .ok()
        }
        _ => None,
    }
}

/// 解析金额字段（返回分值）。
/// 如果单元格是数字或可解析为数字的文本，返回对应的分值；
/// 如果单元格包含汉字或其他无法直接解析为数字的文本，
/// 则把原始文本作为备注返回，金额置为空。
fn parse_money_with_remark(cell: &calamine::Data) -> (Option<i64>, Option<String>) {
    match cell {
        calamine::Data::Float(n) => (Some((*n * 100.0).round() as i64), None),
        calamine::Data::Int(n) => (Some(*n * 100), None),
        calamine::Data::String(s) => {
            let raw = s.trim();
            if raw.is_empty() {
                return (None, None);
            }
            let normalized = raw.replace(',', "");
            match normalized.parse::<f64>() {
                Ok(n) => (Some((n * 100.0).round() as i64), None),
                Err(_) => (None, Some(raw.to_string())),
            }
        }
        _ => (None, None),
    }
}

/// 解析日期字段。
/// 能解析为日期时返回日期；非空但无法解析为日期的内容，
/// 把原始文本作为备注返回，日期置为空。
fn parse_date_with_remark(cell: &calamine::Data) -> (Option<NaiveDate>, Option<String>) {
    match parse_date(cell) {
        Some(d) => (Some(d), None),
        None => {
            let raw = as_string(cell);
            if raw.is_empty() {
                (None, None)
            } else {
                (None, Some(raw))
            }
        }
    }
}

fn as_string(cell: &calamine::Data) -> String {
    match cell {
        calamine::Data::String(s) => s.trim().to_string(),
        calamine::Data::Float(n) => n.to_string(),
        calamine::Data::Int(n) => n.to_string(),
        calamine::Data::Bool(b) => b.to_string(),
        calamine::Data::DateTime(_) => String::new(),
        _ => String::new(),
    }
}

fn generate_contract_no() -> String {
    let now = chrono::Local::now();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() % 100_000)
        .unwrap_or(0);
    format!("HT-{}-{:05}", now.format("%Y%m%d%H%M%S"), nanos)
}

#[tauri::command]
pub fn import_contracts_from_excel(path: String) -> Result<usize, String> {
    use calamine::{open_workbook, DataType, Reader, Xlsx};

    let mut workbook: Xlsx<_> = open_workbook(&path)
        .map_err(|e| format!("无法打开 Excel 文件：{}", e))?;
    let range = workbook
        .worksheet_range("Sheet1")
        .map_err(|e| format!("读取 Sheet1 失败：{}", e))?;

    if range.height() < 2 {
        return Err("Excel 数据行数不足，至少需要表头和一行数据".to_string());
    }

    let row0: Vec<String> = range
        .rows()
        .next()
        .unwrap_or(&[])
        .iter()
        .map(|c| c.as_string().unwrap_or_default().trim().to_string())
        .collect();
    let row1: Vec<String> = range
        .rows()
        .nth(1)
        .unwrap_or(&[])
        .iter()
        .map(|c| c.as_string().unwrap_or_default().trim().to_string())
        .collect();

    // 合并表头：优先使用第二行（子表头），否则使用第一行。
    // 子表头行通常包含合同甲方/乙方、甲方/乙方经办人。
    let known_subheaders = ["合同甲方", "合同乙方", "甲方经办人", "乙方经办人"];
    let has_subheader = row1.iter().any(|s| known_subheaders.contains(&s.as_str()));

    let mut effective_header: Vec<String> = Vec::new();
    let max_cols = row0.len().max(row1.len()).max(CONTRACT_HEADERS.len());
    for i in 0..max_cols {
        let r1 = row1.get(i).map(|s| s.as_str()).unwrap_or("");
        let r0 = row0.get(i).map(|s| s.as_str()).unwrap_or("");
        if has_subheader && !r1.is_empty() {
            effective_header.push(r1.to_string());
        } else {
            effective_header.push(r0.to_string());
        }
    }

    let mut header_map: HashMap<String, usize> = HashMap::new();
    for (i, h) in effective_header.iter().enumerate() {
        header_map.insert(h.clone(), i);
    }

    let required = ["合同编号", "合同名称"];
    for h in &required {
        if !header_map.contains_key(*h) {
            return Err(format!("缺少必要列：{}", h));
        }
    }

    let data_start = if has_subheader { 2 } else { 1 };

    let db = db();
    let mut conn = db.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let get_string = |row: &[calamine::Data], name: &str| -> String {
        header_map
            .get(name)
            .and_then(|i| row.get(*i))
            .map(as_string)
            .unwrap_or_default()
    };
    let get_money_fen = |row: &[calamine::Data], name: &str| -> (Option<i64>, Option<String>) {
        header_map
            .get(name)
            .and_then(|i| row.get(*i))
            .map(parse_money_with_remark)
            .unwrap_or((None, None))
    };
    let get_date = |row: &[calamine::Data], name: &str| -> (Option<NaiveDate>, Option<String>) {
        header_map
            .get(name)
            .and_then(|i| row.get(*i))
            .map(parse_date_with_remark)
            .unwrap_or((None, None))
    };

    let mut count = 0usize;
    for row in range.rows().skip(data_start) {
        let contract_name = get_string(row, "合同名称");
        if contract_name.is_empty() {
            continue;
        }

        let contract_no = {
            let no = get_string(row, "合同编号");
            if no.trim().is_empty() {
                generate_contract_no()
            } else {
                no
            }
        };

        let (total_with_tax, r_total_with_tax) = get_money_fen(row, "合同总金额（含税）");
        let (total_without_tax, r_total_without_tax) = get_money_fen(row, "合同总金额（不含税）");
        let (tax_amount, r_tax_amount) = get_money_fen(row, "税额");
        let (payment_amount, r_payment_amount) = get_money_fen(row, "每次支付金额（含税）");
        let (effective_date, r_effective_date) = get_date(row, "合同生效日期");
        let (end_date, r_end_date) = get_date(row, "合同终止日期");
        let (sign_date, r_sign_date) = get_date(row, "合同签订日期");

        // 金额/日期字段中无法解析的内容，以“字段名：内容”的形式追加到备注。
        let mut remark_parts: Vec<String> = Vec::new();
        let base_remark = get_string(row, "备注");
        if !base_remark.is_empty() {
            remark_parts.push(base_remark);
        }
        for (name, note) in [
            ("合同总金额（含税）", r_total_with_tax),
            ("合同总金额（不含税）", r_total_without_tax),
            ("税额", r_tax_amount),
            ("每次支付金额（含税）", r_payment_amount),
            ("合同生效日期", r_effective_date),
            ("合同终止日期", r_end_date),
            ("合同签订日期", r_sign_date),
        ] {
            if let Some(text) = note.filter(|s| !s.is_empty()) {
                remark_parts.push(format!("{}：{}", name, text));
            }
        }
        let remark = if remark_parts.is_empty() {
            None
        } else {
            Some(remark_parts.join("；"))
        };

        tx.execute(
            "INSERT INTO contracts (
                contract_no, contract_name, party_a, party_b, contact_person, contact_info,
                total_amount_with_tax, total_amount_without_tax, tax_amount, payment_cycle,
                payment_amount_with_tax, payment_method, effective_date, end_date, sign_date,
                handler_party_a, handler_party_b, remark
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            rusqlite::params![
                Some(contract_no),
                contract_name,
                Some(get_string(row, "合同甲方")).filter(|s| !s.is_empty()),
                Some(get_string(row, "合同乙方")).filter(|s| !s.is_empty()),
                Some(get_string(row, "对方联系人")).filter(|s| !s.is_empty()),
                Some(get_string(row, "联系方式")).filter(|s| !s.is_empty()),
                total_with_tax,
                total_without_tax,
                tax_amount,
                Some(get_string(row, "付款周期")).filter(|s| !s.is_empty()),
                payment_amount,
                Some(get_string(row, "付款方式")).filter(|s| !s.is_empty()),
                effective_date,
                end_date,
                sign_date,
                Some(get_string(row, "甲方经办人")).filter(|s| !s.is_empty()),
                Some(get_string(row, "乙方经办人")).filter(|s| !s.is_empty()),
                remark,
            ],
        )
        .map_err(|e| e.to_string())?;
        count += 1;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(count)
}

#[tauri::command]
pub fn export_contracts_xlsx() -> Result<Vec<u8>, String> {
    use rust_xlsxwriter::{Format, Workbook};

    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    let header_format = Format::new().set_bold().set_background_color(0xD9E1F2);

    // 第一行表头（含合并单元格）
    let merged_pairs: [(usize, usize, &str); 2] = [(3, 4, "合同当事人"), (16, 17, "经办人")];
    let mut skip_cols: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for (start, end, title) in merged_pairs {
        worksheet
            .merge_range(0, start as u16, 0, end as u16, title, &header_format)
            .map_err(|e| e.to_string())?;
        for c in start..=end {
            skip_cols.insert(c);
        }
    }

    for (col, h) in CONTRACT_HEADERS.iter().enumerate() {
        if skip_cols.contains(&col) {
            continue;
        }
        worksheet
            .write_string_with_format(0, col as u16, *h, &header_format)
            .map_err(|e| e.to_string())?;
    }

    // 第二行子表头
    let subheaders = [
        "", "", "", "合同甲方", "合同乙方", "", "", "", "", "", "", "", "", "", "", "", "甲方经办人", "乙方经办人", "",
    ];
    for (col, h) in subheaders.iter().enumerate() {
        if h.is_empty() {
            continue;
        }
        worksheet
            .write_string_with_format(1, col as u16, *h, &header_format)
            .map_err(|e| e.to_string())?;
    }

    let mut stmt = conn
        .prepare(
            "SELECT contract_no, contract_name, party_a, party_b, contact_person, contact_info,
                    total_amount_with_tax, total_amount_without_tax, tax_amount, payment_cycle,
                    payment_amount_with_tax, payment_method, effective_date, end_date, sign_date,
                    handler_party_a, handler_party_b, remark
             FROM contracts ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, Option<String>>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<i64>>(6)?,
                row.get::<_, Option<i64>>(7)?,
                row.get::<_, Option<i64>>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, Option<i64>>(10)?,
                row.get::<_, Option<String>>(11)?,
                row.get::<_, Option<NaiveDate>>(12)?,
                row.get::<_, Option<NaiveDate>>(13)?,
                row.get::<_, Option<NaiveDate>>(14)?,
                row.get::<_, Option<String>>(15)?,
                row.get::<_, Option<String>>(16)?,
                row.get::<_, Option<String>>(17)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut row_idx = 2u32;
    let write_opt_string = |worksheet: &mut rust_xlsxwriter::Worksheet,
                            r: u32,
                            c: u16,
                            val: &Option<String>|
     -> Result<(), String> {
        worksheet
            .write_string(r, c, val.as_deref().unwrap_or(""))
            .map_err(|e| e.to_string())?;
        Ok(())
    };
    let write_opt_money = |worksheet: &mut rust_xlsxwriter::Worksheet,
                           r: u32,
                           c: u16,
                           val: &Option<i64>|
     -> Result<(), String> {
        match val {
            Some(n) => { worksheet.write_number(r, c, *n as f64 / 100.0).map_err(|e| e.to_string())?; }
            None => { worksheet.write_string(r, c, "").map_err(|e| e.to_string())?; }
        }
        Ok(())
    };
    let write_opt_date = |worksheet: &mut rust_xlsxwriter::Worksheet,
                          r: u32,
                          c: u16,
                          val: &Option<NaiveDate>|
     -> Result<(), String> {
        worksheet
            .write_string(r, c, &val.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default())
            .map_err(|e| e.to_string())?;
        Ok(())
    };

    for row in rows {
        let (
            contract_no,
            contract_name,
            party_a,
            party_b,
            contact_person,
            contact_info,
            total_with_tax,
            total_without_tax,
            tax_amount,
            payment_cycle,
            payment_amount,
            payment_method,
            effective_date,
            end_date,
            sign_date,
            handler_party_a,
            handler_party_b,
            remark,
        ) = row.map_err(|e| e.to_string())?;

        worksheet
            .write_number(row_idx, 0, (row_idx - 1) as f64)
            .map_err(|e| e.to_string())?;
        write_opt_string(worksheet, row_idx, 1, &contract_no)?;
        worksheet
            .write_string(row_idx, 2, &contract_name)
            .map_err(|e| e.to_string())?;
        write_opt_string(worksheet, row_idx, 3, &party_a)?;
        write_opt_string(worksheet, row_idx, 4, &party_b)?;
        write_opt_string(worksheet, row_idx, 5, &contact_person)?;
        write_opt_string(worksheet, row_idx, 6, &contact_info)?;
        write_opt_money(worksheet, row_idx, 7, &total_with_tax)?;
        write_opt_money(worksheet, row_idx, 8, &total_without_tax)?;
        write_opt_money(worksheet, row_idx, 9, &tax_amount)?;
        write_opt_string(worksheet, row_idx, 10, &payment_cycle)?;
        write_opt_money(worksheet, row_idx, 11, &payment_amount)?;
        write_opt_string(worksheet, row_idx, 12, &payment_method)?;
        write_opt_date(worksheet, row_idx, 13, &effective_date)?;
        write_opt_date(worksheet, row_idx, 14, &end_date)?;
        write_opt_date(worksheet, row_idx, 15, &sign_date)?;
        write_opt_string(worksheet, row_idx, 16, &handler_party_a)?;
        write_opt_string(worksheet, row_idx, 17, &handler_party_b)?;
        write_opt_string(worksheet, row_idx, 18, &remark)?;

        row_idx += 1;
    }

    workbook.save_to_buffer().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_contracts_csv() -> Result<String, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let mut csv = String::from("\u{FEFF}");
    csv.push_str(&CONTRACT_HEADERS.join(","));
    csv.push('\n');

    let mut stmt = conn
        .prepare(
            "SELECT contract_no, contract_name, party_a, party_b, contact_person, contact_info,
                    total_amount_with_tax, total_amount_without_tax, tax_amount, payment_cycle,
                    payment_amount_with_tax, payment_method, effective_date, end_date, sign_date,
                    handler_party_a, handler_party_b, remark
             FROM contracts ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, Option<String>>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<i64>>(6)?,
                row.get::<_, Option<i64>>(7)?,
                row.get::<_, Option<i64>>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, Option<i64>>(10)?,
                row.get::<_, Option<String>>(11)?,
                row.get::<_, Option<NaiveDate>>(12)?,
                row.get::<_, Option<NaiveDate>>(13)?,
                row.get::<_, Option<NaiveDate>>(14)?,
                row.get::<_, Option<String>>(15)?,
                row.get::<_, Option<String>>(16)?,
                row.get::<_, Option<String>>(17)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut row_idx = 1u32;
    for row in rows {
        let (
            contract_no,
            contract_name,
            party_a,
            party_b,
            contact_person,
            contact_info,
            total_with_tax,
            total_without_tax,
            tax_amount,
            payment_cycle,
            payment_amount,
            payment_method,
            effective_date,
            end_date,
            sign_date,
            handler_party_a,
            handler_party_b,
            remark,
        ) = row.map_err(|e| e.to_string())?;

        csv.push_str(&format!(
            "{},\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",{},{},{},\"{}\",{},\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
            row_idx,
            escape_csv(&contract_no.unwrap_or_default()),
            escape_csv(&contract_name),
            escape_csv(&party_a.unwrap_or_default()),
            escape_csv(&party_b.unwrap_or_default()),
            escape_csv(&contact_person.unwrap_or_default()),
            escape_csv(&contact_info.unwrap_or_default()),
            fmt_opt_money(total_with_tax),
            fmt_opt_money(total_without_tax),
            fmt_opt_money(tax_amount),
            escape_csv(&payment_cycle.unwrap_or_default()),
            fmt_opt_money(payment_amount),
            escape_csv(&payment_method.unwrap_or_default()),
            fmt_opt_date(effective_date),
            fmt_opt_date(end_date),
            fmt_opt_date(sign_date),
            escape_csv(&handler_party_a.unwrap_or_default()),
            escape_csv(&handler_party_b.unwrap_or_default()),
            escape_csv(&remark.unwrap_or_default()),
        ));
        row_idx += 1;
    }

    Ok(csv)
}

fn escape_csv(s: &str) -> String {
    s.replace('"', "\"\"")
}

fn fmt_opt_money(v: Option<i64>) -> String {
    v.map(|n| format!("{:.2}", n as f64 / 100.0)).unwrap_or_default()
}

fn fmt_opt_date(v: Option<NaiveDate>) -> String {
    v.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default()
}
