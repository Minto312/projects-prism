/**
 * syncOperations Usecase
 *
 * 同期処理のユースケース（純粋関数）
 */

import type { SyncPort } from '../ports/SyncPort';
import type { SyncState, SyncResult } from '../dtos/SyncDto';

/**
 * 同期を実行
 *
 * pending opsをGitHubへ適用
 */
export async function syncNow(port: SyncPort): Promise<SyncResult> {
  return port.syncNow();
}

/**
 * 同期状態を取得
 */
export async function getSyncState(port: SyncPort): Promise<SyncState> {
  return port.getSyncState();
}

/**
 * コンフリクトを解決（現在のGitHub上の値を採用）
 *
 * 操作をキャンセルし、ローカルの状態をGitHubの値に戻す
 */
export async function resolveConflictWithCurrent(
  port: SyncPort,
  operationId: string
): Promise<void> {
  return port.resolveConflictWithCurrent(operationId);
}

/**
 * コンフリクトを解決（ローカルの操作を再試行）
 *
 * 現在のGitHubの状態を新しいbase_item_updated_atとして
 * 操作を再試行
 */
export async function resolveConflictWithOperation(
  port: SyncPort,
  operationId: string,
  currentOptionId: string
): Promise<void> {
  return port.resolveConflictWithOperation(operationId, currentOptionId);
}

/**
 * 操作をキャンセル
 */
export async function cancelOperation(
  port: SyncPort,
  operationId: string
): Promise<void> {
  return port.cancelOperation(operationId);
}

/**
 * 同期が必要かどうかを判定
 */
export function shouldSync(state: SyncState): boolean {
  // 既に同期中なら不要
  if (state.status === 'syncing') return false;

  // コンフリクト停止中なら不要（ユーザーの解決待ち）
  if (state.status === 'paused_conflict') return false;

  // 認証エラーなら不要（設定変更待ち）
  if (state.status === 'paused_auth_error') return false;

  // レート制限中で解除前なら不要
  if (state.status === 'paused_rate_limited' && state.rateLimitResetAt) {
    if (Date.now() < state.rateLimitResetAt) return false;
  }

  // pending操作があれば同期が必要
  return state.pendingCount > 0;
}

/**
 * 同期状態のユーザー向けメッセージを取得
 */
export function getSyncStatusMessage(state: SyncState): string {
  switch (state.status) {
    case 'idle':
      if (state.pendingCount > 0) {
        return `${state.pendingCount}件の未同期の変更があります`;
      }
      return '同期済み';

    case 'syncing':
      return '同期中...';

    case 'paused_conflict':
      return `${state.conflictCount}件のコンフリクトがあります`;

    case 'paused_rate_limited':
      if (state.rateLimitResetAt) {
        const resetTime = new Date(state.rateLimitResetAt);
        return `APIレート制限中（${resetTime.toLocaleTimeString()}に解除）`;
      }
      return 'APIレート制限中';

    case 'paused_auth_error':
      return '認証エラー。PATを確認してください';

    case 'error':
      return state.lastError ?? '同期エラーが発生しました';

    default:
      return '不明な状態';
  }
}

/**
 * 同期状態のアイコン種別を取得
 */
export function getSyncStatusIcon(
  state: SyncState
): 'success' | 'warning' | 'error' | 'loading' {
  switch (state.status) {
    case 'idle':
      return state.pendingCount > 0 ? 'warning' : 'success';
    case 'syncing':
      return 'loading';
    case 'paused_conflict':
    case 'paused_rate_limited':
      return 'warning';
    case 'paused_auth_error':
    case 'error':
      return 'error';
    default:
      return 'success';
  }
}
