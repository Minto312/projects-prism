# Project Prism - 設計仕様書

## 1. プロジェクト概要

### 1.1 目的

複数のGitHub Organization/アカウントに散らばったGitHub Projects V2を横断的に管理するデスクトップアプリケーション．

### 1.2 主要機能

- **マイタスクビュー**：全プロジェクト横断で自分がアサインされているタスクを一覧表示
- **プロジェクトビュー**：各GitHub Projectsのカンバン表示（ラッパー）
- **オフライン対応**：ローカルキャッシュによる閲覧・編集，オンライン復帰時の同期

### 1.3 技術スタック

| 領域 | 技術 |
|------|------|
| デスクトップフレームワーク | Tauri |
| フロントエンド | React |
| 状態管理 | Zustand + TanStack Query |
| バックエンド | Rust |
| ローカルDB | SQLite |
| 認証 | GitHub Personal Access Token（Classic PAT） |

---

## 2. 要件定義

### 2.1 MVP（v0.1）スコープ

| 機能 | 内容 |
|------|------|
| 認証 | PAT設定（単一アカウント） |
| プロジェクト | 一覧取得，選択 |
| マイタスク | 期限順表示のみ |
| プロジェクトビュー | カンバン表示（閲覧） |
| 操作 | ステータス変更のみ |
| データ | SQLiteキャッシュ，手動リロード |
| オフライン | 閲覧のみ |
| 通知 | なし |

### 2.2 v0.2以降のスコープ

| 機能 | 内容 |
|------|------|
| マイタスク | 4種グルーピング（期限順，プロジェクト別，ステータス別，優先度別） |
| 操作 | タスクのフルCRUD（作成・編集・削除） |
| オフライン | 編集対応＋同期（コンフリクト時はユーザー確認：C2方式） |
| 通知 | 期限が近いタスクの通知 |
| ポーリング | 設定可能な自動更新 |
| 認証 | 複数PAT対応 |

### 2.3 対象範囲

- 複数のOrganization + 個人アカウントにまたがるGitHub Projects
- クライアント先など外部Orgも含む（アクセス権限がある前提）

### 2.4 非機能要件

#### データ更新戦略

- **基本**：開いた時＋手動リロード（オンデマンド）
- **ポーリング**：設定で調整可能（デフォルトは無効または5分間隔）
- **Rate Limit考慮**：GitHub GraphQL API 5,000ポイント/時間

#### 公開形態

- OSS想定（各自がPATで利用するセルフホスト型）
- SaaS化は想定しない（Tauriデスクトップアプリ）

---

## 3. アーキテクチャ設計

### 3.1 設計原則

#### 依存関係ルール（一方向）

```
ui(presentation) → app(application) → domain
infra → app（ports実装として差し込む）

app/domain は infra を参照しない
変換（DB・API表現→モデル解釈）は infra（adapter/mapper）責務
```

#### 命名規則

| 避ける語彙 | 採用する語彙 | 理由 |
|------------|--------------|------|
| services（多層で衝突） | usecases | 責務の明確化 |
| repositories（曖昧） | ports + implementations | インターフェースと実装の分離 |
| adapters | implementations | portsの実装であることを明示 |

### 3.2 ディレクトリ構成

#### React（src/）

```
src/
├── ui/                           # プレゼンテーション層
│   ├── components/
│   │   ├── common/               # Button, Modal, etc.
│   │   ├── kanban/               # カンバンボード
│   │   ├── task/                 # タスクカード，詳細
│   │   └── layout/               # Sidebar, Header
│   ├── pages/
│   │   ├── MyTasksPage.tsx
│   │   ├── ProjectPage.tsx
│   │   └── SettingsPage.tsx
│   └── hooks/                    # UI用hook（usecaseを呼ぶ薄いラッパー）
│       ├── useMoveTask.ts
│       ├── useLoadBootstrap.ts
│       └── useSyncOperations.ts
│
├── app/                          # アプリケーション層
│   ├── usecases/                 # 純粋な関数（React非依存）
│   │   ├── moveTask.ts
│   │   ├── loadBootstrap.ts
│   │   └── syncOperations.ts
│   ├── ports/                    # 抽象インターフェース
│   │   ├── SyncPort.ts
│   │   └── BootstrapPort.ts
│   └── dtos/
│
├── ui_domain/                    # UI状態遷移ドメイン
│   ├── model/
│   │   ├── Task.ts
│   │   ├── Project.ts
│   │   └── KanbanState.ts
│   ├── ops/
│   │   └── MoveItemToColumn.ts
│   ├── reducers/
│   │   └── kanbanReducer.ts
│   └── errors/
│
├── infra/                        # インフラ層
│   ├── query/                    # TanStack Query（読み取り専用）
│   │   ├── queryClient.ts
│   │   ├── keys.ts
│   │   └── bootstrapQuery.ts
│   ├── sync/                     # 同期管理
│   │   ├── syncAdapter.ts        # ports実装
│   │   ├── opQueueCache.ts       # pending opsメモリキャッシュ
│   │   └── conflictStore.ts      # コンフリクト状態管理
│   ├── state/                    # Zustand stores（器）
│   │   ├── sessionStore.ts       # 選択中プロジェクト，UIタブ等
│   │   └── boardStore.ts         # KanbanState保持
│   └── tauri/
│       └── client.ts             # invoke薄ラッパ
│
├── shared/
│   ├── utils/
│   └── types/
│
├── App.tsx
└── main.tsx
```

#### Rust（src-tauri/src/）

```
src-tauri/src/
├── main.rs
├── lib.rs
│
├── interface/                    # 外部I/O境界
│   └── tauri/
│       └── commands/
│           ├── mod.rs
│           ├── bootstrap.rs      # get_bootstrap
│           ├── operations.rs     # append_ops
│           └── sync.rs           # sync_now, get_sync_state
│
├── app/                          # アプリケーション層
│   ├── usecases/
│   │   ├── mod.rs
│   │   ├── fetch_bootstrap.rs
│   │   ├── append_operations.rs
│   │   └── sync_to_github.rs
│   ├── ports/                    # 抽象インターフェース（trait）
│   │   ├── mod.rs
│   │   ├── github_port.rs
│   │   └── persistence_port.rs
│   └── dtos/
│       └── mod.rs
│
├── domain/                       # ドメイン層（最小）
│   ├── mod.rs
│   ├── ids/
│   │   └── mod.rs
│   └── errors/
│       └── mod.rs
│
└── infra/                        # インフラ層
    ├── mod.rs
    ├── github/
    │   ├── mod.rs
    │   ├── api/                  # GraphQL呼び出し
    │   ├── mapper/               # API応答→内部モデル変換
    │   └── implementations/      # ports実装
    └── persistence/
        └── sqlite/
            ├── mod.rs
            ├── schema.rs
            └── implementations/  # ports実装
```

### 3.3 Tauri Commandの設計

CRUDではなく「操作列」と「同期」を境界とする．

| Command | 説明 |
|---------|------|
| `get_bootstrap()` | snapshot＋rev＋pending ops＋conflicts |
| `append_ops(ops)` | 操作列を永続キューへ追加（即時応答） |
| `sync_now()` | pending ops を GitHub へ適用（結果・停止理由を返す） |
| `get_sync_state()` | レート制限，認証切れ等の状態取得 |

### 3.4 TanStack Queryの役割分担

| 責務 | 担当 |
|------|------|
| 初期ロード・表示キャッシュ | TanStack Query（`infra/query/`） |
| Operation Logの append | `infra/sync/syncAdapter.ts` |
| `sync_now()` の実行・停止理由分類 | `infra/sync/syncAdapter.ts` |

**設計理由**：
- C2（コンフリクト時に止める）とQueryの自動リトライが衝突するため分離
- 同期は「順序，停止点，副作用の分岐」が中心でQueryの抽象に合わない

---

## 4. データ設計

### 4.1 Operation Log方式

- **Source of Truth**：GitHub Projects V2
- **ローカル**：Operation Log（操作列）を正として扱い，GitHubへ後追い適用
- **永続化**：Rust側SQLite（TS側はメモリキャッシュのみ）

### 4.2 コンフリクト解決方式：C2

- コンフリクト検出時は**停止してユーザーに手動解決を求める**
- 自動リベースはしない

### 4.3 衝突検出ロジック

#### precondition（Operation作成時に記録）

- `base_item_updated_at`：作成時点の`ProjectV2Item.updatedAt`（監査用・将来の警告表示用）
- `expected_from_option_id`：作成時点のStatus optionId（**衝突判定に使用**）

#### 判定ルール

```
sync_now() 実行時：

1. operationsテーブルから status='pending' を取得（created_at順）

2. 各operationについて：
   a. GitHub APIで該当itemの現在Status optionIdを取得
   
   b. 判定：
      - expected_from_option_id == current_option_id
        → mutation実行（Status更新）
        → 成功: status='completed', resolved_at=now
        → API失敗: status='failed', error_message記録
      
      - expected_from_option_id != current_option_id
        → status='conflict'
        → UIに通知，ユーザー判断を待つ（C2）

3. conflict発生時は後続operationsの処理を停止
```

**設計理由**：
- `updatedAt`のみだと「Status以外の変更」でも止まってしまう
- Statusが一致していれば同期続行することで，不要な停止を回避

### 4.4 SQLiteスキーマ

#### 時刻カラムの規約

- **形式**：INTEGER（epoch ms）
- **理由**：ソート・比較の確実性，Rust/TSとの相互変換の容易さ

#### スキーマ定義

```sql
-- ========================================
-- 設定
-- ========================================
CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- ========================================
-- プロジェクト（キャッシュ）
-- ========================================
CREATE TABLE projects (
    id          TEXT PRIMARY KEY,   -- GitHub ProjectV2 node ID
    owner_type  TEXT NOT NULL CHECK(owner_type IN ('organization', 'user')),
    owner_login TEXT NOT NULL,
    title       TEXT NOT NULL,
    url         TEXT NOT NULL,
    updated_at  INTEGER,            -- epoch ms：キャッシュ無効化判断用
    synced_at   INTEGER             -- epoch ms：最後に同期した日時
);

-- ========================================
-- ステータスフィールド定義（キャッシュ）
-- ========================================
CREATE TABLE status_fields (
    id          TEXT PRIMARY KEY,   -- GitHub SingleSelectField node ID
    project_id  TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    UNIQUE(project_id)              -- 1プロジェクト1Statusフィールド前提
);

-- ========================================
-- ステータス選択肢（キャッシュ）
-- ========================================
CREATE TABLE status_options (
    id              TEXT PRIMARY KEY,   -- GitHub SingleSelectOption node ID
    status_field_id TEXT NOT NULL REFERENCES status_fields(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    color           TEXT,
    position        INTEGER NOT NULL,
    UNIQUE(status_field_id, position)
);

-- ========================================
-- タスク（ProjectV2 Item）（キャッシュ）
-- ========================================
CREATE TABLE tasks (
    id                TEXT PRIMARY KEY,   -- GitHub ProjectV2Item node ID
    project_id        TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    content_type      TEXT NOT NULL CHECK(content_type IN ('Issue', 'DraftIssue', 'PullRequest')),
    content_id        TEXT,               -- Issue/PR node ID（DraftIssueはnull）
    title             TEXT NOT NULL,
    body              TEXT,
    status_option_id  TEXT REFERENCES status_options(id),
    assignee_login    TEXT,               -- MVP: 単一担当のみ（複数担当は将来対応）
    due_date          TEXT,               -- ISO8601 date（YYYY-MM-DD形式）
    url               TEXT,
    updated_at        INTEGER,            -- epoch ms：ProjectV2Item.updatedAt
    synced_at         INTEGER             -- epoch ms
);

-- ========================================
-- Operation Log（pending ops）
-- ========================================
CREATE TABLE operations (
    id                      TEXT PRIMARY KEY,   -- UUID
    op_type                 TEXT NOT NULL CHECK(op_type IN ('MoveItemToColumn')),
    
    -- payload fields
    item_id                 TEXT NOT NULL,      -- 対象タスクID（FKなし：キャッシュ削除時もop保持）
    project_id              TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    status_field_id         TEXT NOT NULL,      -- StatusフィールドID
    to_option_id            TEXT NOT NULL,      -- 移動先Status optionId
    
    -- precondition（衝突検出用）
    base_item_updated_at    INTEGER NOT NULL,   -- epoch ms：監査用・将来の警告表示用
    expected_from_option_id TEXT NOT NULL,      -- 作成時点のStatus optionId（衝突判定に使用）
    
    -- メタ情報
    created_at              INTEGER NOT NULL,   -- epoch ms
    status                  TEXT NOT NULL DEFAULT 'pending'
                            CHECK(status IN ('pending', 'syncing', 'completed', 'conflict', 'failed')),
    error_message           TEXT,
    resolved_at             INTEGER             -- epoch ms：conflict解決 or 完了日時
);

-- ========================================
-- インデックス
-- ========================================
CREATE INDEX idx_tasks_project_id ON tasks(project_id);
CREATE INDEX idx_tasks_assignee ON tasks(assignee_login);
CREATE INDEX idx_tasks_due_date ON tasks(due_date);
CREATE INDEX idx_operations_status_created_at ON operations(status, created_at);
CREATE INDEX idx_operations_project_status_created_at ON operations(project_id, status, created_at);
```

#### 外部キーの設計方針

| カラム | FK有無 | 理由 |
|--------|--------|------|
| `operations.project_id` | あり（CASCADE） | プロジェクト削除時にopも削除 |
| `operations.item_id` | なし | キャッシュ削除時もopを保持したい |

#### MVP制約事項

- `assignee_login`は単一担当のみ対応（複数担当は将来`task_assignees`テーブルで対応）
- `status_fields`は1プロジェクト1フィールド前提

---

## 5. UI設計

### 5.1 画面構成

```
┌─────────────────────────────────────────────────┐
│ サイドバー          │  メインエリア              │
│                     │                            │
│ ・マイタスク        │  [選択したビューの内容]    │
│ ・プロジェクトA     │                            │
│ ・プロジェクトB     │  - カンバン表示            │
│ ・プロジェクトC     │  - 期限でソート/フィルタ   │
│ ...                 │                            │
│                     │                            │
│ [設定]              │                            │
└─────────────────────────────────────────────────┘
```

### 5.2 ビュー仕様

#### マイタスクビュー

- 全プロジェクト横断で自分がアサインされているタスクを表示
- グルーピング（v0.2以降で4種切り替え可能）：
  1. 期限順（MVP）
  2. プロジェクト別
  3. ステータス別
  4. 優先度別

#### プロジェクトビュー

- GitHub Projectsのラッパー
- カンバン表示のみ（テーブル，ロードマップは対象外）
- カラムはGitHub Projects側の`Status`フィールドをそのまま使用

---

## 6. 考慮事項・制約

### 6.1 GitHub API Rate Limit

- GraphQL API：5,000ポイント/時間
- 10プロジェクト×50タスクの場合，約500ポイント/回
- 1分間隔ポーリングはRate Limit超過の可能性あり

#### 対策

- 基本はオンデマンド（開いた時＋手動リロード）
- 「自分のアサインのみ取得」を基本にしてコスト削減
- 個別プロジェクト詳細は開いたときだけフル取得

### 6.2 Tauriの考慮点

| 観点 | 考慮事項 |
|------|----------|
| クロスプラットフォーム | macOS/Windows/Linuxでテスト・ビルドが必要 |
| 配布 | コード署名がないと警告が出る |
| 自動更新 | tauri-plugin-updaterの設定が必要 |
| WebViewの差異 | Windows=Edge WebView2，macOS=WebKit |

### 6.3 Classic PAT のスコープ

- `repo`スコープで大体カバー可能
- Organization Projectsへのアクセスも可能

### 6.4 二重ドメイン問題の回避

- Rust側`domain/`：最小（ID，エラー分類のみ）
- TS側`ui_domain/`：UI状態遷移のドメイン
- ビジネスロジックのバリデーションはGitHub APIの制約に任せる（ローカルでは検証しない）

---

## 7. 用語定義

| 用語 | 定義 |
|------|------|
| Operation | ユーザーの操作を表すデータ（例：MoveItemToColumn） |
| Operation Log | 未同期のOperationの列（pending ops） |
| C2 | コンフリクト時に止めてユーザー確認を求める方式 |
| bootstrap | 初期データ取得（snapshot＋rev＋pending ops＋conflicts） |
| precondition | Operation実行の前提条件（衝突検出に使用） |
| rev | revision：更新バージョンを表す値（updatedAt等） |

---

## 8. 今後の実装順序

1. **スキャフォールディング**：Tauri + React + SQLiteの初期セットアップ
2. **Rust側：GitHub API連携**：GraphQLクエリ，bootstrap取得
3. **Rust側：SQLite永続化層**：スキーマ作成，CRUD実装
4. **TS側：UI**：カンバン表示，マイタスクビュー

---

## 変更履歴

| 日付 | 内容 |
|------|------|
| 2026-01-31 | 初版作成 |