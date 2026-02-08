/**
 * Tauri Client
 *
 * Tauri invoke の薄いラッパー
 */

import { invoke } from '@tauri-apps/api/core';
import type { BootstrapResponse, ProjectBootstrapResponse } from '../../app/dtos/BootstrapDto';
import type { SyncState, SyncResult, AppendOperationInput } from '../../app/dtos/SyncDto';
import type { Operation } from '../../ui_domain/ops/MoveItemToColumn';

/**
 * Tauri Commands の型定義
 */
export interface TauriCommands {
  /**
   * Bootstrap データを取得
   */
  get_bootstrap: () => Promise<BootstrapResponse>;

  /**
   * プロジェクト詳細の Bootstrap データを取得
   */
  get_project_bootstrap: (args: { projectId: string }) => Promise<ProjectBootstrapResponse>;

  /**
   * Bootstrap をリフレッシュ（GitHub から再取得）
   */
  refresh_bootstrap: () => Promise<BootstrapResponse>;

  /**
   * プロジェクト詳細をリフレッシュ
   */
  refresh_project_bootstrap: (args: { projectId: string }) => Promise<ProjectBootstrapResponse>;

  /**
   * 操作をキューに追加
   */
  append_ops: (args: { ops: AppendOperationInput[] }) => Promise<Operation[]>;

  /**
   * 同期を実行
   */
  sync_now: () => Promise<SyncResult>;

  /**
   * 同期状態を取得
   */
  get_sync_state: () => Promise<SyncState>;

  /**
   * コンフリクトを解決（現在の値を採用）
   */
  resolve_conflict_with_current: (args: { operationId: string }) => Promise<void>;

  /**
   * コンフリクトを解決（操作を再試行）
   */
  resolve_conflict_with_operation: (args: { operationId: string }) => Promise<void>;

  /**
   * 操作をキャンセル
   */
  cancel_operation: (args: { operationId: string }) => Promise<void>;

  /**
   * PAT を設定
   */
  set_pat: (args: { pat: string }) => Promise<void>;

  /**
   * PAT が設定済みかどうか
   */
  has_pat: () => Promise<boolean>;

  /**
   * PAT を削除
   */
  clear_pat: () => Promise<void>;
}

/**
 * 型安全な invoke ラッパー
 */
export async function tauriInvoke<K extends keyof TauriCommands>(
  command: K,
  ...args: Parameters<TauriCommands[K]>
): Promise<Awaited<ReturnType<TauriCommands[K]>>> {
  const arg = args[0];
  return invoke(command, arg as Record<string, unknown>) as Awaited<ReturnType<TauriCommands[K]>>;
}

/**
 * Bootstrap API
 */
export const bootstrapApi = {
  getBootstrap: () => tauriInvoke('get_bootstrap'),
  getProjectBootstrap: (projectId: string) =>
    tauriInvoke('get_project_bootstrap', { projectId }),
  refreshBootstrap: () => tauriInvoke('refresh_bootstrap'),
  refreshProjectBootstrap: (projectId: string) =>
    tauriInvoke('refresh_project_bootstrap', { projectId }),
};

/**
 * Sync API
 */
export const syncApi = {
  appendOps: (ops: AppendOperationInput[]) => tauriInvoke('append_ops', { ops }),
  syncNow: () => tauriInvoke('sync_now'),
  getSyncState: () => tauriInvoke('get_sync_state'),
  resolveConflictWithCurrent: (operationId: string) =>
    tauriInvoke('resolve_conflict_with_current', { operationId }),
  resolveConflictWithOperation: (operationId: string) =>
    tauriInvoke('resolve_conflict_with_operation', { operationId }),
  cancelOperation: (operationId: string) =>
    tauriInvoke('cancel_operation', { operationId }),
};

/**
 * Settings API
 */
export const settingsApi = {
  setPat: (pat: string) => tauriInvoke('set_pat', { pat }),
  hasPat: () => tauriInvoke('has_pat'),
  clearPat: () => tauriInvoke('clear_pat'),
};
