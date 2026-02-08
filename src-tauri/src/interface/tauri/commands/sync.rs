use tauri::State;

use crate::app::dtos::{SyncResult, SyncState};
use crate::app::usecases::sync_to_github::SyncToGitHubUseCase;
use crate::AppState;

#[tauri::command]
pub async fn sync_now(state: State<'_, AppState>) -> Result<SyncResult, String> {
    SyncToGitHubUseCase::execute(state.github.as_ref(), state.persistence.as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_sync_state(state: State<'_, AppState>) -> Result<SyncState, String> {
    SyncToGitHubUseCase::get_sync_state(state.persistence.as_ref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resolve_conflict_with_current(
    state: State<'_, AppState>,
    operation_id: String,
) -> Result<(), String> {
    SyncToGitHubUseCase::resolve_conflict_with_current(state.persistence.as_ref(), &operation_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resolve_conflict_with_operation(
    state: State<'_, AppState>,
    operation_id: String,
) -> Result<(), String> {
    SyncToGitHubUseCase::resolve_conflict_with_operation(state.persistence.as_ref(), &operation_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cancel_operation(
    state: State<'_, AppState>,
    operation_id: String,
) -> Result<(), String> {
    SyncToGitHubUseCase::cancel_operation(state.persistence.as_ref(), &operation_id)
        .map_err(|e| e.to_string())
}
