use async_trait::async_trait;
use crate::domain::{DomainError, ProjectId, TaskId, StatusFieldId, StatusOptionId};
use crate::app::dtos::{ProjectDto, TaskDto, StatusFieldDto, RateLimitInfo};

#[async_trait]
pub trait GitHubPort: Send + Sync {
    // ============================================
    // Authentication
    // ============================================

    /// Validate the PAT and return the authenticated user's login
    async fn validate_token(&self, token: &str) -> Result<String, DomainError>;

    // ============================================
    // Projects
    // ============================================

    /// Fetch all accessible projects for the authenticated user
    async fn fetch_accessible_projects(&self, token: &str) -> Result<Vec<ProjectDto>, DomainError>;

    /// Fetch project details including status field
    async fn fetch_project_details(
        &self,
        token: &str,
        project_id: &ProjectId,
    ) -> Result<(ProjectDto, Option<StatusFieldDto>), DomainError>;

    // ============================================
    // Items
    // ============================================

    /// Fetch all items in a project
    async fn fetch_project_items(
        &self,
        token: &str,
        project_id: &ProjectId,
    ) -> Result<Vec<TaskDto>, DomainError>;

    /// Fetch current status of an item
    async fn fetch_item_current_status(
        &self,
        token: &str,
        item_id: &TaskId,
        status_field_id: &StatusFieldId,
    ) -> Result<Option<StatusOptionId>, DomainError>;

    // ============================================
    // Mutations
    // ============================================

    /// Update item status
    async fn update_item_status(
        &self,
        token: &str,
        project_id: &ProjectId,
        item_id: &TaskId,
        status_field_id: &StatusFieldId,
        status_option_id: &StatusOptionId,
    ) -> Result<(), DomainError>;

    // ============================================
    // Rate Limit
    // ============================================

    /// Get current rate limit information
    async fn get_rate_limit(&self, token: &str) -> Result<RateLimitInfo, DomainError>;
}
