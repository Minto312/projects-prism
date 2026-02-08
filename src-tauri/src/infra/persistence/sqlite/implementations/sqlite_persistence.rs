use async_trait::async_trait;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Mutex;

use std::collections::HashMap;

use crate::app::dtos::{
    ContentType, Operation, OperationPayload, OperationStatus, OperationType, OwnerType,
    Precondition, ProjectDto, StatusFieldDto, StatusOptionDto, TaskDto,
};
use crate::app::ports::PersistencePort;
use crate::domain::{DomainError, OperationId, ProjectId, StatusFieldId, StatusOptionId, TaskId};
use crate::infra::persistence::sqlite::schema::SCHEMA;

pub struct SqlitePersistence {
    conn: Mutex<Connection>,
}

impl SqlitePersistence {
    pub fn new(db_path: &Path) -> Result<Self, DomainError> {
        let conn = Connection::open(db_path)
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        conn.execute_batch(SCHEMA)
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        // Recover from poison: if a previous operation panicked while holding the lock,
        // the connection is still usable since SQLite auto-rollbacks incomplete transactions.
        // MutexGuard must NOT be held across .await points.
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }

    // ============================================
    // Internal helpers (take &Connection to avoid deadlocks)
    // ============================================

    fn get_status_options_with(
        conn: &Connection,
        field_id: &StatusFieldId,
    ) -> Result<Vec<StatusOptionDto>, DomainError> {
        let mut stmt = conn
            .prepare(
                "SELECT id, status_field_id, name, color, position FROM status_options WHERE status_field_id = ?1 ORDER BY position",
            )
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        let rows = stmt
            .query_map(params![field_id.as_str()], |row| {
                let id: String = row.get(0)?;
                let field_id: String = row.get(1)?;
                let name: String = row.get(2)?;
                let color: Option<String> = row.get(3)?;
                let position: i32 = row.get(4)?;

                Ok(StatusOptionDto {
                    id: StatusOptionId::new(id),
                    status_field_id: StatusFieldId::new(field_id),
                    name,
                    color,
                    position,
                })
            })
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        let mut options = Vec::new();
        for row in rows {
            options.push(row.map_err(|e| DomainError::PersistenceError(e.to_string()))?);
        }
        Ok(options)
    }

    fn get_status_field_with(
        conn: &Connection,
        project_id: &ProjectId,
    ) -> Result<Option<StatusFieldDto>, DomainError> {
        let result = conn
            .query_row(
                "SELECT id, project_id, name FROM status_fields WHERE project_id = ?1",
                params![project_id.as_str()],
                |row| {
                    let id: String = row.get(0)?;
                    let proj_id: String = row.get(1)?;
                    let name: String = row.get(2)?;
                    Ok((id, proj_id, name))
                },
            )
            .optional()
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        if let Some((id, proj_id, name)) = result {
            let field_id = StatusFieldId::new(&id);
            let options = Self::get_status_options_with(conn, &field_id)?;
            Ok(Some(StatusFieldDto {
                id: field_id,
                project_id: ProjectId::new(proj_id),
                name,
                options,
            }))
        } else {
            Ok(None)
        }
    }

    fn upsert_status_option_with(
        conn: &Connection,
        option: &StatusOptionDto,
    ) -> Result<(), DomainError> {
        conn.execute(
            r#"
            INSERT INTO status_options (id, status_field_id, name, color, position)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(id) DO UPDATE SET
                status_field_id = excluded.status_field_id,
                name = excluded.name,
                color = excluded.color,
                position = excluded.position
            "#,
            params![
                option.id.as_str(),
                option.status_field_id.as_str(),
                option.name,
                option.color,
                option.position,
            ],
        )
        .map_err(|e| DomainError::PersistenceError(e.to_string()))?;
        Ok(())
    }

    fn delete_status_options_by_field_with(
        conn: &Connection,
        field_id: &StatusFieldId,
    ) -> Result<(), DomainError> {
        conn.execute(
            "DELETE FROM status_options WHERE status_field_id = ?1",
            params![field_id.as_str()],
        )
        .map_err(|e| DomainError::PersistenceError(e.to_string()))?;
        Ok(())
    }

    fn upsert_status_field_with(
        conn: &Connection,
        field: &StatusFieldDto,
    ) -> Result<(), DomainError> {
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        tx.execute(
            r#"
            INSERT INTO status_fields (id, project_id, name)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(id) DO UPDATE SET
                project_id = excluded.project_id,
                name = excluded.name
            "#,
            params![field.id.as_str(), field.project_id.as_str(), field.name,],
        )
        .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        tx.execute(
            "DELETE FROM status_options WHERE status_field_id = ?1",
            params![field.id.as_str()],
        )
        .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        for option in &field.options {
            tx.execute(
                r#"
                INSERT INTO status_options (id, status_field_id, name, color, position)
                VALUES (?1, ?2, ?3, ?4, ?5)
                ON CONFLICT(id) DO UPDATE SET
                    status_field_id = excluded.status_field_id,
                    name = excluded.name,
                    color = excluded.color,
                    position = excluded.position
                "#,
                params![
                    option.id.as_str(),
                    option.status_field_id.as_str(),
                    option.name,
                    option.color,
                    option.position,
                ],
            )
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;
        }

        tx.commit()
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;
        Ok(())
    }

    // ============================================
    // Row mappers
    // ============================================

    fn row_to_task(row: &rusqlite::Row) -> rusqlite::Result<TaskDto> {
        let id: String = row.get(0)?;
        let project_id: String = row.get(1)?;
        let content_type_str: String = row.get(2)?;
        let content_id: Option<String> = row.get(3)?;
        let title: String = row.get(4)?;
        let body: Option<String> = row.get(5)?;
        let status_option_id: Option<String> = row.get(6)?;
        let assignee_login: Option<String> = row.get(7)?;
        let due_date: Option<String> = row.get(8)?;
        let url: Option<String> = row.get(9)?;
        let updated_at: Option<i64> = row.get(10)?;
        let synced_at: Option<i64> = row.get(11)?;

        Ok(TaskDto {
            id: TaskId::new(id),
            project_id: ProjectId::new(project_id),
            content_type: content_type_str.parse().unwrap_or(ContentType::Issue),
            content_id,
            title,
            body,
            status_option_id: status_option_id.map(StatusOptionId::new),
            assignee_login,
            due_date,
            url,
            updated_at,
            synced_at,
        })
    }

    fn row_to_operation(row: &rusqlite::Row) -> rusqlite::Result<Operation> {
        let id: String = row.get(0)?;
        let op_type_str: String = row.get(1)?;
        let item_id: String = row.get(2)?;
        let project_id: String = row.get(3)?;
        let status_field_id: String = row.get(4)?;
        let to_option_id: String = row.get(5)?;
        let base_item_updated_at: i64 = row.get(6)?;
        let expected_from_option_id: String = row.get(7)?;
        let created_at: i64 = row.get(8)?;
        let status_str: String = row.get(9)?;
        let error_message: Option<String> = row.get(10)?;
        let resolved_at: Option<i64> = row.get(11)?;

        Ok(Operation {
            id: OperationId::new(id),
            op_type: op_type_str
                .parse()
                .unwrap_or(OperationType::MoveItemToColumn),
            payload: OperationPayload {
                item_id: TaskId::new(item_id),
                project_id: ProjectId::new(project_id),
                status_field_id: StatusFieldId::new(status_field_id),
                to_option_id: StatusOptionId::new(to_option_id),
            },
            precondition: Precondition {
                base_item_updated_at,
                expected_from_option_id: StatusOptionId::new(expected_from_option_id),
            },
            created_at,
            status: status_str.parse().unwrap_or(OperationStatus::Pending),
            error_message,
            resolved_at,
        })
    }

    fn query_tasks_with<P: rusqlite::Params>(
        stmt: &mut rusqlite::Statement,
        params: P,
    ) -> Result<Vec<TaskDto>, DomainError> {
        let rows = stmt
            .query_map(params, |row| Self::row_to_task(row))
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(row.map_err(|e| DomainError::PersistenceError(e.to_string()))?);
        }
        Ok(tasks)
    }

    fn query_operations_with<P: rusqlite::Params>(
        stmt: &mut rusqlite::Statement,
        params: P,
    ) -> Result<Vec<Operation>, DomainError> {
        let rows = stmt
            .query_map(params, |row| Self::row_to_operation(row))
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        let mut operations = Vec::new();
        for row in rows {
            operations.push(row.map_err(|e| DomainError::PersistenceError(e.to_string()))?);
        }
        Ok(operations)
    }
}

#[async_trait]
impl PersistencePort for SqlitePersistence {
    // ============================================
    // Settings
    // ============================================

    async fn get_setting(&self, key: &str) -> Result<Option<String>, DomainError> {
        let conn = self.conn();
        conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| DomainError::PersistenceError(e.to_string()))
    }

    async fn set_setting(&self, key: &str, value: &str) -> Result<(), DomainError> {
        let conn = self.conn();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )
        .map_err(|e| DomainError::PersistenceError(e.to_string()))?;
        Ok(())
    }

    async fn delete_setting(&self, key: &str) -> Result<(), DomainError> {
        let conn = self.conn();
        conn.execute("DELETE FROM settings WHERE key = ?1", params![key])
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;
        Ok(())
    }

    // ============================================
    // Projects
    // ============================================

    async fn get_all_projects(&self) -> Result<Vec<ProjectDto>, DomainError> {
        let conn = self.conn();

        // 1. Fetch all projects
        let mut stmt = conn
            .prepare(
                "SELECT id, owner_type, owner_login, title, url, updated_at, synced_at FROM projects",
            )
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let owner_type_str: String = row.get(1)?;
                let owner_login: String = row.get(2)?;
                let title: String = row.get(3)?;
                let url: String = row.get(4)?;
                let updated_at: Option<i64> = row.get(5)?;
                let synced_at: Option<i64> = row.get(6)?;

                Ok(ProjectDto {
                    id: ProjectId::new(id),
                    owner_type: owner_type_str.parse().unwrap_or(OwnerType::User),
                    owner_login,
                    title,
                    url,
                    updated_at,
                    synced_at,
                    status_field: None,
                })
            })
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        let mut projects = Vec::new();
        for row in rows {
            projects.push(row.map_err(|e| DomainError::PersistenceError(e.to_string()))?);
        }

        // 2. Batch-fetch all status fields
        let mut field_stmt = conn
            .prepare("SELECT id, project_id, name FROM status_fields")
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        let mut fields_by_project: HashMap<String, (StatusFieldId, String)> = HashMap::new();
        let field_rows = field_stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let project_id: String = row.get(1)?;
                let name: String = row.get(2)?;
                Ok((id, project_id, name))
            })
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        for row in field_rows {
            let (id, project_id, name) =
                row.map_err(|e| DomainError::PersistenceError(e.to_string()))?;
            fields_by_project.insert(project_id, (StatusFieldId::new(&id), name));
        }

        // 3. Batch-fetch all status options
        let mut opt_stmt = conn
            .prepare(
                "SELECT id, status_field_id, name, color, position FROM status_options ORDER BY position",
            )
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        let mut options_by_field: HashMap<String, Vec<StatusOptionDto>> = HashMap::new();
        let opt_rows = opt_stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let field_id: String = row.get(1)?;
                let name: String = row.get(2)?;
                let color: Option<String> = row.get(3)?;
                let position: i32 = row.get(4)?;
                Ok((id, field_id, name, color, position))
            })
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        for row in opt_rows {
            let (id, field_id, name, color, position) =
                row.map_err(|e| DomainError::PersistenceError(e.to_string()))?;
            options_by_field
                .entry(field_id.clone())
                .or_default()
                .push(StatusOptionDto {
                    id: StatusOptionId::new(id),
                    status_field_id: StatusFieldId::new(field_id),
                    name,
                    color,
                    position,
                });
        }

        // 4. Assemble projects with status fields and options
        for project in &mut projects {
            if let Some((field_id, name)) = fields_by_project.remove(project.id.as_str()) {
                let options = options_by_field
                    .remove(field_id.as_str())
                    .unwrap_or_default();
                project.status_field = Some(StatusFieldDto {
                    id: field_id,
                    project_id: project.id.clone(),
                    name,
                    options,
                });
            }
        }

        Ok(projects)
    }

    async fn get_project(&self, id: &ProjectId) -> Result<Option<ProjectDto>, DomainError> {
        let conn = self.conn();
        let result = conn
            .query_row(
                "SELECT id, owner_type, owner_login, title, url, updated_at, synced_at FROM projects WHERE id = ?1",
                params![id.as_str()],
                |row| {
                    let id: String = row.get(0)?;
                    let owner_type_str: String = row.get(1)?;
                    let owner_login: String = row.get(2)?;
                    let title: String = row.get(3)?;
                    let url: String = row.get(4)?;
                    let updated_at: Option<i64> = row.get(5)?;
                    let synced_at: Option<i64> = row.get(6)?;

                    Ok(ProjectDto {
                        id: ProjectId::new(id),
                        owner_type: owner_type_str.parse().unwrap_or(OwnerType::User),
                        owner_login,
                        title,
                        url,
                        updated_at,
                        synced_at,
                        status_field: None,
                    })
                },
            )
            .optional()
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        if let Some(mut project) = result {
            if let Ok(Some(field)) = Self::get_status_field_with(&conn, &project.id) {
                project.status_field = Some(field);
            }
            Ok(Some(project))
        } else {
            Ok(None)
        }
    }

    async fn upsert_project(&self, project: &ProjectDto) -> Result<(), DomainError> {
        let conn = self.conn();
        conn.execute(
            r#"
            INSERT INTO projects (id, owner_type, owner_login, title, url, updated_at, synced_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(id) DO UPDATE SET
                owner_type = excluded.owner_type,
                owner_login = excluded.owner_login,
                title = excluded.title,
                url = excluded.url,
                updated_at = excluded.updated_at,
                synced_at = excluded.synced_at
            "#,
            params![
                project.id.as_str(),
                project.owner_type.as_str(),
                project.owner_login,
                project.title,
                project.url,
                project.updated_at,
                project.synced_at,
            ],
        )
        .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        if let Some(ref field) = project.status_field {
            Self::upsert_status_field_with(&conn, field)?;
        }

        Ok(())
    }

    async fn delete_project(&self, id: &ProjectId) -> Result<(), DomainError> {
        let conn = self.conn();
        conn.execute("DELETE FROM projects WHERE id = ?1", params![id.as_str()])
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;
        Ok(())
    }

    // ============================================
    // Status Fields
    // ============================================

    async fn get_status_field(
        &self,
        project_id: &ProjectId,
    ) -> Result<Option<StatusFieldDto>, DomainError> {
        let conn = self.conn();
        Self::get_status_field_with(&conn, project_id)
    }

    async fn upsert_status_field(&self, field: &StatusFieldDto) -> Result<(), DomainError> {
        let conn = self.conn();
        Self::upsert_status_field_with(&conn, field)
    }

    async fn delete_status_field(&self, id: &StatusFieldId) -> Result<(), DomainError> {
        let conn = self.conn();
        conn.execute(
            "DELETE FROM status_fields WHERE id = ?1",
            params![id.as_str()],
        )
        .map_err(|e| DomainError::PersistenceError(e.to_string()))?;
        Ok(())
    }

    // ============================================
    // Status Options
    // ============================================

    async fn get_status_options(
        &self,
        field_id: &StatusFieldId,
    ) -> Result<Vec<StatusOptionDto>, DomainError> {
        let conn = self.conn();
        Self::get_status_options_with(&conn, field_id)
    }

    async fn upsert_status_option(&self, option: &StatusOptionDto) -> Result<(), DomainError> {
        let conn = self.conn();
        Self::upsert_status_option_with(&conn, option)
    }

    async fn delete_status_options_by_field(
        &self,
        field_id: &StatusFieldId,
    ) -> Result<(), DomainError> {
        let conn = self.conn();
        Self::delete_status_options_by_field_with(&conn, field_id)
    }

    // ============================================
    // Tasks
    // ============================================

    async fn get_tasks_by_project(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<TaskDto>, DomainError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, project_id, content_type, content_id, title, body,
                       status_option_id, assignee_login, due_date, url, updated_at, synced_at
                FROM tasks WHERE project_id = ?1
                "#,
            )
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        Self::query_tasks_with(&mut stmt, params![project_id.as_str()])
    }

    async fn get_tasks_by_assignee(
        &self,
        assignee_login: &str,
    ) -> Result<Vec<TaskDto>, DomainError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, project_id, content_type, content_id, title, body,
                       status_option_id, assignee_login, due_date, url, updated_at, synced_at
                FROM tasks WHERE assignee_login = ?1
                "#,
            )
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        Self::query_tasks_with(&mut stmt, params![assignee_login])
    }

    async fn get_all_tasks(&self) -> Result<Vec<TaskDto>, DomainError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, project_id, content_type, content_id, title, body,
                       status_option_id, assignee_login, due_date, url, updated_at, synced_at
                FROM tasks
                "#,
            )
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        Self::query_tasks_with(&mut stmt, [])
    }

    async fn get_task(&self, id: &TaskId) -> Result<Option<TaskDto>, DomainError> {
        let conn = self.conn();
        conn.query_row(
            r#"
            SELECT id, project_id, content_type, content_id, title, body,
                   status_option_id, assignee_login, due_date, url, updated_at, synced_at
            FROM tasks WHERE id = ?1
            "#,
            params![id.as_str()],
            |row| Self::row_to_task(row),
        )
        .optional()
        .map_err(|e| DomainError::PersistenceError(e.to_string()))
    }

    async fn upsert_task(&self, task: &TaskDto) -> Result<(), DomainError> {
        let conn = self.conn();
        conn.execute(
            r#"
            INSERT INTO tasks
                (id, project_id, content_type, content_id, title, body,
                 status_option_id, assignee_login, due_date, url, updated_at, synced_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            ON CONFLICT(id) DO UPDATE SET
                project_id = excluded.project_id,
                content_type = excluded.content_type,
                content_id = excluded.content_id,
                title = excluded.title,
                body = excluded.body,
                status_option_id = excluded.status_option_id,
                assignee_login = excluded.assignee_login,
                due_date = excluded.due_date,
                url = excluded.url,
                updated_at = excluded.updated_at,
                synced_at = excluded.synced_at
            "#,
            params![
                task.id.as_str(),
                task.project_id.as_str(),
                task.content_type.as_str(),
                task.content_id,
                task.title,
                task.body,
                task.status_option_id.as_ref().map(|id| id.as_str()),
                task.assignee_login,
                task.due_date,
                task.url,
                task.updated_at,
                task.synced_at,
            ],
        )
        .map_err(|e| DomainError::PersistenceError(e.to_string()))?;
        Ok(())
    }

    async fn delete_task(&self, id: &TaskId) -> Result<(), DomainError> {
        let conn = self.conn();
        conn.execute("DELETE FROM tasks WHERE id = ?1", params![id.as_str()])
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;
        Ok(())
    }

    async fn delete_tasks_by_project(&self, project_id: &ProjectId) -> Result<(), DomainError> {
        let conn = self.conn();
        conn.execute(
            "DELETE FROM tasks WHERE project_id = ?1",
            params![project_id.as_str()],
        )
        .map_err(|e| DomainError::PersistenceError(e.to_string()))?;
        Ok(())
    }

    async fn upsert_tasks_batch(&self, tasks: &[TaskDto]) -> Result<(), DomainError> {
        let conn = self.conn();
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        {
            let mut stmt = tx
                .prepare(
                    r#"
                    INSERT INTO tasks
                        (id, project_id, content_type, content_id, title, body,
                         status_option_id, assignee_login, due_date, url, updated_at, synced_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                    ON CONFLICT(id) DO UPDATE SET
                        project_id = excluded.project_id,
                        content_type = excluded.content_type,
                        content_id = excluded.content_id,
                        title = excluded.title,
                        body = excluded.body,
                        status_option_id = excluded.status_option_id,
                        assignee_login = excluded.assignee_login,
                        due_date = excluded.due_date,
                        url = excluded.url,
                        updated_at = excluded.updated_at,
                        synced_at = excluded.synced_at
                    "#,
                )
                .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

            for task in tasks {
                stmt.execute(params![
                    task.id.as_str(),
                    task.project_id.as_str(),
                    task.content_type.as_str(),
                    task.content_id,
                    task.title,
                    task.body,
                    task.status_option_id.as_ref().map(|id| id.as_str()),
                    task.assignee_login,
                    task.due_date,
                    task.url,
                    task.updated_at,
                    task.synced_at,
                ])
                .map_err(|e| DomainError::PersistenceError(e.to_string()))?;
            }
        }

        tx.commit()
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;
        Ok(())
    }

    async fn delete_stale_tasks_by_project(
        &self,
        project_id: &ProjectId,
        active_task_ids: &[TaskId],
    ) -> Result<(), DomainError> {
        if active_task_ids.is_empty() {
            return self.delete_tasks_by_project(project_id).await;
        }

        let conn = self.conn();
        let placeholders: Vec<String> = (0..active_task_ids.len())
            .map(|i| format!("?{}", i + 2))
            .collect();
        let sql = format!(
            "DELETE FROM tasks WHERE project_id = ?1 AND id NOT IN ({})",
            placeholders.join(", ")
        );

        let mut params_vec: Vec<Box<dyn rusqlite::types::ToSql>> =
            vec![Box::new(project_id.as_str().to_string())];
        for id in active_task_ids {
            params_vec.push(Box::new(id.as_str().to_string()));
        }

        conn.execute(
            &sql,
            rusqlite::params_from_iter(params_vec.iter().map(|p| p.as_ref())),
        )
        .map_err(|e| DomainError::PersistenceError(e.to_string()))?;
        Ok(())
    }

    // ============================================
    // Operations
    // ============================================

    async fn get_pending_operations(&self) -> Result<Vec<Operation>, DomainError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, op_type, item_id, project_id, status_field_id, to_option_id,
                       base_item_updated_at, expected_from_option_id, created_at, status,
                       error_message, resolved_at
                FROM operations WHERE status = 'pending' ORDER BY created_at ASC
                "#,
            )
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        Self::query_operations_with(&mut stmt, [])
    }

    async fn get_pending_operations_by_project(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<Operation>, DomainError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, op_type, item_id, project_id, status_field_id, to_option_id,
                       base_item_updated_at, expected_from_option_id, created_at, status,
                       error_message, resolved_at
                FROM operations WHERE status = 'pending' AND project_id = ?1 ORDER BY created_at ASC
                "#,
            )
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        Self::query_operations_with(&mut stmt, params![project_id.as_str()])
    }

    async fn get_operation(&self, id: &OperationId) -> Result<Option<Operation>, DomainError> {
        let conn = self.conn();
        conn.query_row(
            r#"
            SELECT id, op_type, item_id, project_id, status_field_id, to_option_id,
                   base_item_updated_at, expected_from_option_id, created_at, status,
                   error_message, resolved_at
            FROM operations WHERE id = ?1
            "#,
            params![id.as_str()],
            |row| Self::row_to_operation(row),
        )
        .optional()
        .map_err(|e| DomainError::PersistenceError(e.to_string()))
    }

    async fn insert_operation(&self, operation: &Operation) -> Result<(), DomainError> {
        let conn = self.conn();
        conn.execute(
            r#"
            INSERT INTO operations
                (id, op_type, item_id, project_id, status_field_id, to_option_id,
                 base_item_updated_at, expected_from_option_id, created_at, status,
                 error_message, resolved_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
            params![
                operation.id.as_str(),
                operation.op_type.as_str(),
                operation.payload.item_id.as_str(),
                operation.payload.project_id.as_str(),
                operation.payload.status_field_id.as_str(),
                operation.payload.to_option_id.as_str(),
                operation.precondition.base_item_updated_at,
                operation.precondition.expected_from_option_id.as_str(),
                operation.created_at,
                operation.status.as_str(),
                operation.error_message,
                operation.resolved_at,
            ],
        )
        .map_err(|e| DomainError::PersistenceError(e.to_string()))?;
        Ok(())
    }

    async fn update_operation_status(
        &self,
        id: &OperationId,
        status: OperationStatus,
        error_message: Option<&str>,
        resolved_at: Option<i64>,
    ) -> Result<(), DomainError> {
        let conn = self.conn();
        conn.execute(
            r#"
            UPDATE operations
            SET status = ?2, error_message = ?3, resolved_at = ?4
            WHERE id = ?1
            "#,
            params![id.as_str(), status.as_str(), error_message, resolved_at],
        )
        .map_err(|e| DomainError::PersistenceError(e.to_string()))?;
        Ok(())
    }

    async fn get_conflict_operations(&self) -> Result<Vec<Operation>, DomainError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, op_type, item_id, project_id, status_field_id, to_option_id,
                       base_item_updated_at, expected_from_option_id, created_at, status,
                       error_message, resolved_at
                FROM operations WHERE status = 'conflict' ORDER BY created_at ASC
                "#,
            )
            .map_err(|e| DomainError::PersistenceError(e.to_string()))?;

        Self::query_operations_with(&mut stmt, [])
    }
}
