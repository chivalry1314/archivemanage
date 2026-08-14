use crate::db::{db, get_ai_config, models::*, set_ai_config};
use serde_json::json;

#[tauri::command]
pub fn get_ai_config_command() -> Result<AiConfig, String> {
    get_ai_config()
}

#[tauri::command]
pub fn set_ai_config_command(config: AiConfig) -> Result<(), String> {
    set_ai_config(config)
}

#[tauri::command]
pub async fn analyze_archive_box(
    req: AnalyzeArchiveBoxRequest,
    existing_boxes: Vec<ArchiveBox>,
) -> Result<ArchiveBoxSuggestion, String> {
    let config = get_ai_config()?;
    if !config.enabled {
        return Err("档案盒 AI 识别未开启，请到设置中启用。".to_string());
    }
    if config.api_key.trim().is_empty() {
        return Err("未配置 AI API Key，请到设置中填写。".to_string());
    }
    if config.base_url.trim().is_empty() || config.model.trim().is_empty() {
        return Err("AI 配置不完整，请检查 API 地址和模型名。".to_string());
    }

    let category_name = if let Some(cid) = req.category_id {
        let db = db();
        let conn = db.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT name FROM archive_categories WHERE id = ?1",
            [cid],
            |row| row.get::<_, String>(0),
        )
        .ok()
    } else {
        None
    };

    let box_list = existing_boxes
        .iter()
        .map(|b| format!("- {}（id={}）", b.name, b.id))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        "你是一名物业档案管理助手。请根据以下档案信息，从已有档案盒列表中推荐最合适的档案盒；\
         如果没有合适的，请建议一个新的档案盒名称。\n\n\
         档案标题：{}\n\
         档案分类：{}\n\n\
         已有档案盒列表：\n{}\n\n\
         请严格按以下 JSON 格式返回，不要包含其他内容：\n\
         {{\"box_name\": \"推荐或建议的档案盒名称\", \"reason\": \"推荐理由\"}}",
        req.title,
        category_name.as_deref().unwrap_or("未分类"),
        if box_list.is_empty() {
            "（暂无）".to_string()
        } else {
            box_list
        }
    );

    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let body = json!({
        "model": config.model,
        "messages": [
            {"role": "system", "content": "你是一个档案分类助手，只返回 JSON。"},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.3
    });

    let res = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("AI 请求失败：{}", e))?;

    if !res.status().is_success() {
        let text = res.text().await.unwrap_or_default();
        return Err(format!("AI 接口返回错误：{}", text));
    }

    let data: serde_json::Value = res
        .json()
        .await
        .map_err(|e| format!("解析 AI 响应失败：{}", e))?;

    let content = data["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("AI 响应内容为空")?;

    let parsed: serde_json::Value = serde_json::from_str(content)
        .map_err(|e| format!("AI 返回不是合法 JSON：{}，内容：{}", e, content))?;

    let box_name = parsed["box_name"]
        .as_str()
        .ok_or("AI 响应缺少 box_name")?
        .trim()
        .to_string();

    if box_name.is_empty() {
        return Err("AI 返回的档案盒名称为空".to_string());
    }

    let normalized = box_name.replace([' ', '　'], "");
    let matched = existing_boxes.iter().find(|b| {
        b.name.replace([' ', '　'], "") == normalized
    });

    let (is_existing, matched_box_id) = match matched {
        Some(b) => (true, Some(b.id)),
        None => (false, None),
    };

    let reason = parsed["reason"].as_str().unwrap_or("").to_string();

    Ok(ArchiveBoxSuggestion {
        box_name,
        reason,
        is_existing,
        matched_box_id,
    })
}
