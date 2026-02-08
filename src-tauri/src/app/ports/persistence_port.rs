use async_trait::async_trait;
use crate::domain::{DomainError, ProjectId, TaskId, StatusFieldId, OperationId};
use crate::app::dtos::{
    ProjectDto, TaskDto, StatusFieldDto, StatusOptionDto, Operation, OperationStatus,
};

#[async_trait]
pub trait PersistencePort: Send + Sync {
    // ============================================
    // Settings
    // ============================================

    async fn get_setting(&self, key: &str) -> Result<Option<String>, DomainError>;

    async fn set_setting(&self, key: &str, value: &str) -> Result<(), DomainError>;

    async fn delete_setting(&self, key: &str) -> Result<(), DomainError>;

    // ============================================
    // Projects
    // ============================================

    async fn get_all_projects(&self) -> Result<Vec<ProjectDto>, DomainError>;

    async fn get_project(&self, id: &ProjectId) -> Result<Option<ProjectDto>, DomainError>;

    async fn upsert_project(&self, project: &ProjectDto) -> Result<(), DomainError>;

    async fn delete_project(&self, id: &ProjectId) -> Result<(), DomainError>;

    // ============================================
    // Status Fields
    // ============================================

    async fn get_status_field(&self, project_id: &ProjectId) -> Result<Option<StatusFieldDto>, DomainError>;

    async fn upsert_status_field(&self, field: &StatusFieldDto) -> Result<(), DomainError>;

    async fn delete_status_field(&self, id: &StatusFieldId) -> Result<(), DomainError>;

    // ============================================
    // Status Options
    // ============================================

    async fn get_status_options(&self, field_id: &StatusFieldId) -> Result<Vec<StatusOptionDto>, DomainError>;

    async fn upsert_status_option(&self, option: &StatusOptionDto) -> Result<(), DomainError>;

    async fn delete_status_options_by_field(&self, field_id: &StatusFieldId) -> Result<(), DomainError>;

    // ============================================
    // Tasks
    // ============================================

    async fn get_tasks_by_project(&self, project_id: &ProjectId) -> Result<Vec<TaskDto>, DomainError>;

    async fn get_tasks_by_assignee(&self, assignee_login: &str) -> Result<Vec<TaskDto>, DomainError>;

    async fn get_all_tasks(&self) -> Result<Vec<TaskDto>, DomainError>;

    async fn get_task(&self, id: &TaskId) -> Result<Option<TaskDto>, DomainError>;

    async fn upsert_task(&self, task: &TaskDto) -> Result<(), DomainError>;

    async fn delete_task(&self, id: &TaskId) -> Result<(), DomainError>;

    async fn delete_tasks_by_project(&self, project_id: &ProjectId) -> Result<(), DomainError>;

    // ============================================
    // Operations
    // ============================================

    async fn get_pending_operations(&self) -> Result<Vec<Operation>, DomainError>;

    async fn get_pending_operations_by_project(&self, project_id: &ProjectId) -> Result<Vec<Operation>, DomainError>;

    async fn get_operation(&self, id: &OperationId) -> Result<Option<Operation>, DomainError>;

    async fn insert_operation(&self, operation: &Operation) -> Result<(), DomainError>;

    async fn update_operation_status(
        &self,
        id: &OperationId,
        status: OperationStatus,
        error_message: Option<&str>,
        resolved_at: Option<i64>,
    ) -> Result<(), DomainError>;

    async fn get_conflict_operations(&self) -> Result<Vec<Operation>, DomainError>;
}
