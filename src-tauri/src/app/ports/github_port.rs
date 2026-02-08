use async_trait::async_trait;

use crate::app::dtos::{ProjectDto, StatusFieldDto, StatusOptionDto, TaskDto};
use crate::domain::errors::DomainError;

#[derive(Debug, Clone)]
pub struct GitHubProjectData {
    pub project: ProjectDto,
    pub status_field: Option<StatusFieldDto>,
    pub status_options: Vec<StatusOptionDto>,
    pub tasks: Vec<TaskDto>,
}

#[async_trait]
pub trait GitHubPort: Send + Sync {
    /// PAT の有効性を検証し、ログインユーザー名を返す
    async fn validate_token(&self, pat: &str) -> Result<String, DomainError>;

    /// ユーザーがアクセス可能な全プロジェクトを取得
    async fn fetch_projects(&self, pat: &str) -> Result<Vec<ProjectDto>, DomainError>;

    /// 指定プロジェクトのアイテム（タスク）と Status フィールド情報を取得
    async fn fetch_project_items(
        &self,
        pat: &str,
        project_id: &str,
    ) -> Result<GitHubProjectData, DomainError>;

    /// 指定アイテムの現在の Status option ID を取得
    async fn fetch_item_current_status(
        &self,
        pat: &str,
        item_id: &str,
        project_id: &str,
    ) -> Result<Option<String>, DomainError>;

    /// アイテムの Status フィールド値を更新
    async fn update_item_status(
        &self,
        pat: &str,
        project_id: &str,
        item_id: &str,
        field_id: &str,
        option_id: &str,
    ) -> Result<(), DomainError>;

    /// 現在のレート制限情報を取得（リセット時刻を返す）
    async fn get_rate_limit(&self, pat: &str) -> Result<Option<i64>, DomainError>;
}
