use crate::app::dtos::{BootstrapResponse, ConflictInfo, ProjectDto, SyncState, TaskDto};
use crate::app::ports::{GitHubPort, PersistencePort};
use crate::domain::DomainError;

pub struct FetchBootstrapUseCase<'a, P: PersistencePort, G: GitHubPort> {
    persistence: &'a P,
    github: &'a G,
}

impl<'a, P: PersistencePort, G: GitHubPort> FetchBootstrapUseCase<'a, P, G> {
    pub fn new(persistence: &'a P, github: &'a G) -> Self {
        Self {
            persistence,
            github,
        }
    }

    pub async fn execute(&self, refresh: bool) -> Result<BootstrapResponse, DomainError> {
        let token = self.persistence.get_setting("github_token").await?;

        let (projects, tasks) = if refresh {
            if let Some(ref token) = token {
                self.refresh_from_github(token).await?
            } else {
                self.load_from_cache().await?
            }
        } else {
            self.load_from_cache().await?
        };

        let pending_ops = self.persistence.get_pending_operations().await?;
        let conflict_ops = self.persistence.get_conflict_operations().await?;

        let conflicts: Vec<ConflictInfo> = conflict_ops
            .into_iter()
            .map(|op| {
                ConflictInfo {
                    current_status_option_id: None,
                    expected_status_option_id: op.precondition.expected_from_option_id.clone(),
                    operation: op,
                }
            })
            .collect();

        let rate_limit = if let Some(ref token) = token {
            self.github.get_rate_limit(token).await.ok()
        } else {
            None
        };

        let last_sync_at = self.persistence.get_setting("last_sync_at").await?;
        let last_sync_at = last_sync_at.and_then(|s| s.parse().ok());

        let sync_state = SyncState {
            is_syncing: false,
            last_sync_at,
            rate_limit,
            error: None,
        };

        Ok(BootstrapResponse {
            projects,
            tasks,
            pending_ops,
            conflicts,
            sync_state,
        })
    }

    async fn load_from_cache(
        &self,
    ) -> Result<(Vec<ProjectDto>, Vec<TaskDto>), DomainError> {
        let projects = self.persistence.get_all_projects().await?;
        let tasks = self.persistence.get_all_tasks().await?;
        Ok((projects, tasks))
    }

    async fn refresh_from_github(
        &self,
        token: &str,
    ) -> Result<(Vec<ProjectDto>, Vec<TaskDto>), DomainError> {
        let now = chrono::Utc::now().timestamp_millis();

        let projects = self.github.fetch_accessible_projects(token).await?;

        let mut all_tasks = Vec::new();
        let mut updated_projects = Vec::new();

        for mut project in projects {
            let (project_with_details, status_field) = self
                .github
                .fetch_project_details(token, &project.id)
                .await?;

            project.title = project_with_details.title;
            project.url = project_with_details.url;
            project.updated_at = project_with_details.updated_at;
            project.status_field = status_field;
            project.synced_at = Some(now);

            self.persistence.upsert_project(&project).await?;

            let mut tasks = self.github.fetch_project_items(token, &project.id).await?;
            for task in &mut tasks {
                task.synced_at = Some(now);
            }

            self.persistence.upsert_tasks_batch(&tasks).await?;

            let active_ids: Vec<_> = tasks.iter().map(|t| t.id.clone()).collect();
            self.persistence
                .delete_stale_tasks_by_project(&project.id, &active_ids)
                .await?;

            all_tasks.extend(tasks);
            updated_projects.push(project);
        }

        self.persistence
            .set_setting("last_sync_at", &now.to_string())
            .await?;

        Ok((updated_projects, all_tasks))
    }
}
