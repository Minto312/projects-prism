use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

use crate::app::dtos::ProjectDto;
use crate::app::ports::github_port::{GitHubPort, GitHubProjectData};
use crate::domain::errors::DomainError;
use crate::infra::github::api::queries;
use crate::infra::github::mapper::response_mapper;

const GITHUB_GRAPHQL_URL: &str = "https://api.github.com/graphql";

pub struct GitHubApiClient {
    client: Client,
}

impl GitHubApiClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    async fn graphql_request(
        &self,
        pat: &str,
        query: &str,
        variables: Option<Value>,
    ) -> Result<Value, DomainError> {
        let mut body = json!({ "query": query });
        if let Some(vars) = variables {
            body["variables"] = vars;
        }

        let response = self
            .client
            .post(GITHUB_GRAPHQL_URL)
            .header("Authorization", format!("bearer {}", pat))
            .header("User-Agent", "Prism/0.1.0")
            .json(&body)
            .send()
            .await
            .map_err(|e| DomainError::Network(e.to_string()))?;

        let status = response.status();

        if status.as_u16() == 401 {
            return Err(DomainError::Authentication(
                "Invalid or expired PAT".to_string(),
            ));
        }

        if status.as_u16() == 403 {
            // Rate limit の可能性をチェック
            let body: Value = response
                .json()
                .await
                .map_err(|e| DomainError::Api(e.to_string()))?;

            if let Some(msg) = body["message"].as_str() {
                if msg.contains("rate limit") || msg.contains("API rate limit") {
                    // Reset time を取得
                    let reset_at = chrono::Utc::now().timestamp_millis() + 60 * 60 * 1000; // 1時間後
                    return Err(DomainError::RateLimited { reset_at });
                }
            }

            return Err(DomainError::Api(format!("Forbidden: {:?}", body)));
        }

        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(DomainError::Api(format!(
                "HTTP {}: {}",
                status.as_u16(),
                body
            )));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| DomainError::Api(e.to_string()))?;

        // GraphQL エラーチェック
        if let Some(errors) = json["errors"].as_array() {
            if !errors.is_empty() {
                let error_msg = errors
                    .iter()
                    .filter_map(|e| e["message"].as_str())
                    .collect::<Vec<_>>()
                    .join("; ");
                return Err(DomainError::Api(error_msg));
            }
        }

        Ok(json)
    }
}

#[async_trait]
impl GitHubPort for GitHubApiClient {
    async fn validate_token(&self, pat: &str) -> Result<String, DomainError> {
        let response = self
            .graphql_request(pat, queries::VIEWER_QUERY, None)
            .await?;
        response_mapper::extract_viewer_login(&response)
    }

    async fn fetch_projects(&self, pat: &str) -> Result<Vec<ProjectDto>, DomainError> {
        let mut all_projects = Vec::new();
        let mut after: Option<String> = None;

        loop {
            let variables = json!({
                "after": after,
            });

            let response = self
                .graphql_request(pat, queries::VIEWER_PROJECTS_QUERY, Some(variables))
                .await?;

            let (projects, next_cursor) =
                response_mapper::extract_viewer_projects(&response)?;
            all_projects.extend(projects);

            if next_cursor.is_none() {
                break;
            }
            after = next_cursor;
        }

        Ok(all_projects)
    }

    async fn fetch_project_items(
        &self,
        pat: &str,
        project_id: &str,
    ) -> Result<GitHubProjectData, DomainError> {
        let mut all_tasks = Vec::new();
        let mut after: Option<String> = None;
        let mut result_data: Option<GitHubProjectData> = None;

        loop {
            let variables = json!({
                "projectId": project_id,
                "after": after,
            });

            let response = self
                .graphql_request(pat, queries::PROJECT_ITEMS_QUERY, Some(variables))
                .await?;

            let (data, next_cursor) =
                response_mapper::extract_project_items(&response, project_id)?;

            all_tasks.extend(data.tasks);

            if result_data.is_none() {
                result_data = Some(GitHubProjectData {
                    project: data.project,
                    status_field: data.status_field,
                    status_options: data.status_options,
                    tasks: Vec::new(),
                });
            }

            if next_cursor.is_none() {
                break;
            }
            after = next_cursor;
        }

        let mut data = result_data.unwrap();
        data.tasks = all_tasks;
        Ok(data)
    }

    async fn fetch_item_current_status(
        &self,
        pat: &str,
        item_id: &str,
        project_id: &str,
    ) -> Result<Option<String>, DomainError> {
        let variables = json!({
            "itemId": item_id,
            "projectId": project_id,
        });

        let response = self
            .graphql_request(pat, queries::ITEM_STATUS_QUERY, Some(variables))
            .await?;

        response_mapper::extract_item_status(&response)
    }

    async fn update_item_status(
        &self,
        pat: &str,
        project_id: &str,
        item_id: &str,
        field_id: &str,
        option_id: &str,
    ) -> Result<(), DomainError> {
        let variables = json!({
            "input": {
                "projectId": project_id,
                "itemId": item_id,
                "fieldId": field_id,
                "value": {
                    "singleSelectOptionId": option_id
                }
            }
        });

        self.graphql_request(pat, queries::UPDATE_ITEM_STATUS_MUTATION, Some(variables))
            .await?;

        Ok(())
    }

    async fn get_rate_limit(&self, pat: &str) -> Result<Option<i64>, DomainError> {
        let response = self
            .graphql_request(pat, queries::RATE_LIMIT_QUERY, None)
            .await?;
        response_mapper::extract_rate_limit_reset(&response)
    }
}
