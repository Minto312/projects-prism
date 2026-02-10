/**
 * useSyncOperations Hook
 *
 * 同期処理の管理
 */

import { useCallback, useEffect, useRef } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { queryKeys } from '../../infra/query/keys';
import { getSyncAdapter } from '../../infra/sync/syncAdapter';
import { useOpQueueStore, opQueueSelectors } from '../../infra/sync/opQueueCache';
import { useConflictStore, conflictSelectors } from '../../infra/sync/conflictStore';
import type { SyncState, SyncResult } from '../../application/dtos/SyncDto';
import {
  shouldSync,
  getSyncStatusMessage,
  getSyncStatusIcon,
} from '../../application/usecases/syncOperations';

export interface UseSyncOperationsResult {
  syncState: SyncState | null;
  isSyncing: boolean;
  pendingCount: number;
  conflictCount: number;
  hasConflicts: boolean;
  statusMessage: string;
  statusIcon: 'success' | 'warning' | 'error' | 'loading';
  syncNow: () => Promise<SyncResult | null>;
  resolveConflictWithCurrent: (operationId: string) => Promise<void>;
  resolveConflictWithOperation: (operationId: string, currentOptionId: string) => Promise<void>;
  cancelOperation: (operationId: string) => Promise<void>;
}

export function useSyncOperations(): UseSyncOperationsResult {
  const queryClient = useQueryClient();
  const syncAdapter = getSyncAdapter();

  // Zustand storeからの状態
  const pendingCount = useOpQueueStore(opQueueSelectors.pendingCount);
  const conflictCount = useOpQueueStore(opQueueSelectors.conflictCount);
  const hasConflicts = useConflictStore(conflictSelectors.hasConflicts);

  // 同期状態の取得
  const syncStateQuery = useQuery<SyncState>({
    queryKey: queryKeys.sync.state(),
    queryFn: () => syncAdapter.getSyncState(),
    refetchInterval: 30000, // 30秒ごとに更新
  });

  // 同期実行のミューテーション
  const syncMutation = useMutation<SyncResult>({
    mutationFn: () => syncAdapter.syncNow(),
    onSuccess: () => {
      // 同期後にBootstrapを再取得
      queryClient.invalidateQueries({ queryKey: queryKeys.bootstrap.all });
      queryClient.invalidateQueries({ queryKey: queryKeys.sync.state() });
    },
  });

  // コンフリクト解決（現在の値を採用）
  const resolveWithCurrentMutation = useMutation({
    mutationFn: (operationId: string) =>
      syncAdapter.resolveConflictWithCurrent(operationId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.sync.state() });
    },
  });

  // コンフリクト解決（操作を再試行）
  const resolveWithOperationMutation = useMutation({
    mutationFn: ({ operationId, currentOptionId }: { operationId: string; currentOptionId: string }) =>
      syncAdapter.resolveConflictWithOperation(operationId, currentOptionId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.sync.state() });
    },
  });

  // 操作キャンセル
  const cancelMutation = useMutation({
    mutationFn: (operationId: string) =>
      syncAdapter.cancelOperation(operationId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.sync.state() });
    },
  });

  const syncNow = useCallback(async () => {
    if (syncMutation.isPending) return null;
    return syncMutation.mutateAsync();
  }, [syncMutation]);

  const resolveConflictWithCurrent = useCallback(
    async (operationId: string) => {
      await resolveWithCurrentMutation.mutateAsync(operationId);
    },
    [resolveWithCurrentMutation]
  );

  const resolveConflictWithOperation = useCallback(
    async (operationId: string, currentOptionId: string) => {
      await resolveWithOperationMutation.mutateAsync({ operationId, currentOptionId });
    },
    [resolveWithOperationMutation]
  );

  const cancelOperation = useCallback(
    async (operationId: string) => {
      await cancelMutation.mutateAsync(operationId);
    },
    [cancelMutation]
  );

  // 同期状態からメッセージとアイコンを計算
  const syncState = syncStateQuery.data ?? null;
  const statusMessage = syncState
    ? getSyncStatusMessage(syncState)
    : pendingCount > 0
    ? `${pendingCount}件の未同期の変更があります`
    : '同期済み';
  const statusIcon = syncState
    ? getSyncStatusIcon(syncState)
    : pendingCount > 0
    ? 'warning'
    : 'success';

  return {
    syncState,
    isSyncing: syncMutation.isPending,
    pendingCount,
    conflictCount,
    hasConflicts,
    statusMessage,
    statusIcon,
    syncNow,
    resolveConflictWithCurrent,
    resolveConflictWithOperation,
    cancelOperation,
  };
}

/**
 * 自動同期のフック（未使用だがMVP後の拡張用）
 */
export function useAutoSync(enabled: boolean = false, intervalMs: number = 300000) {
  const { syncNow, syncState, pendingCount } = useSyncOperations();

  const syncNowRef = useRef(syncNow);
  syncNowRef.current = syncNow;

  const canSync = enabled && pendingCount > 0 && (!syncState || shouldSync(syncState));

  useEffect(() => {
    if (!canSync) return;

    const timer = setInterval(() => {
      syncNowRef.current();
    }, intervalMs);

    return () => clearInterval(timer);
  }, [canSync, intervalMs]);
}
