use std::sync::Mutex;

use rusqlite::{params, Connection};

use crate::app::dtos::{
    AppendOperationInput, ContentType, Operation, OperationPayload, OperationStatus,
    OperationType, OwnerType, Precondition, ProjectDto, StatusFieldDto, StatusOptionDto, TaskDto,
};
use crate::app::ports::persistence_port::PersistencePort;
use crate::domain::errors::DomainError;
use crate::infra::persistence::sqlite::schema;

pub struct SqlitePersistence {
    conn: Mutex<Connection>,
}

impl SqlitePersistence {
    pub fn new(db_path: &str) -> Result<Self, DomainError> {
        let conn =
            Connection::open(db_path).map_err(|e| DomainError::Persistence(e.to_string()))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| DomainError::Persistence(e.to_string()))?;

        conn.execute_batch(schema::CREATE_TABLES)
            .map_err(|e| DomainError::Persistence(e.to_string()))?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

impl PersistencePort for SqlitePersistence {
    // ========================================
    // Settings
    // ========================================

    fn get_setting(&self, key: &str) -> Result<Option<String>, DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT value FROM settings WHERE key = ?1")
            .map_err(|e| DomainError::Persistence(e.to_string()))?;
        let result = stmt
            .query_row(params![key], |row| row.get::<_, String>(0))
            .ok();
        Ok(result)
    }

    fn set_setting(&self, key: &str, value: &str) -> Result<(), DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )
        .map_err(|e| DomainError::Persistence(e.to_string()))?;
        Ok(())
    }

    fn delete_setting(&self, key: &str) -> Result<(), DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        conn.execute("DELETE FROM settings WHERE key = ?1", params![key])
            .map_err(|e| DomainError::Persistence(e.to_string()))?;
        Ok(())
    }

    // ========================================
    // Projects
    // ========================================

    fn get_all_projects(&self) -> Result<Vec<ProjectDto>, DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT id, owner_type, owner_login, title, url, updated_at, synced_at FROM projects ORDER BY title")
            .map_err(|e| DomainError::Persistence(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(ProjectDto {
                    id: row.get(0)?,
                    owner_type: parse_owner_type(&row.get::<_, String>(1)?),
                    owner_login: row.get(2)?,
                    title: row.get(3)?,
                    url: row.get(4)?,
                    updated_at: row.get(5)?,
                    synced_at: row.get(6)?,
                })
            })
            .map_err(|e| DomainError::Persistence(e.to_string()))?;

        let mut projects = Vec::new();
        for row in rows {
            projects.push(row.map_err(|e| DomainError::Persistence(e.to_string()))?);
        }
        Ok(projects)
    }

    fn get_project(&self, project_id: &str) -> Result<Option<ProjectDto>, DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT id, owner_type, owner_login, title, url, updated_at, synced_at FROM projects WHERE id = ?1")
            .map_err(|e| DomainError::Persistence(e.to_string()))?;
        let result = stmt
            .query_row(params![project_id], |row| {
                Ok(ProjectDto {
                    id: row.get(0)?,
                    owner_type: parse_owner_type(&row.get::<_, String>(1)?),
                    owner_login: row.get(2)?,
                    title: row.get(3)?,
                    url: row.get(4)?,
                    updated_at: row.get(5)?,
                    synced_at: row.get(6)?,
                })
            })
            .ok();
        Ok(result)
    }

    fn upsert_project(&self, project: &ProjectDto) -> Result<(), DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        let owner_type_str = match project.owner_type {
            OwnerType::Organization => "organization",
            OwnerType::User => "user",
        };
        conn.execute(
            "INSERT OR REPLACE INTO projects (id, owner_type, owner_login, title, url, updated_at, synced_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                project.id,
                owner_type_str,
                project.owner_login,
                project.title,
                project.url,
                project.updated_at,
                project.synced_at,
            ],
        )
        .map_err(|e| DomainError::Persistence(e.to_string()))?;
        Ok(())
    }

    fn delete_projects_not_in(&self, project_ids: &[String]) -> Result<(), DomainError> {
        if project_ids.is_empty() {
            let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
            conn.execute("DELETE FROM projects", [])
                .map_err(|e| DomainError::Persistence(e.to_string()))?;
            return Ok(());
        }

        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        let placeholders: Vec<String> = project_ids.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect();
        let sql = format!(
            "DELETE FROM projects WHERE id NOT IN ({})",
            placeholders.join(", ")
        );
        let params: Vec<&dyn rusqlite::ToSql> = project_ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
        conn.execute(&sql, params.as_slice())
            .map_err(|e| DomainError::Persistence(e.to_string()))?;
        Ok(())
    }

    // ========================================
    // Status Fields
    // ========================================

    fn get_all_status_fields(&self) -> Result<Vec<StatusFieldDto>, DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT id, project_id, name FROM status_fields")
            .map_err(|e| DomainError::Persistence(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(StatusFieldDto {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    name: row.get(2)?,
                })
            })
            .map_err(|e| DomainError::Persistence(e.to_string()))?;

        let mut fields = Vec::new();
        for row in rows {
            fields.push(row.map_err(|e| DomainError::Persistence(e.to_string()))?);
        }
        Ok(fields)
    }

    fn get_status_fields_by_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<StatusFieldDto>, DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT id, project_id, name FROM status_fields WHERE project_id = ?1")
            .map_err(|e| DomainError::Persistence(e.to_string()))?;
        let rows = stmt
            .query_map(params![project_id], |row| {
                Ok(StatusFieldDto {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    name: row.get(2)?,
                })
            })
            .map_err(|e| DomainError::Persistence(e.to_string()))?;

        let mut fields = Vec::new();
        for row in rows {
            fields.push(row.map_err(|e| DomainError::Persistence(e.to_string()))?);
        }
        Ok(fields)
    }

    fn upsert_status_field(&self, field: &StatusFieldDto) -> Result<(), DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO status_fields (id, project_id, name) VALUES (?1, ?2, ?3)",
            params![field.id, field.project_id, field.name],
        )
        .map_err(|e| DomainError::Persistence(e.to_string()))?;
        Ok(())
    }

    fn delete_status_fields_by_project(&self, project_id: &str) -> Result<(), DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        conn.execute(
            "DELETE FROM status_fields WHERE project_id = ?1",
            params![project_id],
        )
        .map_err(|e| DomainError::Persistence(e.to_string()))?;
        Ok(())
    }

    // ========================================
    // Status Options
    // ========================================

    fn get_all_status_options(&self) -> Result<Vec<StatusOptionDto>, DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT id, status_field_id, name, color, position FROM status_options ORDER BY position")
            .map_err(|e| DomainError::Persistence(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(StatusOptionDto {
                    id: row.get(0)?,
                    status_field_id: row.get(1)?,
                    name: row.get(2)?,
                    color: row.get(3)?,
                    position: row.get(4)?,
                })
            })
            .map_err(|e| DomainError::Persistence(e.to_string()))?;

        let mut options = Vec::new();
        for row in rows {
            options.push(row.map_err(|e| DomainError::Persistence(e.to_string()))?);
        }
        Ok(options)
    }

    fn get_status_options_by_field(
        &self,
        field_id: &str,
    ) -> Result<Vec<StatusOptionDto>, DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT id, status_field_id, name, color, position FROM status_options WHERE status_field_id = ?1 ORDER BY position")
            .map_err(|e| DomainError::Persistence(e.to_string()))?;
        let rows = stmt
            .query_map(params![field_id], |row| {
                Ok(StatusOptionDto {
                    id: row.get(0)?,
                    status_field_id: row.get(1)?,
                    name: row.get(2)?,
                    color: row.get(3)?,
                    position: row.get(4)?,
                })
            })
            .map_err(|e| DomainError::Persistence(e.to_string()))?;

        let mut options = Vec::new();
        for row in rows {
            options.push(row.map_err(|e| DomainError::Persistence(e.to_string()))?);
        }
        Ok(options)
    }

    fn upsert_status_option(&self, option: &StatusOptionDto) -> Result<(), DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO status_options (id, status_field_id, name, color, position) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                option.id,
                option.status_field_id,
                option.name,
                option.color,
                option.position,
            ],
        )
        .map_err(|e| DomainError::Persistence(e.to_string()))?;
        Ok(())
    }

    fn delete_status_options_by_field(&self, field_id: &str) -> Result<(), DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        conn.execute(
            "DELETE FROM status_options WHERE status_field_id = ?1",
            params![field_id],
        )
        .map_err(|e| DomainError::Persistence(e.to_string()))?;
        Ok(())
    }

    // ========================================
    // Tasks
    // ========================================

    fn get_all_tasks(&self) -> Result<Vec<TaskDto>, DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT id, project_id, content_type, content_id, title, body, status_option_id, assignee_login, due_date, url, updated_at, synced_at FROM tasks")
            .map_err(|e| DomainError::Persistence(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(TaskDto {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    content_type: parse_content_type(&row.get::<_, String>(2)?),
                    content_id: row.get(3)?,
                    title: row.get(4)?,
                    body: row.get(5)?,
                    status_option_id: row.get(6)?,
                    assignee_login: row.get(7)?,
                    due_date: row.get(8)?,
                    url: row.get(9)?,
                    updated_at: row.get(10)?,
                    synced_at: row.get(11)?,
                })
            })
            .map_err(|e| DomainError::Persistence(e.to_string()))?;

        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(row.map_err(|e| DomainError::Persistence(e.to_string()))?);
        }
        Ok(tasks)
    }

    fn get_tasks_by_project(&self, project_id: &str) -> Result<Vec<TaskDto>, DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT id, project_id, content_type, content_id, title, body, status_option_id, assignee_login, due_date, url, updated_at, synced_at FROM tasks WHERE project_id = ?1")
            .map_err(|e| DomainError::Persistence(e.to_string()))?;
        let rows = stmt
            .query_map(params![project_id], |row| {
                Ok(TaskDto {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    content_type: parse_content_type(&row.get::<_, String>(2)?),
                    content_id: row.get(3)?,
                    title: row.get(4)?,
                    body: row.get(5)?,
                    status_option_id: row.get(6)?,
                    assignee_login: row.get(7)?,
                    due_date: row.get(8)?,
                    url: row.get(9)?,
                    updated_at: row.get(10)?,
                    synced_at: row.get(11)?,
                })
            })
            .map_err(|e| DomainError::Persistence(e.to_string()))?;

        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(row.map_err(|e| DomainError::Persistence(e.to_string()))?);
        }
        Ok(tasks)
    }

    fn upsert_task(&self, task: &TaskDto) -> Result<(), DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        let content_type_str = match task.content_type {
            ContentType::Issue => "Issue",
            ContentType::DraftIssue => "DraftIssue",
            ContentType::PullRequest => "PullRequest",
        };
        conn.execute(
            "INSERT OR REPLACE INTO tasks (id, project_id, content_type, content_id, title, body, status_option_id, assignee_login, due_date, url, updated_at, synced_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                task.id,
                task.project_id,
                content_type_str,
                task.content_id,
                task.title,
                task.body,
                task.status_option_id,
                task.assignee_login,
                task.due_date,
                task.url,
                task.updated_at,
                task.synced_at,
            ],
        )
        .map_err(|e| DomainError::Persistence(e.to_string()))?;
        Ok(())
    }

    fn delete_tasks_by_project(&self, project_id: &str) -> Result<(), DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        conn.execute(
            "DELETE FROM tasks WHERE project_id = ?1",
            params![project_id],
        )
        .map_err(|e| DomainError::Persistence(e.to_string()))?;
        Ok(())
    }

    // ========================================
    // Operations
    // ========================================

    fn get_all_operations(&self) -> Result<Vec<Operation>, DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT id, op_type, item_id, project_id, status_field_id, to_option_id, base_item_updated_at, expected_from_option_id, created_at, status, error_message, resolved_at FROM operations ORDER BY created_at")
            .map_err(|e| DomainError::Persistence(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| row_to_operation(row))
            .map_err(|e| DomainError::Persistence(e.to_string()))?;

        let mut operations = Vec::new();
        for row in rows {
            operations.push(row.map_err(|e| DomainError::Persistence(e.to_string()))?);
        }
        Ok(operations)
    }

    fn get_operations_by_status(
        &self,
        status: &OperationStatus,
    ) -> Result<Vec<Operation>, DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        let status_str = operation_status_to_str(status);
        let mut stmt = conn
            .prepare("SELECT id, op_type, item_id, project_id, status_field_id, to_option_id, base_item_updated_at, expected_from_option_id, created_at, status, error_message, resolved_at FROM operations WHERE status = ?1 ORDER BY created_at")
            .map_err(|e| DomainError::Persistence(e.to_string()))?;
        let rows = stmt
            .query_map(params![status_str], |row| row_to_operation(row))
            .map_err(|e| DomainError::Persistence(e.to_string()))?;

        let mut operations = Vec::new();
        for row in rows {
            operations.push(row.map_err(|e| DomainError::Persistence(e.to_string()))?);
        }
        Ok(operations)
    }

    fn get_operations_by_project_and_status(
        &self,
        project_id: &str,
        statuses: &[OperationStatus],
    ) -> Result<Vec<Operation>, DomainError> {
        if statuses.is_empty() {
            return Ok(Vec::new());
        }

        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        let status_strs: Vec<&str> = statuses.iter().map(operation_status_to_str).collect();
        let placeholders: Vec<String> = (0..status_strs.len())
            .map(|i| format!("?{}", i + 2))
            .collect();
        let sql = format!(
            "SELECT id, op_type, item_id, project_id, status_field_id, to_option_id, base_item_updated_at, expected_from_option_id, created_at, status, error_message, resolved_at FROM operations WHERE project_id = ?1 AND status IN ({}) ORDER BY created_at",
            placeholders.join(", ")
        );

        let mut all_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        all_params.push(Box::new(project_id.to_string()));
        for s in &status_strs {
            all_params.push(Box::new(s.to_string()));
        }
        let params_ref: Vec<&dyn rusqlite::ToSql> = all_params.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| DomainError::Persistence(e.to_string()))?;
        let rows = stmt
            .query_map(params_ref.as_slice(), |row| row_to_operation(row))
            .map_err(|e| DomainError::Persistence(e.to_string()))?;

        let mut operations = Vec::new();
        for row in rows {
            operations.push(row.map_err(|e| DomainError::Persistence(e.to_string()))?);
        }
        Ok(operations)
    }

    fn get_operation(&self, operation_id: &str) -> Result<Option<Operation>, DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT id, op_type, item_id, project_id, status_field_id, to_option_id, base_item_updated_at, expected_from_option_id, created_at, status, error_message, resolved_at FROM operations WHERE id = ?1")
            .map_err(|e| DomainError::Persistence(e.to_string()))?;
        let result = stmt
            .query_row(params![operation_id], |row| row_to_operation(row))
            .ok();
        Ok(result)
    }

    fn insert_operation(&self, input: &AppendOperationInput) -> Result<Operation, DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp_millis();
        let op_type_str = "MoveItemToColumn";
        let status_str = "pending";

        conn.execute(
            "INSERT INTO operations (id, op_type, item_id, project_id, status_field_id, to_option_id, base_item_updated_at, expected_from_option_id, created_at, status) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                id,
                op_type_str,
                input.item_id,
                input.project_id,
                input.status_field_id,
                input.to_option_id,
                input.base_item_updated_at,
                input.expected_from_option_id,
                now,
                status_str,
            ],
        )
        .map_err(|e| DomainError::Persistence(e.to_string()))?;

        Ok(Operation {
            id,
            op_type: OperationType::MoveItemToColumn,
            payload: OperationPayload {
                item_id: input.item_id.clone(),
                project_id: input.project_id.clone(),
                status_field_id: input.status_field_id.clone(),
                to_option_id: input.to_option_id.clone(),
            },
            precondition: Precondition {
                base_item_updated_at: input.base_item_updated_at,
                expected_from_option_id: input.expected_from_option_id.clone(),
            },
            created_at: now,
            status: OperationStatus::Pending,
            error_message: None,
            resolved_at: None,
        })
    }

    fn update_operation_status(
        &self,
        operation_id: &str,
        status: &OperationStatus,
        error_message: Option<&str>,
    ) -> Result<(), DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        let status_str = operation_status_to_str(status);
        let resolved_at = match status {
            OperationStatus::Completed | OperationStatus::Failed => {
                Some(chrono::Utc::now().timestamp_millis())
            }
            _ => None,
        };

        conn.execute(
            "UPDATE operations SET status = ?1, error_message = ?2, resolved_at = ?3 WHERE id = ?4",
            params![status_str, error_message, resolved_at, operation_id],
        )
        .map_err(|e| DomainError::Persistence(e.to_string()))?;
        Ok(())
    }

    fn update_operation_precondition(
        &self,
        operation_id: &str,
        expected_from_option_id: &str,
    ) -> Result<(), DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        conn.execute(
            "UPDATE operations SET expected_from_option_id = ?1 WHERE id = ?2",
            params![expected_from_option_id, operation_id],
        )
        .map_err(|e| DomainError::Persistence(e.to_string()))?;
        Ok(())
    }

    fn delete_operation(&self, operation_id: &str) -> Result<(), DomainError> {
        let conn = self.conn.lock().map_err(|e| DomainError::Persistence(e.to_string()))?;
        conn.execute(
            "DELETE FROM operations WHERE id = ?1",
            params![operation_id],
        )
        .map_err(|e| DomainError::Persistence(e.to_string()))?;
        Ok(())
    }
}

// ============================================================
// Internal helpers
// ============================================================

fn row_to_operation(row: &rusqlite::Row<'_>) -> rusqlite::Result<Operation> {
    Ok(Operation {
        id: row.get(0)?,
        op_type: OperationType::MoveItemToColumn,
        payload: OperationPayload {
            item_id: row.get(2)?,
            project_id: row.get(3)?,
            status_field_id: row.get(4)?,
            to_option_id: row.get(5)?,
        },
        precondition: Precondition {
            base_item_updated_at: row.get(6)?,
            expected_from_option_id: row.get(7)?,
        },
        created_at: row.get(8)?,
        status: parse_operation_status(&row.get::<_, String>(9)?),
        error_message: row.get(10)?,
        resolved_at: row.get(11)?,
    })
}

fn parse_owner_type(s: &str) -> OwnerType {
    match s {
        "organization" => OwnerType::Organization,
        _ => OwnerType::User,
    }
}

fn parse_content_type(s: &str) -> ContentType {
    match s {
        "Issue" => ContentType::Issue,
        "DraftIssue" => ContentType::DraftIssue,
        "PullRequest" => ContentType::PullRequest,
        _ => ContentType::Issue,
    }
}

fn parse_operation_status(s: &str) -> OperationStatus {
    match s {
        "pending" => OperationStatus::Pending,
        "syncing" => OperationStatus::Syncing,
        "completed" => OperationStatus::Completed,
        "conflict" => OperationStatus::Conflict,
        "failed" => OperationStatus::Failed,
        _ => OperationStatus::Pending,
    }
}

fn operation_status_to_str(status: &OperationStatus) -> &'static str {
    match status {
        OperationStatus::Pending => "pending",
        OperationStatus::Syncing => "syncing",
        OperationStatus::Completed => "completed",
        OperationStatus::Conflict => "conflict",
        OperationStatus::Failed => "failed",
    }
}
