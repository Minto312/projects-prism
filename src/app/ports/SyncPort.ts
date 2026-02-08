/**
 * Sync Port
 *
 * 同期処理の抽象インターフェース
 */

import type { Operation } from '../../ui_domain/ops/MoveItemToColumn';
import type { SyncState, SyncResult, AppendOperationInput } from '../dtos/SyncDto';

/**
 * Sync Port インターフェース
 */
export interface SyncPort {
  /**
   * 操作をキューに追加
   *
   * 即時応答（Rust側の永続キューに追加）
   */
  appendOperation(input: AppendOperationInput): Promise<Operation>;

  /**
   * 同期を実行
   *
   * pending opsをGitHubへ適用
   * - コンフリクト発生時は停止
   * - レート制限時は停止
   */
  syncNow(): Promise<SyncResult>;

  /**
   * 現在の同期状態を取得
   */
  getSyncState(): Promise<SyncState>;

  /**
   * コンフリクトを解決（現在の値を採用）
   */
  resolveConflictWithCurrent(operationId: string): Promise<void>;

  /**
   * コンフリクトを解決（操作を再試行）
   */
  resolveConflictWithOperation(operationId: string): Promise<void>;

  /**
   * 操作をキャンセル
   */
  cancelOperation(operationId: string): Promise<void>;
}
