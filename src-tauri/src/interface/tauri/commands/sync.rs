use std::sync::atomic::Ordering;
use tauri::State;

use crate::app::dtos::{SyncResult, SyncState};
use crate::app::ports::{GitHubPort, PersistencePort};
use crate::app::usecases::SyncToGitHubUseCase;
use crate::domain::DomainError;
use crate::AppState;

#[tauri::command]
pub async fn sync_now(state: State<'_, AppState>) -> Result<SyncResult, DomainError> {
    if state
        .is_syncing
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err(DomainError::InvalidInput(
            "Sync is already in progress".to_string(),
        ));
    }

    let result = async {
        let usecase = SyncToGitHubUseCase::new(
            state.persistence.as_ref(),
            state.github_client.as_ref(),
        );
        usecase.execute().await
    }
    .await;

    state.is_syncing.store(false, Ordering::SeqCst);

    result
}

#[tauri::command]
pub async fn get_sync_state(state: State<'_, AppState>) -> Result<SyncState, DomainError> {
    let persistence = state.persistence.as_ref();
    let github = state.github_client.as_ref();

    let is_syncing = state.is_syncing.load(Ordering::SeqCst);

    let token = persistence.get_setting("github_token").await?;

    let rate_limit = if let Some(ref token) = token {
        github.get_rate_limit(token).await.ok()
    } else {
        None
    };

    let last_sync_at = persistence.get_setting("last_sync_at").await?;
    let last_sync_at = last_sync_at.and_then(|s| s.parse().ok());

    let pending_count = persistence.get_pending_operations().await?.len();
    let conflict_count = persistence.get_conflict_operations().await?.len();

    let error = if conflict_count > 0 {
        Some(format!("{} conflict(s) need resolution", conflict_count))
    } else if pending_count > 0 {
        Some(format!("{} operation(s) pending sync", pending_count))
    } else {
        None
    };

    Ok(SyncState {
        is_syncing,
        last_sync_at,
        rate_limit,
        error,
    })
}
