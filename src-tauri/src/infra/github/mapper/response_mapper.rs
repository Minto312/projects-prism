use serde_json::Value;

use crate::app::dtos::{
    ContentType, OwnerType, ProjectDto, StatusFieldDto, StatusOptionDto, TaskDto,
};
use crate::app::ports::github_port::GitHubProjectData;
use crate::domain::errors::DomainError;

/// viewer クエリからログイン名を抽出
pub fn extract_viewer_login(response: &Value) -> Result<String, DomainError> {
    response["data"]["viewer"]["login"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| DomainError::Api("Failed to extract viewer login".to_string()))
}

/// viewer projects クエリからプロジェクト一覧を抽出
pub fn extract_viewer_projects(response: &Value) -> Result<(Vec<ProjectDto>, Option<String>), DomainError> {
    let projects_data = &response["data"]["viewer"]["projectsV2"];
    let nodes = projects_data["nodes"]
        .as_array()
        .ok_or_else(|| DomainError::Api("Failed to parse projects nodes".to_string()))?;

    let mut projects = Vec::new();
    let now = chrono::Utc::now().timestamp_millis();

    for node in nodes {
        let id = node["id"].as_str().unwrap_or_default().to_string();
        let title = node["title"].as_str().unwrap_or_default().to_string();
        let url = node["url"].as_str().unwrap_or_default().to_string();
        let updated_at = parse_datetime_to_epoch_ms(node["updatedAt"].as_str());

        let owner = &node["owner"];
        let (owner_type, owner_login) = extract_owner(owner);

        projects.push(ProjectDto {
            id,
            owner_type,
            owner_login,
            title,
            url,
            updated_at,
            synced_at: Some(now),
        });
    }

    let has_next = projects_data["pageInfo"]["hasNextPage"]
        .as_bool()
        .unwrap_or(false);
    let end_cursor = if has_next {
        projects_data["pageInfo"]["endCursor"]
            .as_str()
            .map(|s| s.to_string())
    } else {
        None
    };

    Ok((projects, end_cursor))
}

/// viewer organizations クエリから Organization ログイン名一覧を抽出
pub fn extract_viewer_organizations(response: &Value) -> Result<(Vec<String>, Option<String>), DomainError> {
    let orgs_data = &response["data"]["viewer"]["organizations"];
    let nodes = orgs_data["nodes"]
        .as_array()
        .ok_or_else(|| DomainError::Api("Failed to parse organizations nodes".to_string()))?;

    let logins: Vec<String> = nodes
        .iter()
        .filter_map(|node| node["login"].as_str().map(|s| s.to_string()))
        .collect();

    let has_next = orgs_data["pageInfo"]["hasNextPage"]
        .as_bool()
        .unwrap_or(false);
    let end_cursor = if has_next {
        orgs_data["pageInfo"]["endCursor"]
            .as_str()
            .map(|s| s.to_string())
    } else {
        None
    };

    Ok((logins, end_cursor))
}

/// organization projects クエリからプロジェクト一覧を抽出
pub fn extract_org_projects(response: &Value, org_login: &str) -> Result<(Vec<ProjectDto>, Option<String>), DomainError> {
    let projects_data = &response["data"]["organization"]["projectsV2"];
    let nodes = projects_data["nodes"]
        .as_array()
        .ok_or_else(|| DomainError::Api(format!("Failed to parse org projects nodes for {}", org_login)))?;

    let mut projects = Vec::new();
    let now = chrono::Utc::now().timestamp_millis();

    for node in nodes {
        let id = node["id"].as_str().unwrap_or_default().to_string();
        let title = node["title"].as_str().unwrap_or_default().to_string();
        let url = node["url"].as_str().unwrap_or_default().to_string();
        let updated_at = parse_datetime_to_epoch_ms(node["updatedAt"].as_str());

        projects.push(ProjectDto {
            id,
            owner_type: OwnerType::Organization,
            owner_login: org_login.to_string(),
            title,
            url,
            updated_at,
            synced_at: Some(now),
        });
    }

    let has_next = projects_data["pageInfo"]["hasNextPage"]
        .as_bool()
        .unwrap_or(false);
    let end_cursor = if has_next {
        projects_data["pageInfo"]["endCursor"]
            .as_str()
            .map(|s| s.to_string())
    } else {
        None
    };

    Ok((projects, end_cursor))
}

/// プロジェクトアイテムクエリからデータを抽出
pub fn extract_project_items(
    response: &Value,
    project_id: &str,
) -> Result<(GitHubProjectData, Option<String>), DomainError> {
    let project_node = &response["data"]["node"];

    let title = project_node["title"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let url = project_node["url"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let updated_at = parse_datetime_to_epoch_ms(project_node["updatedAt"].as_str());
    let (owner_type, owner_login) = extract_owner(&project_node["owner"]);

    let now = chrono::Utc::now().timestamp_millis();
    let project = ProjectDto {
        id: project_id.to_string(),
        owner_type,
        owner_login,
        title,
        url,
        updated_at,
        synced_at: Some(now),
    };

    // Status フィールドの抽出
    let mut status_field: Option<StatusFieldDto> = None;
    let mut status_options: Vec<StatusOptionDto> = Vec::new();

    if let Some(field_nodes) = project_node["fields"]["nodes"].as_array() {
        for field_node in field_nodes {
            // SingleSelectField のみ（name が "Status" のもの）
            if let Some(field_id) = field_node["id"].as_str() {
                let field_name = field_node["name"].as_str().unwrap_or_default();
                if field_name == "Status" {
                    status_field = Some(StatusFieldDto {
                        id: field_id.to_string(),
                        project_id: project_id.to_string(),
                        name: field_name.to_string(),
                    });

                    if let Some(options) = field_node["options"].as_array() {
                        for (pos, opt) in options.iter().enumerate() {
                            let opt_id = opt["id"].as_str().unwrap_or_default().to_string();
                            let opt_name = opt["name"].as_str().unwrap_or_default().to_string();
                            let opt_color = opt["color"].as_str().map(|s| s.to_string());

                            status_options.push(StatusOptionDto {
                                id: opt_id,
                                status_field_id: field_id.to_string(),
                                name: opt_name,
                                color: opt_color,
                                position: pos as i32,
                            });
                        }
                    }
                    break;
                }
            }
        }
    }

    // タスクの抽出
    let items_data = &project_node["items"];
    let item_nodes = items_data["nodes"]
        .as_array()
        .ok_or_else(|| DomainError::Api("Failed to parse items nodes".to_string()))?;

    let mut tasks = Vec::new();

    for item_node in item_nodes {
        let item_id = item_node["id"].as_str().unwrap_or_default().to_string();
        let item_updated_at = parse_datetime_to_epoch_ms(item_node["updatedAt"].as_str());

        let status_option_id = item_node["fieldValueByName"]["optionId"]
            .as_str()
            .map(|s| s.to_string());

        let content = &item_node["content"];
        let typename = content["__typename"].as_str().unwrap_or_default();

        let (content_type, content_id, task_title, body, assignee_login, url, due_date) =
            match typename {
                "Issue" => {
                    let cid = content["id"].as_str().map(|s| s.to_string());
                    let t = content["title"].as_str().unwrap_or_default().to_string();
                    let b = content["body"].as_str().map(|s| s.to_string());
                    let a = extract_first_assignee(content);
                    let u = content["url"].as_str().map(|s| s.to_string());
                    let d = extract_due_date_from_labels(content);
                    (ContentType::Issue, cid, t, b, a, u, d)
                }
                "DraftIssue" => {
                    let t = content["title"].as_str().unwrap_or_default().to_string();
                    let b = content["body"].as_str().map(|s| s.to_string());
                    let a = extract_first_assignee(content);
                    (ContentType::DraftIssue, None, t, b, a, None, None)
                }
                "PullRequest" => {
                    let cid = content["id"].as_str().map(|s| s.to_string());
                    let t = content["title"].as_str().unwrap_or_default().to_string();
                    let b = content["body"].as_str().map(|s| s.to_string());
                    let a = extract_first_assignee(content);
                    let u = content["url"].as_str().map(|s| s.to_string());
                    (ContentType::PullRequest, cid, t, b, a, u, None)
                }
                _ => continue,
            };

        tasks.push(TaskDto {
            id: item_id,
            project_id: project_id.to_string(),
            content_type,
            content_id,
            title: task_title,
            body,
            status_option_id,
            assignee_login,
            due_date,
            url,
            updated_at: item_updated_at,
            synced_at: Some(now),
        });
    }

    let has_next = items_data["pageInfo"]["hasNextPage"]
        .as_bool()
        .unwrap_or(false);
    let end_cursor = if has_next {
        items_data["pageInfo"]["endCursor"]
            .as_str()
            .map(|s| s.to_string())
    } else {
        None
    };

    Ok((
        GitHubProjectData {
            project,
            status_field,
            status_options,
            tasks,
        },
        end_cursor,
    ))
}

/// 単一アイテムの Status option ID を抽出
pub fn extract_item_status(response: &Value) -> Result<Option<String>, DomainError> {
    let node = &response["data"]["node"];
    let option_id = node["fieldValueByName"]["optionId"]
        .as_str()
        .map(|s| s.to_string());
    Ok(option_id)
}

/// Rate Limit 情報を抽出
pub fn extract_rate_limit_reset(response: &Value) -> Result<Option<i64>, DomainError> {
    let reset_at_str = response["data"]["rateLimit"]["resetAt"].as_str();
    Ok(reset_at_str.and_then(|s| {
        chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.timestamp_millis())
    }))
}

// ============================================================
// Internal helpers
// ============================================================

fn extract_owner(owner: &Value) -> (OwnerType, String) {
    let typename = owner["__typename"].as_str().unwrap_or("User");
    let login = owner["login"].as_str().unwrap_or_default().to_string();
    let owner_type = match typename {
        "Organization" => OwnerType::Organization,
        _ => OwnerType::User,
    };
    (owner_type, login)
}

fn extract_first_assignee(content: &Value) -> Option<String> {
    content["assignees"]["nodes"]
        .as_array()
        .and_then(|nodes| nodes.first())
        .and_then(|node| node["login"].as_str())
        .map(|s| s.to_string())
}

fn extract_due_date_from_labels(_content: &Value) -> Option<String> {
    // MVP: due date は label からは抽出しない
    // GitHub Projects V2 の Date フィールドから取得する場合は
    // クエリの拡張が必要
    None
}

fn parse_datetime_to_epoch_ms(datetime_str: Option<&str>) -> Option<i64> {
    datetime_str.and_then(|s| {
        chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.timestamp_millis())
    })
}
