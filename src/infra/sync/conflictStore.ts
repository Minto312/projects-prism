/**
 * Conflict Store
 *
 * コンフリクト状態の管理
 */

import { create } from 'zustand';
import type { ConflictInfo } from '../../application/dtos/BootstrapDto';

/**
 * 拡張コンフリクト情報（UI表示用）
 */
export interface ConflictDetail extends ConflictInfo {
  /** ローカルでの変更先Status名 */
  localOptionName: string;
  /** 検出時刻 */
  detectedAt: number;
}

/**
 * Conflict State
 */
export interface ConflictState {
  /** コンフリクト一覧 */
  conflicts: ConflictDetail[];
  /** 現在表示中のコンフリクトID */
  activeConflictId: string | null;
  /** コンフリクトダイアログの表示状態 */
  isDialogOpen: boolean;
}

/**
 * Conflict Actions
 */
export interface ConflictActions {
  /** コンフリクトを追加 */
  addConflict: (conflict: ConflictDetail) => void;
  /** コンフリクトを複数設定 */
  setConflicts: (conflicts: ConflictDetail[]) => void;
  /** コンフリクトを解決（削除） */
  resolveConflict: (operationId: string) => void;
  /** アクティブなコンフリクトを設定 */
  setActiveConflict: (operationId: string | null) => void;
  /** ダイアログを開く */
  openDialog: (operationId: string) => void;
  /** ダイアログを閉じる */
  closeDialog: () => void;
  /** コンフリクトをクリア */
  clear: () => void;
}

/**
 * 初期状態
 */
const initialState: ConflictState = {
  conflicts: [],
  activeConflictId: null,
  isDialogOpen: false,
};

/**
 * Conflict Store
 */
export const useConflictStore = create<ConflictState & ConflictActions>(
  (set) => ({
    ...initialState,

    addConflict: (conflict) =>
      set((state) => {
        // 同じ操作のコンフリクトがあれば更新
        const existing = state.conflicts.find(
          (c) => c.operation.id === conflict.operation.id
        );
        if (existing) {
          return {
            conflicts: state.conflicts.map((c) =>
              c.operation.id === conflict.operation.id ? conflict : c
            ),
          };
        }
        return {
          conflicts: [...state.conflicts, conflict],
        };
      }),

    setConflicts: (conflicts) =>
      set({
        conflicts,
      }),

    resolveConflict: (operationId) =>
      set((state) => ({
        conflicts: state.conflicts.filter(
          (c) => c.operation.id !== operationId
        ),
        activeConflictId:
          state.activeConflictId === operationId
            ? null
            : state.activeConflictId,
        isDialogOpen:
          state.activeConflictId === operationId ? false : state.isDialogOpen,
      })),

    setActiveConflict: (operationId) =>
      set({
        activeConflictId: operationId,
      }),

    openDialog: (operationId) =>
      set({
        activeConflictId: operationId,
        isDialogOpen: true,
      }),

    closeDialog: () =>
      set({
        isDialogOpen: false,
      }),

    clear: () => set(initialState),
  })
);

/**
 * セレクター
 */
export const conflictSelectors = {
  conflicts: (state: ConflictState) => state.conflicts,
  activeConflict: (state: ConflictState) =>
    state.conflicts.find((c) => c.operation.id === state.activeConflictId) ??
    null,
  isDialogOpen: (state: ConflictState) => state.isDialogOpen,
  hasConflicts: (state: ConflictState) => state.conflicts.length > 0,
  conflictCount: (state: ConflictState) => state.conflicts.length,
};

/**
 * ConflictInfoからConflictDetailを作成
 */
export function createConflictDetail(
  conflictInfo: ConflictInfo,
  localOptionName: string
): ConflictDetail {
  return {
    ...conflictInfo,
    localOptionName,
    detectedAt: Date.now(),
  };
}
