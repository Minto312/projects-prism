/**
 * Bootstrap DTO
 *
 * Tauri Command `get_bootstrap()` の応答型
 */

import type { Task } from '../../ui_domain/model/Task';
import type { Project, StatusField, StatusOption } from '../../ui_domain/model/Project';
import type { Operation } from '../../ui_domain/ops/MoveItemToColumn';

/**
 * コンフリクト情報
 */
export interface ConflictInfo {
  /** コンフリクトした操作 */
  operation: Operation;
  /** 現在のGitHub上のStatus optionId */
  currentOptionId: string;
  /** 現在のGitHub上のStatus名 */
  currentOptionName: string;
}

/**
 * Bootstrap応答
 */
export interface BootstrapResponse {
  /** プロジェクト一覧 */
  projects: Project[];
  /** プロジェクトごとのStatusフィールド */
  statusFields: StatusField[];
  /** Status選択肢一覧 */
  statusOptions: StatusOption[];
  /** タスク一覧 */
  tasks: Task[];
  /** 未同期の操作一覧（pending ops） */
  pendingOperations: Operation[];
  /** コンフリクト情報 */
  conflicts: ConflictInfo[];
  /** 認証済みユーザーのログイン名 */
  currentUserLogin: string;
}

/**
 * プロジェクト詳細のBootstrap応答
 */
export interface ProjectBootstrapResponse {
  /** プロジェクト */
  project: Project;
  /** Statusフィールド */
  statusField: StatusField | null;
  /** Status選択肢一覧 */
  statusOptions: StatusOption[];
  /** タスク一覧 */
  tasks: Task[];
  /** 未同期の操作一覧（このプロジェクトのみ） */
  pendingOperations: Operation[];
  /** コンフリクト情報（このプロジェクトのみ） */
  conflicts: ConflictInfo[];
}
