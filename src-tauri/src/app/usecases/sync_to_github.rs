use crate::app::dtos::{ConflictInfo, OperationStatus, SyncResult, SyncState};
use crate::app::ports::{GitHubPort, PersistencePort};
use crate::domain::DomainError;

pub struct SyncToGitHubUseCase<'a, P: PersistencePort, G: GitHubPort> {
    persistence: &'a P,
    github: &'a G,
}

impl<'a, P: PersistencePort, G: GitHubPort> SyncToGitHubUseCase<'a, P, G> {
    pub fn new(persistence: &'a P, github: &'a G) -> Self {
        Self {
            persistence,
            github,
        }
    }

    fn is_abort_error(e: &DomainError) -> bool {
        matches!(
            e,
            DomainError::AuthenticationError(_)
                | DomainError::RateLimitExceeded(_)
                | DomainError::NetworkError(_)
        )
    }

    pub async fn execute(&self) -> Result<SyncResult, DomainError> {
        let token = self
            .persistence
            .get_setting("github_token")
            .await?
            .ok_or_else(|| DomainError::AuthenticationError("No token configured".to_string()))?;

        let pending_ops = self.persistence.get_pending_operations().await?;

        let mut completed_count = 0;
        let mut conflict_count = 0;
        let mut failed_count = 0;
        let mut conflicts = Vec::new();

        for operation in pending_ops {
            self.persistence
                .update_operation_status(&operation.id, OperationStatus::Syncing, None, None)
                .await?;

            let current_status = self
                .github
                .fetch_item_current_status(
                    &token,
                    &operation.payload.item_id,
                    &operation.payload.status_field_id,
                )
                .await;

            let current_status = match current_status {
                Ok(status) => status,
                Err(e) => {
                    let should_abort = Self::is_abort_error(&e);
                    let now = chrono::Utc::now().timestamp_millis();

                    if should_abort {
                        // Transient/auth error: revert to pending for retry
                        self.persistence
                            .update_operation_status(
                                &operation.id,
                                OperationStatus::Pending,
                                None,
                                None,
                            )
                            .await?;
                    } else {
                        self.persistence
                            .update_operation_status(
                                &operation.id,
                                OperationStatus::Failed,
                                Some(&e.to_string()),
                                Some(now),
                            )
                            .await?;
                        failed_count += 1;
                    }

                    if should_abort {
                        break;
                    }
                    continue;
                }
            };

            let expected = Some(operation.precondition.expected_from_option_id.clone());
            if current_status != expected {
                let now = chrono::Utc::now().timestamp_millis();
                self.persistence
                    .update_operation_status(
                        &operation.id,
                        OperationStatus::Conflict,
                        None,
                        Some(now),
                    )
                    .await?;

                conflicts.push(ConflictInfo {
                    current_status_option_id: current_status,
                    expected_status_option_id: operation.precondition.expected_from_option_id.clone(),
                    operation,
                });

                conflict_count += 1;
                break;
            }

            let update_result = self
                .github
                .update_item_status(
                    &token,
                    &operation.payload.project_id,
                    &operation.payload.item_id,
                    &operation.payload.status_field_id,
                    &operation.payload.to_option_id,
                )
                .await;

            let now = chrono::Utc::now().timestamp_millis();
            match update_result {
                Ok(()) => {
                    self.persistence
                        .update_operation_status(
                            &operation.id,
                            OperationStatus::Completed,
                            None,
                            Some(now),
                        )
                        .await?;

                    if let Ok(Some(mut task)) = self
                        .persistence
                        .get_task(&operation.payload.item_id)
                        .await
                    {
                        task.status_option_id = Some(operation.payload.to_option_id.clone());
                        task.synced_at = Some(now);
                        self.persistence.upsert_task(&task).await?;
                    }

                    completed_count += 1;
                }
                Err(e) => {
                    let should_abort = Self::is_abort_error(&e);

                    if should_abort {
                        self.persistence
                            .update_operation_status(
                                &operation.id,
                                OperationStatus::Pending,
                                None,
                                None,
                            )
                            .await?;
                        break;
                    }

                    self.persistence
                        .update_operation_status(
                            &operation.id,
                            OperationStatus::Failed,
                            Some(&e.to_string()),
                            Some(now),
                        )
                        .await?;
                    failed_count += 1;
                }
            }
        }

        let now = chrono::Utc::now().timestamp_millis();
        self.persistence
            .set_setting("last_sync_at", &now.to_string())
            .await?;

        let rate_limit = self.github.get_rate_limit(&token).await.ok();

        let sync_state = SyncState {
            is_syncing: false,
            last_sync_at: Some(now),
            rate_limit,
            error: None,
        };

        Ok(SyncResult {
            completed_count,
            conflict_count,
            failed_count,
            conflicts,
            sync_state,
        })
    }
}
