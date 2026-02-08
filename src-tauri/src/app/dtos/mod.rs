use serde::{Deserialize, Serialize};
use crate::domain::{ProjectId, TaskId, StatusFieldId, StatusOptionId, OperationId};

// ============================================
// Project DTOs
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDto {
    pub id: ProjectId,
    pub owner_type: OwnerType,
    pub owner_login: String,
    pub title: String,
    pub url: String,
    pub updated_at: Option<i64>,
    pub synced_at: Option<i64>,
    pub status_field: Option<StatusFieldDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OwnerType {
    Organization,
    User,
}

impl OwnerType {
    pub fn as_str(&self) -> &str {
        match self {
            OwnerType::Organization => "organization",
            OwnerType::User => "user",
        }
    }
}

impl std::str::FromStr for OwnerType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "organization" => Ok(OwnerType::Organization),
            "user" => Ok(OwnerType::User),
            _ => Err(format!("Invalid owner type: {}", s)),
        }
    }
}

// ============================================
// Status Field DTOs
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusFieldDto {
    pub id: StatusFieldId,
    pub project_id: ProjectId,
    pub name: String,
    pub options: Vec<StatusOptionDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusOptionDto {
    pub id: StatusOptionId,
    pub status_field_id: StatusFieldId,
    pub name: String,
    pub color: Option<String>,
    pub position: i32,
}

// ============================================
// Task DTOs
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDto {
    pub id: TaskId,
    pub project_id: ProjectId,
    pub content_type: ContentType,
    pub content_id: Option<String>,
    pub title: String,
    pub body: Option<String>,
    pub status_option_id: Option<StatusOptionId>,
    pub assignee_login: Option<String>,
    pub due_date: Option<String>,
    pub url: Option<String>,
    pub updated_at: Option<i64>,
    pub synced_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContentType {
    Issue,
    DraftIssue,
    PullRequest,
}

impl ContentType {
    pub fn as_str(&self) -> &str {
        match self {
            ContentType::Issue => "Issue",
            ContentType::DraftIssue => "DraftIssue",
            ContentType::PullRequest => "PullRequest",
        }
    }
}

impl std::str::FromStr for ContentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Issue" => Ok(ContentType::Issue),
            "DraftIssue" => Ok(ContentType::DraftIssue),
            "PullRequest" => Ok(ContentType::PullRequest),
            _ => Err(format!("Invalid content type: {}", s)),
        }
    }
}

// ============================================
// Operation DTOs
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: OperationId,
    pub op_type: OperationType,
    pub payload: OperationPayload,
    pub precondition: Precondition,
    pub created_at: i64,
    pub status: OperationStatus,
    pub error_message: Option<String>,
    pub resolved_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperationType {
    MoveItemToColumn,
}

impl OperationType {
    pub fn as_str(&self) -> &str {
        match self {
            OperationType::MoveItemToColumn => "MoveItemToColumn",
        }
    }
}

impl std::str::FromStr for OperationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MoveItemToColumn" => Ok(OperationType::MoveItemToColumn),
            _ => Err(format!("Invalid operation type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationPayload {
    pub item_id: TaskId,
    pub project_id: ProjectId,
    pub status_field_id: StatusFieldId,
    pub to_option_id: StatusOptionId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Precondition {
    pub base_item_updated_at: i64,
    pub expected_from_option_id: StatusOptionId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OperationStatus {
    Pending,
    Syncing,
    Completed,
    Conflict,
    Failed,
}

impl OperationStatus {
    pub fn as_str(&self) -> &str {
        match self {
            OperationStatus::Pending => "pending",
            OperationStatus::Syncing => "syncing",
            OperationStatus::Completed => "completed",
            OperationStatus::Conflict => "conflict",
            OperationStatus::Failed => "failed",
        }
    }
}

impl std::str::FromStr for OperationStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(OperationStatus::Pending),
            "syncing" => Ok(OperationStatus::Syncing),
            "completed" => Ok(OperationStatus::Completed),
            "conflict" => Ok(OperationStatus::Conflict),
            "failed" => Ok(OperationStatus::Failed),
            _ => Err(format!("Invalid operation status: {}", s)),
        }
    }
}

// ============================================
// Bootstrap DTOs
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapResponse {
    pub projects: Vec<ProjectDto>,
    pub tasks: Vec<TaskDto>,
    pub pending_ops: Vec<Operation>,
    pub conflicts: Vec<ConflictInfo>,
    pub sync_state: SyncState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub operation: Operation,
    pub current_status_option_id: Option<StatusOptionId>,
    pub expected_status_option_id: StatusOptionId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub is_syncing: bool,
    pub last_sync_at: Option<i64>,
    pub rate_limit: Option<RateLimitInfo>,
    pub error: Option<String>,
}

impl Default for SyncState {
    fn default() -> Self {
        Self {
            is_syncing: false,
            last_sync_at: None,
            rate_limit: None,
            error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub limit: i32,
    pub remaining: i32,
    pub reset_at: i64,
}

// ============================================
// Sync DTOs
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub completed_count: usize,
    pub conflict_count: usize,
    pub failed_count: usize,
    pub conflicts: Vec<ConflictInfo>,
    pub sync_state: SyncState,
}

// ============================================
// Input DTOs
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendOpsInput {
    pub operations: Vec<NewOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewOperation {
    pub item_id: TaskId,
    pub project_id: ProjectId,
    pub status_field_id: StatusFieldId,
    pub to_option_id: StatusOptionId,
    pub base_item_updated_at: i64,
    pub expected_from_option_id: StatusOptionId,
}
