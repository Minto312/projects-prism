use crate::app::dtos::{ConflictInfo, OperationStatus, SyncResult, SyncResultFailure, SyncState, SyncStatus};
use crate::app::ports::github_port::GitHubPort;
use crate::app::ports::persistence_port::PersistencePort;
use crate::domain::errors::DomainError;

pub struct SyncToGitHubUseCase;

impl SyncToGitHubUseCase {
    pub async fn execute(
        github: &dyn GitHubPort,
        persistence: &dyn PersistencePort,
    ) -> Result<SyncResult, DomainError> {
        let pat = persistence
            .get_setting("github_pat")?
            .ok_or_else(|| DomainError::Authentication("PAT is not configured".to_string()))?;

        let pending_ops = persistence.get_operations_by_status(&OperationStatus::Pending)?;

        let mut completed = Vec::new();
        let mut failed = Vec::new();
        let mut conflicts = Vec::new();

        let now = chrono::Utc::now().timestamp_millis();
        persistence.set_setting("last_sync_attempt_at", &now.to_string())?;

        for mut op in pending_ops {
            // ステータスを syncing に更新
            persistence.update_operation_status(&op.id, &OperationStatus::Syncing, None)?;
            op.status = OperationStatus::Syncing;

            // 現在の Status を取得して precondition チェック
            let current_status = github
                .fetch_item_current_status(
                    &pat,
                    &op.payload.item_id,
                    &op.payload.project_id,
                )
                .await;

            match current_status {
                Ok(current_option_id) => {
                    let current_id = current_option_id.unwrap_or_default();

                    if current_id == op.precondition.expected_from_option_id {
                        // precondition OK → mutation 実行
                        match github
                            .update_item_status(
                                &pat,
                                &op.payload.project_id,
                                &op.payload.item_id,
                                &op.payload.status_field_id,
                                &op.payload.to_option_id,
                            )
                            .await
                        {
                            Ok(()) => {
                                persistence.update_operation_status(
                                    &op.id,
                                    &OperationStatus::Completed,
                                    None,
                                )?;
                                op.status = OperationStatus::Completed;
                                op.resolved_at = Some(chrono::Utc::now().timestamp_millis());
                                completed.push(op);
                            }
                            Err(e) => {
                                let error_msg = e.to_string();
                                persistence.update_operation_status(
                                    &op.id,
                                    &OperationStatus::Failed,
                                    Some(&error_msg),
                                )?;
                                op.status = OperationStatus::Failed;
                                op.error_message = Some(error_msg.clone());
                                failed.push(SyncResultFailure {
                                    operation: op,
                                    error: error_msg,
                                });
                            }
                        }
                    } else {
                        // precondition NG → conflict
                        // option 名をキャッシュから取得する（ベストエフォート）
                        let current_option_name = persistence
                            .get_all_status_options()
                            .ok()
                            .and_then(|opts| {
                                opts.into_iter()
                                    .find(|o| o.id == current_id)
                                    .map(|o| o.name)
                            })
                            .unwrap_or_else(|| current_id.clone());

                        persistence.update_operation_status(
                            &op.id,
                            &OperationStatus::Conflict,
                            None,
                        )?;
                        // コンフリクト情報を永続化
                        persistence.set_setting(
                            &format!("conflict_current_option_id:{}", op.id),
                            &current_id,
                        )?;
                        persistence.set_setting(
                            &format!("conflict_current_option_name:{}", op.id),
                            &current_option_name,
                        )?;
                        op.status = OperationStatus::Conflict;

                        conflicts.push(ConflictInfo {
                            operation: op,
                            current_option_id: current_id,
                            current_option_name,
                        });

                        // conflict 発生時は後続の処理を停止（C2方式）
                        break;
                    }
                }
                Err(DomainError::RateLimited { reset_at }) => {
                    // レート制限 → pending に戻して中断
                    persistence.update_operation_status(
                        &op.id,
                        &OperationStatus::Pending,
                        None,
                    )?;
                    persistence.set_setting("rate_limit_reset_at", &reset_at.to_string())?;
                    break;
                }
                Err(DomainError::Authentication(_)) => {
                    // 認証エラー → pending に戻して中断
                    persistence.update_operation_status(
                        &op.id,
                        &OperationStatus::Pending,
                        None,
                    )?;
                    break;
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    persistence.update_operation_status(
                        &op.id,
                        &OperationStatus::Failed,
                        Some(&error_msg),
                    )?;
                    op.status = OperationStatus::Failed;
                    op.error_message = Some(error_msg.clone());
                    failed.push(SyncResultFailure {
                        operation: op,
                        error: error_msg,
                    });
                }
            }
        }

        if !completed.is_empty() {
            let now = chrono::Utc::now().timestamp_millis();
            persistence.set_setting("last_sync_success_at", &now.to_string())?;
        }

        let state = Self::get_sync_state(persistence)?;

        Ok(SyncResult {
            completed,
            failed,
            conflicts,
            state,
        })
    }

    pub fn get_sync_state(persistence: &dyn PersistencePort) -> Result<SyncState, DomainError> {
        let pending_ops = persistence.get_operations_by_status(&OperationStatus::Pending)?;
        let conflict_ops = persistence.get_operations_by_status(&OperationStatus::Conflict)?;

        let pending_count = pending_ops.len() as i32;
        let conflict_count = conflict_ops.len() as i32;

        let rate_limit_reset_at = persistence
            .get_setting("rate_limit_reset_at")?
            .and_then(|s| s.parse::<i64>().ok());

        let last_error = persistence.get_setting("last_sync_error")?;

        let last_sync_attempt_at = persistence
            .get_setting("last_sync_attempt_at")?
            .and_then(|s| s.parse::<i64>().ok());

        let last_sync_success_at = persistence
            .get_setting("last_sync_success_at")?
            .and_then(|s| s.parse::<i64>().ok());

        let status = if conflict_count > 0 {
            SyncStatus::PausedConflict
        } else if let Some(reset_at) = rate_limit_reset_at {
            let now = chrono::Utc::now().timestamp_millis();
            if reset_at > now {
                SyncStatus::PausedRateLimited
            } else {
                // レート制限解除済み
                persistence.delete_setting("rate_limit_reset_at").ok();
                if pending_count > 0 {
                    SyncStatus::Idle
                } else {
                    SyncStatus::Idle
                }
            }
        } else if last_error.is_some() {
            SyncStatus::Error
        } else {
            SyncStatus::Idle
        };

        Ok(SyncState {
            status,
            pending_count,
            conflict_count,
            rate_limit_reset_at,
            last_error,
            last_sync_attempt_at,
            last_sync_success_at,
        })
    }

    /// コンフリクトを解決（現在の値を採用 = 操作を破棄）
    pub fn resolve_conflict_with_current(
        persistence: &dyn PersistencePort,
        operation_id: &str,
    ) -> Result<(), DomainError> {
        persistence.update_operation_status(
            operation_id,
            &OperationStatus::Completed,
            None,
        )?;
        Self::cleanup_conflict_settings(persistence, operation_id);
        Ok(())
    }

    /// コンフリクトを解決（操作を再試行 = pending に戻す）
    pub fn resolve_conflict_with_operation(
        persistence: &dyn PersistencePort,
        operation_id: &str,
        current_option_id: &str,
    ) -> Result<(), DomainError> {
        // precondition を現在のリモート値に更新してから pending に戻す
        persistence.update_operation_precondition(operation_id, current_option_id)?;
        persistence.update_operation_status(
            operation_id,
            &OperationStatus::Pending,
            None,
        )?;
        Self::cleanup_conflict_settings(persistence, operation_id);
        Ok(())
    }

    /// 操作をキャンセル（削除）
    pub fn cancel_operation(
        persistence: &dyn PersistencePort,
        operation_id: &str,
    ) -> Result<(), DomainError> {
        Self::cleanup_conflict_settings(persistence, operation_id);
        persistence.delete_operation(operation_id)?;
        Ok(())
    }

    fn cleanup_conflict_settings(persistence: &dyn PersistencePort, operation_id: &str) {
        let _ = persistence.delete_setting(&format!("conflict_current_option_id:{}", operation_id));
        let _ = persistence.delete_setting(&format!("conflict_current_option_name:{}", operation_id));
    }
}
