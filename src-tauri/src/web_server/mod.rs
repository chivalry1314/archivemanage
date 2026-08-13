use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{Html, Json},
    routing::get,
    Router,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Mutex;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use crate::commands::archives::{
    get_archive, list_archive_categories, list_archive_tags, list_archives,
};
use crate::commands::members::list_members;
use crate::db::models::{ArchiveCategory, ArchiveDetail, ArchiveTag, Member, Paginated};

struct ServerState {
    handle: JoinHandle<()>,
    shutdown_tx: oneshot::Sender<()>,
    port: u16,
}

static SERVER_STATE: Lazy<Mutex<Option<ServerState>>> = Lazy::new(|| Mutex::new(None));

#[derive(Serialize)]
pub struct ServerStatus {
    pub running: bool,
    pub url: Option<String>,
    pub port: Option<u16>,
    pub error: Option<String>,
}

#[derive(Deserialize)]
struct SearchParams {
    q: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
}

async fn mobile_index() -> Html<&'static str> {
    Html(include_str!("mobile.html"))
}

async fn search_archives(
    Query(params): Query<SearchParams>,
) -> Result<Json<Paginated<ArchiveDetail>>, (StatusCode, String)> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let search = params.q.filter(|s| !s.trim().is_empty());
    list_archives(None, None, search, page, per_page)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

async fn archive_detail_handler(
    Path(id): Path<i64>,
) -> Result<Json<ArchiveDetail>, (StatusCode, String)> {
    get_archive(id)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

async fn list_categories() -> Result<Json<Vec<ArchiveCategory>>, (StatusCode, String)> {
    list_archive_categories()
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

async fn list_tags() -> Result<Json<Vec<ArchiveTag>>, (StatusCode, String)> {
    list_archive_tags()
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

async fn list_members_handler() -> Result<Json<Vec<Member>>, (StatusCode, String)> {
    list_members()
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

fn build_app() -> Router {
    Router::new()
        .route("/", get(mobile_index))
        .route("/api/archives/search", get(search_archives))
        .route("/api/archives/:id", get(archive_detail_handler))
        .route("/api/categories", get(list_categories))
        .route("/api/tags", get(list_tags))
        .route("/api/members", get(list_members_handler))
}

fn get_lan_url(port: u16) -> Option<String> {
    let ip = local_ip_address::local_ip().ok()?;
    Some(format!("http://{}:{}", ip, port))
}

#[tauri::command]
pub async fn start_mobile_server(port: u16) -> Result<ServerStatus, String> {
    {
        let state = SERVER_STATE.lock().map_err(|e| e.to_string())?;
        if state.is_some() {
            return Ok(ServerStatus {
                running: true,
                url: get_lan_url(port),
                port: Some(port),
                error: None,
            });
        }
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
        format!(
            "无法绑定端口 {}，请检查是否被占用或防火墙是否阻止：{}",
            port, e
        )
    })?;

    let app = build_app();
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let handle: JoinHandle<()> = tokio::spawn(async move {
        let server = axum::serve(listener, app).with_graceful_shutdown(async {
            let _ = shutdown_rx.await;
        });
        if let Err(e) = server.await {
            eprintln!("Mobile server error: {}", e);
        }
    });

    {
        let mut state = SERVER_STATE.lock().map_err(|e| e.to_string())?;
        *state = Some(ServerState {
            handle,
            shutdown_tx,
            port,
        });
    }

    Ok(ServerStatus {
        running: true,
        url: get_lan_url(port),
        port: Some(port),
        error: None,
    })
}

#[tauri::command]
pub async fn stop_mobile_server() -> Result<ServerStatus, String> {
    let state = {
        let mut state = SERVER_STATE.lock().map_err(|e| e.to_string())?;
        state.take()
    };
    if let Some(s) = state {
        let _ = s.shutdown_tx.send(());
        let _ = tokio::time::timeout(std::time::Duration::from_secs(3), s.handle).await;
    }
    Ok(ServerStatus {
        running: false,
        url: None,
        port: None,
        error: None,
    })
}

#[tauri::command]
pub async fn get_mobile_server_status() -> Result<ServerStatus, String> {
    let state = SERVER_STATE.lock().map_err(|e| e.to_string())?;
    if let Some(s) = state.as_ref() {
        Ok(ServerStatus {
            running: true,
            url: get_lan_url(s.port),
            port: Some(s.port),
            error: None,
        })
    } else {
        Ok(ServerStatus {
            running: false,
            url: None,
            port: None,
            error: None,
        })
    }
}
