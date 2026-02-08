use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

use crate::app::dtos::{ProjectDto, RateLimitInfo, StatusFieldDto, TaskDto};
use crate::app::ports::GitHubPort;
use crate::domain::{DomainError, ProjectId, StatusFieldId, StatusOptionId, TaskId};
use crate::infra::github::api::*;
use crate::infra::github::mapper::*;

pub struct GitHubApiClient {
    client: Client,
}

impl GitHubApiClient {
    pub fn new() -> Result<Self, DomainError> {
        let client = Client::builder()
            .user_agent("Prism/0.1.0")
            .build()
            .map_err(|e| DomainError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client })
    }

    async fn execute_graphql(
        &self,
        token: &str,
        query: &str,
        variables: Option<Value>,
    ) -> Result<Value, DomainError> {
        let body = json!({
            "query": query,
            "variables": variables.unwrap_or(json!({}))
        });

        let response = self
            .client
            .post(GITHUB_GRAPHQL_ENDPOINT)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    DomainError::NetworkError("Request timeout".to_string())
                } else if e.is_connect() {
                    DomainError::NetworkError("Connection failed".to_string())
                } else {
                    DomainError::NetworkError(e.to_string())
                }
            })?;

        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(DomainError::AuthenticationError(
                "Invalid or expired token".to_string(),
            ));
        }

        if status == reqwest::StatusCode::FORBIDDEN {
            let body = response.text().await.unwrap_or_default();
            if body.to_lowercase().contains("rate limit") {
                return Err(DomainError::RateLimitExceeded(
                    "API rate limit exceeded".to_string(),
                ));
            }
            return Err(DomainError::AuthenticationError(
                "Access forbidden".to_string(),
            ));
        }

        if !status.is_success() {
            return Err(DomainError::GitHubApiError(format!(
                "HTTP error: {}",
                status
            )));
        }

        let json: Value = response.json().await.map_err(|e| {
            DomainError::GitHubApiError(format!("Failed to parse response: {}", e))
        })?;

        check_graphql_errors(&json)?;

        Ok(json)
    }
}

impl Default for GitHubApiClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTP client")
    }
}

#[async_trait]
impl GitHubPort for GitHubApiClient {
    async fn validate_token(&self, token: &str) -> Result<String, DomainError> {
        let response = self
            .execute_graphql(token, VALIDATE_TOKEN_QUERY, None)
            .await?;
        map_viewer_login(&response)
    }

    async fn fetch_accessible_projects(&self, token: &str) -> Result<Vec<ProjectDto>, DomainError> {
        let mut all_projects = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let variables = json!({ "cursor": cursor });
            let response = self
                .execute_graphql(token, FETCH_ACCESSIBLE_PROJECTS_QUERY, Some(variables))
                .await?;

            let (projects, next_cursor) = map_projects(&response)?;
            all_projects.extend(projects);

            match next_cursor {
                Some(c) => cursor = Some(c),
                None => break,
            }
        }

        Ok(all_projects)
    }

    async fn fetch_project_details(
        &self,
        token: &str,
        project_id: &ProjectId,
    ) -> Result<(ProjectDto, Option<StatusFieldDto>), DomainError> {
        let variables = json!({ "projectId": project_id.as_str() });
        let response = self
            .execute_graphql(token, FETCH_PROJECT_DETAILS_QUERY, Some(variables))
            .await?;

        map_project_details(&response)
    }

    async fn fetch_project_items(
        &self,
        token: &str,
        project_id: &ProjectId,
    ) -> Result<Vec<TaskDto>, DomainError> {
        let mut all_tasks = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let variables = json!({
                "projectId": project_id.as_str(),
                "cursor": cursor
            });
            let response = self
                .execute_graphql(token, FETCH_PROJECT_ITEMS_QUERY, Some(variables))
                .await?;

            let (tasks, next_cursor) = map_project_items(&response, project_id)?;
            all_tasks.extend(tasks);

            match next_cursor {
                Some(c) => cursor = Some(c),
                None => break,
            }
        }

        Ok(all_tasks)
    }

    async fn fetch_item_current_status(
        &self,
        token: &str,
        item_id: &TaskId,
        status_field_id: &StatusFieldId,
    ) -> Result<Option<StatusOptionId>, DomainError> {
        let variables = json!({ "itemId": item_id.as_str() });
        let response = self
            .execute_graphql(token, FETCH_ITEM_STATUS_QUERY, Some(variables))
            .await?;

        map_item_status(&response, status_field_id)
    }

    async fn update_item_status(
        &self,
        token: &str,
        project_id: &ProjectId,
        item_id: &TaskId,
        status_field_id: &StatusFieldId,
        status_option_id: &StatusOptionId,
    ) -> Result<(), DomainError> {
        let variables = json!({
            "projectId": project_id.as_str(),
            "itemId": item_id.as_str(),
            "fieldId": status_field_id.as_str(),
            "optionId": status_option_id.as_str()
        });

        self.execute_graphql(token, UPDATE_ITEM_STATUS_MUTATION, Some(variables))
            .await?;

        Ok(())
    }

    async fn get_rate_limit(&self, token: &str) -> Result<RateLimitInfo, DomainError> {
        let response = self
            .execute_graphql(token, RATE_LIMIT_QUERY, None)
            .await?;

        map_rate_limit(&response)
    }
}
