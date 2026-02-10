use std::sync::Arc;

use tauri::Manager;
use tokio::sync::Mutex as TokioMutex;

use infra::github::implementations::github_api_client::GitHubApiClient;
use infra::persistence::sqlite::implementations::sqlite_persistence::SqlitePersistence;

pub mod app;
pub mod domain;
pub mod infra;
pub mod interface;

pub struct AppState {
    pub persistence: Arc<SqlitePersistence>,
    pub github: Arc<GitHubApiClient>,
    pub sync_lock: TokioMutex<()>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Debug)
                .build(),
        )
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("Failed to create app data dir");

            let db_path = app_dir.join("prism.db");
            let db_path_str = db_path.to_string_lossy().to_string();

            let persistence =
                SqlitePersistence::new(&db_path_str).expect("Failed to initialize SQLite");
            let github = GitHubApiClient::new();

            let state = AppState {
                persistence: Arc::new(persistence),
                github: Arc::new(github),
                sync_lock: TokioMutex::new(()),
            };

            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            interface::tauri::commands::bootstrap::get_bootstrap,
            interface::tauri::commands::bootstrap::get_project_bootstrap,
            interface::tauri::commands::bootstrap::refresh_bootstrap,
            interface::tauri::commands::bootstrap::refresh_project_bootstrap,
            interface::tauri::commands::operations::append_ops,
            interface::tauri::commands::sync::sync_now,
            interface::tauri::commands::sync::get_sync_state,
            interface::tauri::commands::sync::resolve_conflict_with_current,
            interface::tauri::commands::sync::resolve_conflict_with_operation,
            interface::tauri::commands::sync::cancel_operation,
            interface::tauri::commands::settings::set_pat,
            interface::tauri::commands::settings::has_pat,
            interface::tauri::commands::settings::clear_pat,
            interface::tauri::commands::updater::get_update_channel,
            interface::tauri::commands::updater::set_update_channel,
            interface::tauri::commands::updater::check_for_update,
            interface::tauri::commands::updater::download_and_install_update,
            interface::tauri::commands::debug::get_debug_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
