use std::sync::atomic::{AtomicBool, Ordering};
use tauri::State;

use crate::app::dtos::{SyncResult, SyncState};
use crate::app::ports::{GitHubPort, PersistencePort};
use crate::app::usecases::SyncToGitHubUseCase;
use crate::domain::DomainError;
use crate::AppState;

struct SyncGuard<'a>(&'a AtomicBool);

impl Drop for SyncGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

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

    let _guard = SyncGuard(&state.is_syncing);

    let usecase = SyncToGitHubUseCase::new(
        state.persistence.as_ref(),
        state.github_client.as_ref(),
    );
    usecase.execute().await
}

#[tauri::command]
pub async fn get_sync_state(state: State<'_, AppState>) -> Result<SyncState, DomainError> {
    let persistence = state.persistence.as_ref();
    let github = state.github_client.as_ref();

    let is_syncing = state.is_syncing.load(Ordering::SeqCst);

    let token = persistence.get_setting("github_token").await?;

    let rate_limit = if let Some(ref token) = token {
        // Check cache (60s TTL) to avoid consuming rate limit on polling
        let cached = {
            let cache = state
                .rate_limit_cache
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            match &*cache {
                Some((info, instant))
                    if instant.elapsed() < std::time::Duration::from_secs(60) =>
                {
                    Some(info.clone())
                }
                _ => None,
            }
        };

        if let Some(info) = cached {
            Some(info)
        } else {
            match github.get_rate_limit(token).await {
                Ok(info) => {
                    let mut cache = state
                        .rate_limit_cache
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    *cache = Some((info.clone(), std::time::Instant::now()));
                    Some(info)
                }
                Err(_) => None,
            }
        }
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
