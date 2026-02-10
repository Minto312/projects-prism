use tauri::State;

use crate::app::dtos::{SyncResult, SyncState};
use crate::app::usecases::sync_to_github::SyncToGitHubUseCase;
use crate::AppState;

#[tauri::command]
pub async fn sync_now(state: State<'_, AppState>) -> Result<SyncResult, String> {
    log::info!("sync_now: 同期開始");
    let _guard = state.sync_lock.lock().await;
    let result = SyncToGitHubUseCase::execute(state.github.as_ref(), state.persistence.as_ref())
        .await
        .map_err(|e| e.to_string());
    match &result {
        Ok(r) => log::info!(
            "sync_now: 完了 - completed={}, failed={}, conflicts={}",
            r.completed.len(),
            r.failed.len(),
            r.conflicts.len()
        ),
        Err(e) => log::error!("sync_now: 失敗 - {}", e),
    }
    result
}

#[tauri::command]
pub async fn get_sync_state(state: State<'_, AppState>) -> Result<SyncState, String> {
    log::debug!("get_sync_state: 同期状態を取得");
    SyncToGitHubUseCase::get_sync_state(state.persistence.as_ref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resolve_conflict_with_current(
    state: State<'_, AppState>,
    operation_id: String,
) -> Result<(), String> {
    log::info!("resolve_conflict_with_current: operation_id={}", operation_id);
    SyncToGitHubUseCase::resolve_conflict_with_current(state.persistence.as_ref(), &operation_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resolve_conflict_with_operation(
    state: State<'_, AppState>,
    operation_id: String,
    current_option_id: String,
) -> Result<(), String> {
    log::info!(
        "resolve_conflict_with_operation: operation_id={}, current_option_id={}",
        operation_id, current_option_id
    );
    SyncToGitHubUseCase::resolve_conflict_with_operation(
        state.persistence.as_ref(),
        &operation_id,
        &current_option_id,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cancel_operation(
    state: State<'_, AppState>,
    operation_id: String,
) -> Result<(), String> {
    log::info!("cancel_operation: operation_id={}", operation_id);
    SyncToGitHubUseCase::cancel_operation(state.persistence.as_ref(), &operation_id)
        .map_err(|e| e.to_string())
}
