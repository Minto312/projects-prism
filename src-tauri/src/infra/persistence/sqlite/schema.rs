pub const SCHEMA: &str = r#"
-- ========================================
-- 設定
-- ========================================
CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- ========================================
-- プロジェクト（キャッシュ）
-- ========================================
CREATE TABLE IF NOT EXISTS projects (
    id          TEXT PRIMARY KEY,
    owner_type  TEXT NOT NULL CHECK(owner_type IN ('organization', 'user')),
    owner_login TEXT NOT NULL,
    title       TEXT NOT NULL,
    url         TEXT NOT NULL,
    updated_at  INTEGER,
    synced_at   INTEGER
);

-- ========================================
-- ステータスフィールド定義（キャッシュ）
-- ========================================
CREATE TABLE IF NOT EXISTS status_fields (
    id          TEXT PRIMARY KEY,
    project_id  TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    UNIQUE(project_id)
);

-- ========================================
-- ステータス選択肢（キャッシュ）
-- ========================================
CREATE TABLE IF NOT EXISTS status_options (
    id              TEXT PRIMARY KEY,
    status_field_id TEXT NOT NULL REFERENCES status_fields(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    color           TEXT,
    position        INTEGER NOT NULL,
    UNIQUE(status_field_id, position)
);

-- ========================================
-- タスク（ProjectV2 Item）（キャッシュ）
-- ========================================
CREATE TABLE IF NOT EXISTS tasks (
    id                TEXT PRIMARY KEY,
    project_id        TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    content_type      TEXT NOT NULL CHECK(content_type IN ('Issue', 'DraftIssue', 'PullRequest')),
    content_id        TEXT,
    title             TEXT NOT NULL,
    body              TEXT,
    status_option_id  TEXT REFERENCES status_options(id) ON DELETE SET NULL,
    assignee_login    TEXT,
    due_date          TEXT,
    url               TEXT,
    updated_at        INTEGER,
    synced_at         INTEGER
);

-- ========================================
-- Operation Log（pending ops）
-- ========================================
CREATE TABLE IF NOT EXISTS operations (
    id                      TEXT PRIMARY KEY,
    op_type                 TEXT NOT NULL CHECK(op_type IN ('MoveItemToColumn')),
    item_id                 TEXT NOT NULL,
    project_id              TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    status_field_id         TEXT NOT NULL,
    to_option_id            TEXT NOT NULL,
    base_item_updated_at    INTEGER NOT NULL,
    expected_from_option_id TEXT NOT NULL,
    created_at              INTEGER NOT NULL,
    status                  TEXT NOT NULL DEFAULT 'pending'
                            CHECK(status IN ('pending', 'syncing', 'completed', 'conflict', 'failed')),
    error_message           TEXT,
    resolved_at             INTEGER
);

-- ========================================
-- インデックス
-- ========================================
CREATE INDEX IF NOT EXISTS idx_tasks_project_id ON tasks(project_id);
CREATE INDEX IF NOT EXISTS idx_tasks_assignee ON tasks(assignee_login);
CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks(due_date);
CREATE INDEX IF NOT EXISTS idx_operations_status_created_at ON operations(status, created_at);
CREATE INDEX IF NOT EXISTS idx_operations_project_status_created_at ON operations(project_id, status, created_at);
"#;
