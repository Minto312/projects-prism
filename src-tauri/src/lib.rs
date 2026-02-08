pub mod app;
pub mod domain;
pub mod infra;
pub mod interface;

use infra::persistence::sqlite::SqlitePersistence;
use infra::github::implementations::GitHubApiClient;
use crate::app::dtos::RateLimitInfo;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;
use tauri::Manager;

pub struct AppState {
    pub persistence: Arc<SqlitePersistence>,
    pub github_client: Arc<GitHubApiClient>,
    pub is_syncing: AtomicBool,
    pub rate_limit_cache: std::sync::Mutex<Option<(RateLimitInfo, Instant)>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data dir");

            let db_path = app_data_dir.join("prism.db");
            let persistence = SqlitePersistence::new(&db_path).expect("Failed to initialize database");

            let github_client = GitHubApiClient::new().expect("Failed to create GitHub API client");

            let state = AppState {
                persistence: Arc::new(persistence),
                github_client: Arc::new(github_client),
                is_syncing: AtomicBool::new(false),
                rate_limit_cache: std::sync::Mutex::new(None),
            };

            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            interface::tauri::commands::bootstrap::get_bootstrap,
            interface::tauri::commands::operations::append_ops,
            interface::tauri::commands::sync::sync_now,
            interface::tauri::commands::sync::get_sync_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
