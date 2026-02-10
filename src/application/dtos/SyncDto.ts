/**
 * Sync DTO
 *
 * 同期関連のDTO型定義
 */

import type { Operation } from '../../ui_domain/ops/MoveItemToColumn';

/**
 * 同期状態
 */
export type SyncStatus =
  | 'idle'
  | 'syncing'
  | 'paused_conflict'
  | 'paused_rate_limited'
  | 'paused_auth_error'
  | 'error';

/**
 * 同期状態の詳細
 */
export interface SyncState {
  /** 現在の同期状態 */
  status: SyncStatus;
  /** 未同期の操作数 */
  pendingCount: number;
  /** コンフリクト中の操作数 */
  conflictCount: number;
  /** レート制限解除予定時刻（epoch ms） */
  rateLimitResetAt: number | null;
  /** 最後のエラーメッセージ */
  lastError: string | null;
  /** 最後に同期を試みた時刻（epoch ms） */
  lastSyncAttemptAt: number | null;
  /** 最後に同期が成功した時刻（epoch ms） */
  lastSyncSuccessAt: number | null;
}

/**
 * 同期結果
 */
export interface SyncResult {
  /** 成功した操作 */
  completed: Operation[];
  /** 失敗した操作 */
  failed: Array<{ operation: Operation; error: string }>;
  /** コンフリクトが発生した操作 */
  conflicts: Array<{
    operation: Operation;
    currentOptionId: string;
    currentOptionName: string;
  }>;
  /** 同期後の状態 */
  state: SyncState;
}

/**
 * 操作追加の入力
 */
export interface AppendOperationInput {
  /** 操作タイプ */
  opType: 'MoveItemToColumn';
  /** 対象タスクID */
  itemId: string;
  /** プロジェクトID */
  projectId: string;
  /** StatusフィールドID */
  statusFieldId: string;
  /** 移動先Status optionId */
  toOptionId: string;
  /** 作成時点のProjectV2Item.updatedAt */
  baseItemUpdatedAt: number;
  /** 作成時点のStatus optionId */
  expectedFromOptionId: string;
}
