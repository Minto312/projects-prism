/**
 * Tauri Client
 *
 * Tauri invoke の薄いラッパー
 */

import { invoke } from '@tauri-apps/api/core';
import type { BootstrapResponse, ProjectBootstrapResponse } from '../../application/dtos/BootstrapDto';
import type { SyncState, SyncResult, AppendOperationInput } from '../../application/dtos/SyncDto';
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
  resolve_conflict_with_operation: (args: { operationId: string; currentOptionId: string }) => Promise<void>;

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

  /**
   * 更新チャネルを取得
   */
  get_update_channel: () => Promise<string>;

  /**
   * 更新チャネルを設定
   */
  set_update_channel: (args: { channel: string }) => Promise<void>;

  /**
   * 更新を確認
   */
  check_for_update: () => Promise<{ version: string; body: string | null } | null>;

  /**
   * 更新をダウンロード・インストール
   */
  download_and_install_update: () => Promise<void>;

  /**
   * デバッグログを取得
   */
  get_debug_logs: () => Promise<string>;
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
  resolveConflictWithOperation: (operationId: string, currentOptionId: string) =>
    tauriInvoke('resolve_conflict_with_operation', { operationId, currentOptionId }),
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

/**
 * Updater API
 */
export const updaterApi = {
  getUpdateChannel: () => tauriInvoke('get_update_channel'),
  setUpdateChannel: (channel: string) => tauriInvoke('set_update_channel', { channel }),
  checkForUpdate: () => tauriInvoke('check_for_update'),
  downloadAndInstallUpdate: () => tauriInvoke('download_and_install_update'),
};

/**
 * Debug API
 */
export const debugApi = {
  getDebugLogs: () => tauriInvoke('get_debug_logs'),
};
