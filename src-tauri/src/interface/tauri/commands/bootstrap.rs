use tauri::State;

use crate::app::dtos::{BootstrapResponse, ProjectBootstrapResponse};
use crate::app::usecases::fetch_bootstrap::FetchBootstrapUseCase;
use crate::AppState;

#[tauri::command]
pub async fn get_bootstrap(state: State<'_, AppState>) -> Result<BootstrapResponse, String> {
    FetchBootstrapUseCase::from_cache(state.persistence.as_ref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_project_bootstrap(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<ProjectBootstrapResponse, String> {
    FetchBootstrapUseCase::project_from_cache(state.persistence.as_ref(), &project_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn refresh_bootstrap(state: State<'_, AppState>) -> Result<BootstrapResponse, String> {
    FetchBootstrapUseCase::refresh(state.github.as_ref(), state.persistence.as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn refresh_project_bootstrap(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<ProjectBootstrapResponse, String> {
    FetchBootstrapUseCase::refresh_project(
        state.github.as_ref(),
        state.persistence.as_ref(),
        &project_id,
    )
    .await
    .map_err(|e| e.to_string())
}
