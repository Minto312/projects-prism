use crate::app::dtos::{
    BootstrapResponse, ConflictInfo, Operation, OperationStatus, ProjectBootstrapResponse,
};
use crate::app::ports::github_port::GitHubPort;
use crate::app::ports::persistence_port::PersistencePort;
use crate::domain::errors::DomainError;

fn build_conflict_info(persistence: &dyn PersistencePort, op: Operation) -> ConflictInfo {
    let current_option_id = persistence
        .get_setting(&format!("conflict_current_option_id:{}", op.id))
        .ok()
        .flatten()
        .unwrap_or_default();
    let current_option_name = persistence
        .get_setting(&format!("conflict_current_option_name:{}", op.id))
        .ok()
        .flatten()
        .unwrap_or_default();
    ConflictInfo {
        operation: op,
        current_option_id,
        current_option_name,
    }
}

pub struct FetchBootstrapUseCase;

impl FetchBootstrapUseCase {
    /// キャッシュからブートストラップデータを取得（refresh=false時）
    pub fn from_cache(persistence: &dyn PersistencePort) -> Result<BootstrapResponse, DomainError> {
        let projects = persistence.get_all_projects()?;
        let status_fields = persistence.get_all_status_fields()?;
        let status_options = persistence.get_all_status_options()?;
        let tasks = persistence.get_all_tasks()?;

        let pending_operations =
            persistence.get_operations_by_status(&OperationStatus::Pending)?;
        let conflict_operations =
            persistence.get_operations_by_status(&OperationStatus::Conflict)?;

        let conflicts = conflict_operations
            .into_iter()
            .map(|op| build_conflict_info(persistence, op))
            .collect();

        let current_user_login = persistence
            .get_setting("current_user_login")?
            .unwrap_or_default();

        Ok(BootstrapResponse {
            projects,
            status_fields,
            status_options,
            tasks,
            pending_operations,
            conflicts,
            current_user_login,
        })
    }

    /// GitHubからデータを取得してキャッシュを更新
    pub async fn refresh(
        github: &dyn GitHubPort,
        persistence: &dyn PersistencePort,
    ) -> Result<BootstrapResponse, DomainError> {
        let pat = persistence
            .get_setting("github_pat")?
            .ok_or_else(|| DomainError::Authentication("PAT is not configured".to_string()))?;

        // ユーザー検証
        let current_user_login = github.validate_token(&pat).await?;
        persistence.set_setting("current_user_login", &current_user_login)?;

        // プロジェクト一覧取得
        let projects = github.fetch_projects(&pat).await?;
        let project_ids: Vec<String> = projects.iter().map(|p| p.id.clone()).collect();

        // キャッシュ更新
        for project in &projects {
            persistence.upsert_project(project)?;
        }
        persistence.delete_projects_not_in(&project_ids)?;

        // 各プロジェクトのアイテム取得
        let mut all_status_fields = Vec::new();
        let mut all_status_options = Vec::new();
        let mut all_tasks = Vec::new();

        for project in &projects {
            let project_data = github.fetch_project_items(&pat, &project.id).await?;

            // Status field を更新
            persistence.delete_status_fields_by_project(&project.id)?;
            if let Some(ref field) = project_data.status_field {
                persistence.upsert_status_field(field)?;
                persistence.delete_status_options_by_field(&field.id)?;
                for option in &project_data.status_options {
                    persistence.upsert_status_option(option)?;
                }
                all_status_fields.push(field.clone());
                all_status_options.extend(project_data.status_options.clone());
            }

            // Tasks を更新
            persistence.delete_tasks_by_project(&project.id)?;
            for task in &project_data.tasks {
                persistence.upsert_task(task)?;
            }
            all_tasks.extend(project_data.tasks);
        }

        let pending_operations =
            persistence.get_operations_by_status(&OperationStatus::Pending)?;
        let conflict_operations =
            persistence.get_operations_by_status(&OperationStatus::Conflict)?;

        let conflicts = conflict_operations
            .into_iter()
            .map(|op| build_conflict_info(persistence, op))
            .collect();

        Ok(BootstrapResponse {
            projects,
            status_fields: all_status_fields,
            status_options: all_status_options,
            tasks: all_tasks,
            pending_operations,
            conflicts,
            current_user_login,
        })
    }

    /// 特定プロジェクトのブートストラップデータをキャッシュから取得
    pub fn project_from_cache(
        persistence: &dyn PersistencePort,
        project_id: &str,
    ) -> Result<ProjectBootstrapResponse, DomainError> {
        let project = persistence
            .get_project(project_id)?
            .ok_or_else(|| DomainError::NotFound(format!("Project not found: {}", project_id)))?;

        let status_fields = persistence.get_status_fields_by_project(project_id)?;
        let status_field = status_fields.into_iter().next();

        let status_options = if let Some(ref field) = status_field {
            persistence.get_status_options_by_field(&field.id)?
        } else {
            Vec::new()
        };

        let tasks = persistence.get_tasks_by_project(project_id)?;

        let statuses = [OperationStatus::Pending, OperationStatus::Conflict];
        let all_ops =
            persistence.get_operations_by_project_and_status(project_id, &statuses)?;

        let mut pending_operations = Vec::new();
        let mut conflicts = Vec::new();
        for op in all_ops {
            match op.status {
                OperationStatus::Conflict => {
                    conflicts.push(build_conflict_info(persistence, op));
                }
                _ => {
                    pending_operations.push(op);
                }
            }
        }

        Ok(ProjectBootstrapResponse {
            project,
            status_field,
            status_options,
            tasks,
            pending_operations,
            conflicts,
        })
    }

    /// 特定プロジェクトのデータをGitHubから再取得
    pub async fn refresh_project(
        github: &dyn GitHubPort,
        persistence: &dyn PersistencePort,
        project_id: &str,
    ) -> Result<ProjectBootstrapResponse, DomainError> {
        let pat = persistence
            .get_setting("github_pat")?
            .ok_or_else(|| DomainError::Authentication("PAT is not configured".to_string()))?;

        let project_data = github.fetch_project_items(&pat, project_id).await?;

        // プロジェクト自体を更新
        persistence.upsert_project(&project_data.project)?;

        // Status field を更新
        persistence.delete_status_fields_by_project(project_id)?;
        if let Some(ref field) = project_data.status_field {
            persistence.upsert_status_field(field)?;
            persistence.delete_status_options_by_field(&field.id)?;
            for option in &project_data.status_options {
                persistence.upsert_status_option(option)?;
            }
        }

        // Tasks を更新
        persistence.delete_tasks_by_project(project_id)?;
        for task in &project_data.tasks {
            persistence.upsert_task(task)?;
        }

        let statuses = [OperationStatus::Pending, OperationStatus::Conflict];
        let all_ops =
            persistence.get_operations_by_project_and_status(project_id, &statuses)?;

        let mut pending_operations = Vec::new();
        let mut conflicts = Vec::new();
        for op in all_ops {
            match op.status {
                OperationStatus::Conflict => {
                    conflicts.push(build_conflict_info(persistence, op));
                }
                _ => {
                    pending_operations.push(op);
                }
            }
        }

        Ok(ProjectBootstrapResponse {
            project: project_data.project,
            status_field: project_data.status_field,
            status_options: project_data.status_options,
            tasks: project_data.tasks,
            pending_operations,
            conflicts,
        })
    }
}
