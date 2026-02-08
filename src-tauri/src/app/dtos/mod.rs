use serde::{Deserialize, Serialize};

// ============================================================
// Enums
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OwnerType {
    #[serde(rename = "organization")]
    Organization,
    #[serde(rename = "user")]
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentType {
    Issue,
    DraftIssue,
    PullRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationType {
    MoveItemToColumn,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "syncing")]
    Syncing,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "conflict")]
    Conflict,
    #[serde(rename = "failed")]
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "syncing")]
    Syncing,
    #[serde(rename = "paused_conflict")]
    PausedConflict,
    #[serde(rename = "paused_rate_limited")]
    PausedRateLimited,
    #[serde(rename = "paused_auth_error")]
    PausedAuthError,
    #[serde(rename = "error")]
    Error,
}

// ============================================================
// DTOs
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDto {
    pub id: String,
    pub owner_type: OwnerType,
    pub owner_login: String,
    pub title: String,
    pub url: String,
    pub updated_at: Option<i64>,
    pub synced_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDto {
    pub id: String,
    pub project_id: String,
    pub content_type: ContentType,
    pub content_id: Option<String>,
    pub title: String,
    pub body: Option<String>,
    pub status_option_id: Option<String>,
    pub assignee_login: Option<String>,
    pub due_date: Option<String>,
    pub url: Option<String>,
    pub updated_at: Option<i64>,
    pub synced_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusFieldDto {
    pub id: String,
    pub project_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusOptionDto {
    pub id: String,
    pub status_field_id: String,
    pub name: String,
    pub color: Option<String>,
    pub position: i32,
}

// ============================================================
// Operation
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationPayload {
    pub item_id: String,
    pub project_id: String,
    pub status_field_id: String,
    pub to_option_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Precondition {
    pub base_item_updated_at: i64,
    pub expected_from_option_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    pub id: String,
    pub op_type: OperationType,
    pub payload: OperationPayload,
    pub precondition: Precondition,
    pub created_at: i64,
    pub status: OperationStatus,
    pub error_message: Option<String>,
    pub resolved_at: Option<i64>,
}

// ============================================================
// Bootstrap Response
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictInfo {
    pub operation: Operation,
    pub current_option_id: String,
    pub current_option_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapResponse {
    pub projects: Vec<ProjectDto>,
    pub status_fields: Vec<StatusFieldDto>,
    pub status_options: Vec<StatusOptionDto>,
    pub tasks: Vec<TaskDto>,
    pub pending_operations: Vec<Operation>,
    pub conflicts: Vec<ConflictInfo>,
    pub current_user_login: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectBootstrapResponse {
    pub project: ProjectDto,
    pub status_field: Option<StatusFieldDto>,
    pub status_options: Vec<StatusOptionDto>,
    pub tasks: Vec<TaskDto>,
    pub pending_operations: Vec<Operation>,
    pub conflicts: Vec<ConflictInfo>,
}

// ============================================================
// Sync
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncState {
    pub status: SyncStatus,
    pub pending_count: i32,
    pub conflict_count: i32,
    pub rate_limit_reset_at: Option<i64>,
    pub last_error: Option<String>,
    pub last_sync_attempt_at: Option<i64>,
    pub last_sync_success_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResultFailure {
    pub operation: Operation,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub completed: Vec<Operation>,
    pub failed: Vec<SyncResultFailure>,
    pub conflicts: Vec<ConflictInfo>,
    pub state: SyncState,
}

// ============================================================
// Input
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppendOperationInput {
    pub op_type: OperationType,
    pub item_id: String,
    pub project_id: String,
    pub status_field_id: String,
    pub to_option_id: String,
    pub base_item_updated_at: i64,
    pub expected_from_option_id: String,
}
