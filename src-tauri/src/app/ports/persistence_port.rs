use crate::app::dtos::{
    AppendOperationInput, Operation, OperationStatus, ProjectDto, StatusFieldDto, StatusOptionDto,
    TaskDto,
};
use crate::domain::errors::DomainError;

pub trait PersistencePort: Send + Sync {
    // ========================================
    // Transaction
    // ========================================
    fn begin_transaction(&self) -> Result<(), DomainError>;
    fn commit_transaction(&self) -> Result<(), DomainError>;
    fn rollback_transaction(&self) -> Result<(), DomainError>;

    // ========================================
    // Settings
    // ========================================
    fn get_setting(&self, key: &str) -> Result<Option<String>, DomainError>;
    fn set_setting(&self, key: &str, value: &str) -> Result<(), DomainError>;
    fn delete_setting(&self, key: &str) -> Result<(), DomainError>;

    // ========================================
    // Projects
    // ========================================
    fn get_all_projects(&self) -> Result<Vec<ProjectDto>, DomainError>;
    fn get_project(&self, project_id: &str) -> Result<Option<ProjectDto>, DomainError>;
    fn upsert_project(&self, project: &ProjectDto) -> Result<(), DomainError>;
    fn delete_projects_not_in(&self, project_ids: &[String]) -> Result<(), DomainError>;

    // ========================================
    // Status Fields
    // ========================================
    fn get_all_status_fields(&self) -> Result<Vec<StatusFieldDto>, DomainError>;
    fn get_status_fields_by_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<StatusFieldDto>, DomainError>;
    fn upsert_status_field(&self, field: &StatusFieldDto) -> Result<(), DomainError>;
    fn delete_status_fields_by_project(&self, project_id: &str) -> Result<(), DomainError>;

    // ========================================
    // Status Options
    // ========================================
    fn get_all_status_options(&self) -> Result<Vec<StatusOptionDto>, DomainError>;
    fn get_status_options_by_field(
        &self,
        field_id: &str,
    ) -> Result<Vec<StatusOptionDto>, DomainError>;
    fn upsert_status_option(&self, option: &StatusOptionDto) -> Result<(), DomainError>;
    fn delete_status_options_by_field(&self, field_id: &str) -> Result<(), DomainError>;

    // ========================================
    // Tasks
    // ========================================
    fn get_all_tasks(&self) -> Result<Vec<TaskDto>, DomainError>;
    fn get_tasks_by_project(&self, project_id: &str) -> Result<Vec<TaskDto>, DomainError>;
    fn upsert_task(&self, task: &TaskDto) -> Result<(), DomainError>;
    fn delete_tasks_by_project(&self, project_id: &str) -> Result<(), DomainError>;

    // ========================================
    // Operations
    // ========================================
    fn get_all_operations(&self) -> Result<Vec<Operation>, DomainError>;
    fn get_operations_by_status(
        &self,
        status: &OperationStatus,
    ) -> Result<Vec<Operation>, DomainError>;
    fn get_operations_by_project_and_status(
        &self,
        project_id: &str,
        statuses: &[OperationStatus],
    ) -> Result<Vec<Operation>, DomainError>;
    fn get_operation(&self, operation_id: &str) -> Result<Option<Operation>, DomainError>;
    fn insert_operation(&self, input: &AppendOperationInput) -> Result<Operation, DomainError>;
    fn update_operation_status(
        &self,
        operation_id: &str,
        status: &OperationStatus,
        error_message: Option<&str>,
    ) -> Result<(), DomainError>;
    fn update_operation_precondition(
        &self,
        operation_id: &str,
        expected_from_option_id: &str,
    ) -> Result<(), DomainError>;
    fn delete_operation(&self, operation_id: &str) -> Result<(), DomainError>;
}
