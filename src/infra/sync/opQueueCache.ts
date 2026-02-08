/**
 * Operation Queue Cache
 *
 * Pending Operations のメモリキャッシュ
 * Rust側の永続キューの鏡像を保持
 */

import { create } from 'zustand';
import type { Operation } from '../../ui_domain/ops/MoveItemToColumn';

/**
 * Operation Queue State
 */
export interface OpQueueState {
  /** Pending operations（created_at順） */
  operations: Operation[];
  /** 最後に同期した時刻 */
  lastSyncedAt: number | null;
}

/**
 * Operation Queue Actions
 */
export interface OpQueueActions {
  /** 操作を追加 */
  addOperation: (operation: Operation) => void;
  /** 操作を複数追加 */
  addOperations: (operations: Operation[]) => void;
  /** 操作を更新 */
  updateOperation: (operationId: string, updates: Partial<Operation>) => void;
  /** 操作を削除 */
  removeOperation: (operationId: string) => void;
  /** 操作リストを置換 */
  setOperations: (operations: Operation[]) => void;
  /** Pending操作を取得 */
  getPendingOperations: () => Operation[];
  /** Conflict操作を取得 */
  getConflictOperations: () => Operation[];
  /** 特定タスクの操作を取得 */
  getOperationsForTask: (taskId: string) => Operation[];
  /** キャッシュをクリア */
  clear: () => void;
  /** 最終同期時刻を更新 */
  setLastSyncedAt: (time: number) => void;
}

/**
 * 初期状態
 */
const initialState: OpQueueState = {
  operations: [],
  lastSyncedAt: null,
};

/**
 * Operation Queue Store
 */
export const useOpQueueStore = create<OpQueueState & OpQueueActions>(
  (set, get) => ({
    ...initialState,

    addOperation: (operation) =>
      set((state) => ({
        operations: [...state.operations, operation].sort(
          (a, b) => a.createdAt - b.createdAt
        ),
      })),

    addOperations: (operations) =>
      set((state) => ({
        operations: [...state.operations, ...operations].sort(
          (a, b) => a.createdAt - b.createdAt
        ),
      })),

    updateOperation: (operationId, updates) =>
      set((state) => ({
        operations: state.operations.map((op) =>
          op.id === operationId ? { ...op, ...updates } : op
        ),
      })),

    removeOperation: (operationId) =>
      set((state) => ({
        operations: state.operations.filter((op) => op.id !== operationId),
      })),

    setOperations: (operations) =>
      set({
        operations: [...operations].sort((a, b) => a.createdAt - b.createdAt),
      }),

    getPendingOperations: () =>
      get().operations.filter((op) => op.status === 'pending'),

    getConflictOperations: () =>
      get().operations.filter((op) => op.status === 'conflict'),

    getOperationsForTask: (taskId) =>
      get().operations.filter((op) => op.payload.itemId === taskId),

    clear: () => set(initialState),

    setLastSyncedAt: (time) => set({ lastSyncedAt: time }),
  })
);

/**
 * セレクター
 */
export const opQueueSelectors = {
  operations: (state: OpQueueState) => state.operations,
  pendingCount: (state: OpQueueState) =>
    state.operations.filter((op) => op.status === 'pending').length,
  conflictCount: (state: OpQueueState) =>
    state.operations.filter((op) => op.status === 'conflict').length,
  hasPendingOperations: (state: OpQueueState) =>
    state.operations.some((op) => op.status === 'pending'),
  hasConflicts: (state: OpQueueState) =>
    state.operations.some((op) => op.status === 'conflict'),
  lastSyncedAt: (state: OpQueueState) => state.lastSyncedAt,
};
