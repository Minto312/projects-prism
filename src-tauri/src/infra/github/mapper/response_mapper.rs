use serde_json::Value;

use crate::app::dtos::{
    ContentType, OwnerType, ProjectDto, RateLimitInfo, StatusFieldDto, StatusOptionDto, TaskDto,
};
use crate::domain::{DomainError, ProjectId, StatusFieldId, StatusOptionId, TaskId};

pub fn map_viewer_login(response: &Value) -> Result<String, DomainError> {
    response["data"]["viewer"]["login"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| DomainError::GitHubApiError("Failed to get viewer login".to_string()))
}

pub fn map_projects(response: &Value) -> Result<(Vec<ProjectDto>, Option<String>), DomainError> {
    let projects_data = &response["data"]["viewer"]["projectsV2"];

    let nodes = projects_data["nodes"]
        .as_array()
        .ok_or_else(|| DomainError::GitHubApiError("Invalid projects response".to_string()))?;

    let mut projects = Vec::new();

    for node in nodes {
        if node.is_null() {
            continue;
        }

        let id = node["id"]
            .as_str()
            .ok_or_else(|| DomainError::GitHubApiError("Missing project id".to_string()))?;

        let title = node["title"]
            .as_str()
            .ok_or_else(|| DomainError::GitHubApiError("Missing project title".to_string()))?;

        let url = node["url"]
            .as_str()
            .ok_or_else(|| DomainError::GitHubApiError("Missing project url".to_string()))?;

        let updated_at = node["updatedAt"]
            .as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp_millis());

        let owner = &node["owner"];
        let owner_type = match owner["__typename"].as_str() {
            Some("Organization") => OwnerType::Organization,
            _ => OwnerType::User,
        };
        let owner_login = owner["login"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        projects.push(ProjectDto {
            id: ProjectId::new(id),
            owner_type,
            owner_login,
            title: title.to_string(),
            url: url.to_string(),
            updated_at,
            synced_at: None,
            status_field: None,
        });
    }

    let page_info = &projects_data["pageInfo"];
    let next_cursor = if page_info["hasNextPage"].as_bool().unwrap_or(false) {
        page_info["endCursor"].as_str().map(|s| s.to_string())
    } else {
        None
    };

    Ok((projects, next_cursor))
}

pub fn map_project_details(
    response: &Value,
) -> Result<(ProjectDto, Option<StatusFieldDto>), DomainError> {
    let node = &response["data"]["node"];

    if node.is_null() {
        return Err(DomainError::NotFound("Project not found".to_string()));
    }

    let id = node["id"]
        .as_str()
        .ok_or_else(|| DomainError::GitHubApiError("Missing project id".to_string()))?;

    let title = node["title"]
        .as_str()
        .ok_or_else(|| DomainError::GitHubApiError("Missing project title".to_string()))?;

    let url = node["url"]
        .as_str()
        .ok_or_else(|| DomainError::GitHubApiError("Missing project url".to_string()))?;

    let updated_at = node["updatedAt"]
        .as_str()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.timestamp_millis());

    let owner = &node["owner"];
    let owner_type = match owner["__typename"].as_str() {
        Some("Organization") => OwnerType::Organization,
        _ => OwnerType::User,
    };
    let owner_login = owner["login"].as_str().unwrap_or("unknown").to_string();

    let project_id = ProjectId::new(id);

    let status_field = extract_status_field(node, &project_id)?;

    let project = ProjectDto {
        id: project_id.clone(),
        owner_type,
        owner_login,
        title: title.to_string(),
        url: url.to_string(),
        updated_at,
        synced_at: None,
        status_field: status_field.clone(),
    };

    Ok((project, status_field))
}

fn extract_status_field(
    node: &Value,
    project_id: &ProjectId,
) -> Result<Option<StatusFieldDto>, DomainError> {
    let fields = match node["fields"]["nodes"].as_array() {
        Some(f) => f,
        None => return Ok(None),
    };

    for field in fields {
        if field["__typename"].as_str() != Some("ProjectV2SingleSelectField") {
            continue;
        }

        let field_name = field["name"].as_str().unwrap_or("");
        if field_name.to_lowercase() != "status" {
            continue;
        }

        let field_id = field["id"]
            .as_str()
            .ok_or_else(|| DomainError::GitHubApiError("Missing field id".to_string()))?;

        let options = match field["options"].as_array() {
            Some(opts) => opts
                .iter()
                .enumerate()
                .filter_map(|(i, opt)| {
                    let opt_id = opt["id"].as_str()?;
                    let name = opt["name"].as_str()?;
                    let color = opt["color"].as_str().map(|s| s.to_string());

                    Some(StatusOptionDto {
                        id: StatusOptionId::new(opt_id),
                        status_field_id: StatusFieldId::new(field_id),
                        name: name.to_string(),
                        color,
                        position: i as i32,
                    })
                })
                .collect(),
            None => Vec::new(),
        };

        return Ok(Some(StatusFieldDto {
            id: StatusFieldId::new(field_id),
            project_id: project_id.clone(),
            name: field_name.to_string(),
            options,
        }));
    }

    Ok(None)
}

pub fn map_project_items(
    response: &Value,
    project_id: &ProjectId,
) -> Result<(Vec<TaskDto>, Option<String>), DomainError> {
    let items_data = &response["data"]["node"]["items"];

    if items_data.is_null() {
        return Ok((Vec::new(), None));
    }

    let nodes = items_data["nodes"]
        .as_array()
        .ok_or_else(|| DomainError::GitHubApiError("Invalid items response".to_string()))?;

    let mut tasks = Vec::new();

    for node in nodes {
        if node.is_null() {
            continue;
        }

        let item_id = node["id"]
            .as_str()
            .ok_or_else(|| DomainError::GitHubApiError("Missing item id".to_string()))?;

        let updated_at = node["updatedAt"]
            .as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp_millis());

        let content = &node["content"];
        if content.is_null() {
            continue;
        }

        let content_type = match content["__typename"].as_str() {
            Some("Issue") => ContentType::Issue,
            Some("PullRequest") => ContentType::PullRequest,
            Some("DraftIssue") => ContentType::DraftIssue,
            _ => continue,
        };

        let content_id = content["id"].as_str().map(|s| s.to_string());
        let title = content["title"].as_str().unwrap_or("").to_string();
        let body = content["body"].as_str().map(|s| s.to_string());
        let url = content["url"].as_str().map(|s| s.to_string());

        let assignee_login = content["assignees"]["nodes"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|a| a["login"].as_str())
            .map(|s| s.to_string());

        let (status_option_id, due_date) = extract_field_values(node);

        tasks.push(TaskDto {
            id: TaskId::new(item_id),
            project_id: project_id.clone(),
            content_type,
            content_id,
            title,
            body,
            status_option_id,
            assignee_login,
            due_date,
            url,
            updated_at,
            synced_at: None,
        });
    }

    let page_info = &items_data["pageInfo"];
    let next_cursor = if page_info["hasNextPage"].as_bool().unwrap_or(false) {
        page_info["endCursor"].as_str().map(|s| s.to_string())
    } else {
        None
    };

    Ok((tasks, next_cursor))
}

fn extract_field_values(node: &Value) -> (Option<StatusOptionId>, Option<String>) {
    let field_values = match node["fieldValues"]["nodes"].as_array() {
        Some(fv) => fv,
        None => return (None, None),
    };

    let mut status_option_id = None;
    let mut due_date = None;

    for fv in field_values {
        match fv["__typename"].as_str() {
            Some("ProjectV2ItemFieldSingleSelectValue") => {
                let field_name = fv["field"]["name"].as_str().unwrap_or("");
                if field_name.to_lowercase() == "status" {
                    if let Some(opt_id) = fv["optionId"].as_str() {
                        status_option_id = Some(StatusOptionId::new(opt_id));
                    }
                }
            }
            Some("ProjectV2ItemFieldDateValue") => {
                let field_name = fv["field"]["name"].as_str().unwrap_or("");
                if field_name.to_lowercase().contains("due") || field_name.to_lowercase() == "date"
                {
                    if let Some(date) = fv["date"].as_str() {
                        due_date = Some(date.to_string());
                    }
                }
            }
            _ => {}
        }
    }

    (status_option_id, due_date)
}

pub fn map_item_status(
    response: &Value,
    status_field_id: &StatusFieldId,
) -> Result<Option<StatusOptionId>, DomainError> {
    let field_values = &response["data"]["node"]["fieldValues"]["nodes"];

    if field_values.is_null() {
        return Ok(None);
    }

    let nodes = field_values.as_array().ok_or_else(|| {
        DomainError::GitHubApiError("Invalid field values response".to_string())
    })?;

    for node in nodes {
        if node["__typename"].as_str() != Some("ProjectV2ItemFieldSingleSelectValue") {
            continue;
        }

        let field_id = node["field"]["id"].as_str();
        if field_id == Some(status_field_id.as_str()) {
            if let Some(opt_id) = node["optionId"].as_str() {
                return Ok(Some(StatusOptionId::new(opt_id)));
            }
        }
    }

    Ok(None)
}

pub fn map_rate_limit(response: &Value) -> Result<RateLimitInfo, DomainError> {
    let rate_limit = &response["data"]["rateLimit"];

    let limit = rate_limit["limit"]
        .as_i64()
        .ok_or_else(|| DomainError::GitHubApiError("Missing rate limit".to_string()))?
        as i32;

    let remaining = rate_limit["remaining"]
        .as_i64()
        .ok_or_else(|| DomainError::GitHubApiError("Missing remaining".to_string()))?
        as i32;

    let reset_at = rate_limit["resetAt"]
        .as_str()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.timestamp_millis())
        .ok_or_else(|| DomainError::GitHubApiError("Missing reset at".to_string()))?;

    Ok(RateLimitInfo {
        limit,
        remaining,
        reset_at,
    })
}

pub fn check_graphql_errors(response: &Value) -> Result<(), DomainError> {
    if let Some(errors) = response["errors"].as_array() {
        if !errors.is_empty() {
            let error_messages: Vec<String> = errors
                .iter()
                .filter_map(|e| e["message"].as_str().map(|s| s.to_string()))
                .collect();

            let error_message = error_messages.join("; ");

            if error_message.contains("401") || error_message.to_lowercase().contains("unauthorized")
            {
                return Err(DomainError::AuthenticationError(error_message));
            }

            if error_message.to_lowercase().contains("rate limit") {
                return Err(DomainError::RateLimitExceeded(error_message));
            }

            return Err(DomainError::GitHubApiError(error_message));
        }
    }
    Ok(())
}
