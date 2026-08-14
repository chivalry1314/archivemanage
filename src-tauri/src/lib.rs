use tauri::Manager;

mod commands;
mod db;
mod scheduler;
mod web_server;

use commands::ai::*;
use commands::archive_boxes::*;
use commands::archives::*;
use commands::export::*;
use commands::instances::*;
use commands::members::*;
use commands::tasks::*;
use web_server::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_dir).ok();
            db::init_db(app_dir).expect("Failed to initialize database");

            // Start background scheduler
            let handle = app.handle().clone();
            scheduler::start_scheduler(handle);

            // Setup tray icon
            setup_tray(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // AI
            get_ai_config_command,
            set_ai_config_command,
            list_ai_models,
            analyze_archive_box,
            // Members
            create_member,
            update_member,
            delete_member,
            list_members,
            list_members_paged,
            // Tasks
            create_task,
            update_task,
            delete_task,
            list_tasks,
            get_task,
            // Instances
            list_instances,
            list_task_instances,
            get_today_instances,
            get_pending_instances,
            get_overdue_instances,
            complete_instance,
            uncomplete_instance,
            get_dashboard_stats,
            // Archives
            create_archive_category,
            update_archive_category,
            delete_archive_category,
            list_archive_categories,
            list_archive_categories_paged,
            create_archive,
            update_archive,
            delete_archive,
            list_archives,
            get_archive,
            update_archive_status,
            get_archive_file_path,
            create_archive_borrow,
            return_archive_borrow,
            update_archive_borrow,
            delete_archive_borrow,
            list_archive_borrows,
            list_active_archive_borrows,
            get_archive_borrow,
            get_archive_stats,
            import_archives_from_excel,
            // Archive Boxes
            create_archive_box,
            update_archive_box,
            delete_archive_box,
            list_archive_boxes,
            list_archive_boxes_paged,
            get_archive_box,
            // Archive Tags
            create_archive_tag,
            update_archive_tag,
            delete_archive_tag,
            list_archive_tags,
            list_archive_tags_paged,
            list_archives_by_tag,
            // Export
            export_instances_csv,
            export_instances_json,
            export_member_stats_csv,
            export_archives_csv,
            export_archives_xlsx,
            export_archive_borrows_csv,
            export_archive_borrows_xlsx,
            save_file_command,
            get_db_path,
            set_db_path_command,
            // Mobile server
            start_mobile_server,
            stop_mobile_server,
            get_mobile_server_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
    use tauri::tray::TrayIconBuilder;

    let quit_i = MenuItem::with_id(app.handle(), "quit", "退出", true, None::<&str>)?;
    let show_i = MenuItem::with_id(app.handle(), "show", "显示窗口", true, None::<&str>)?;
    let menu = Menu::with_items(
        app.handle(),
        &[
            &show_i,
            &PredefinedMenuItem::separator(app.handle())?,
            &quit_i,
        ],
    )?;

    let icon = load_tray_icon()?;

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .tooltip("档案管理OS")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                std::process::exit(0);
            }
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            use tauri::tray::TrayIconEvent;
            if let TrayIconEvent::DoubleClick { .. } = event {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app.handle())?;

    Ok(())
}

fn load_tray_icon() -> Result<tauri::image::Image<'static>, Box<dyn std::error::Error>> {
    let bytes = include_bytes!("../icons/icon.png");
    let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(tauri::image::Image::new_owned(rgba.into_raw(), width, height))
}
