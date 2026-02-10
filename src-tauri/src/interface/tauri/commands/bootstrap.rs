use tauri::State;

use crate::app::dtos::{BootstrapResponse, ProjectBootstrapResponse};
use crate::app::usecases::fetch_bootstrap::FetchBootstrapUseCase;
use crate::AppState;

#[tauri::command]
pub async fn get_bootstrap(state: State<'_, AppState>) -> Result<BootstrapResponse, String> {
    log::debug!("get_bootstrap: キャッシュからBootstrapデータを取得");
    let result = FetchBootstrapUseCase::from_cache(state.persistence.as_ref()).map_err(|e| e.to_string());
    if let Err(ref e) = result {
        log::error!("get_bootstrap: 失敗 - {}", e);
    }
    result
}

#[tauri::command]
pub async fn get_project_bootstrap(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<ProjectBootstrapResponse, String> {
    log::debug!("get_project_bootstrap: project_id={}", project_id);
    let result = FetchBootstrapUseCase::project_from_cache(state.persistence.as_ref(), &project_id)
        .map_err(|e| e.to_string());
    if let Err(ref e) = result {
        log::error!("get_project_bootstrap: 失敗 - {}", e);
    }
    result
}

#[tauri::command]
pub async fn refresh_bootstrap(state: State<'_, AppState>) -> Result<BootstrapResponse, String> {
    log::info!("refresh_bootstrap: GitHubからBootstrapデータをリフレッシュ開始");
    let result = FetchBootstrapUseCase::refresh(state.github.as_ref(), state.persistence.as_ref())
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(_) => log::info!("refresh_bootstrap: リフレッシュ完了"),
        Err(e) => log::error!("refresh_bootstrap: 失敗 - {}", e),
    }
    result
}

#[tauri::command]
pub async fn refresh_project_bootstrap(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<ProjectBootstrapResponse, String> {
    log::info!("refresh_project_bootstrap: project_id={}", project_id);
    let result = FetchBootstrapUseCase::refresh_project(
        state.github.as_ref(),
        state.persistence.as_ref(),
        &project_id,
    )
    .await
    .map_err(|e| e.to_string());
    match &result {
        Ok(_) => log::info!("refresh_project_bootstrap: 完了"),
        Err(e) => log::error!("refresh_project_bootstrap: 失敗 - {}", e),
    }
    result
}
