/**
 * Sync Adapter
 *
 * SyncPort の実装
 */

import type { SyncPort } from '../../app/ports/SyncPort';
import type { Operation } from '../../ui_domain/ops/MoveItemToColumn';
import type { SyncState, SyncResult, AppendOperationInput } from '../../app/dtos/SyncDto';
import { syncApi } from '../tauri/client';
import { useOpQueueStore } from './opQueueCache';
import { useConflictStore, createConflictDetail } from './conflictStore';

/**
 * SyncPort の実装
 */
export class SyncAdapter implements SyncPort {
  async appendOperation(input: AppendOperationInput): Promise<Operation> {
    // Rust側に操作を追加
    const operations = await syncApi.appendOps([input]);

    if (!operations || operations.length === 0) {
      throw new Error('appendOps returned empty result');
    }
    const operation = operations[0];

    // メモリキャッシュにも追加
    useOpQueueStore.getState().addOperation(operation);

    return operation;
  }

  async syncNow(): Promise<SyncResult> {
    const result = await syncApi.syncNow();

    // 完了した操作をキャッシュから削除
    const opQueueStore = useOpQueueStore.getState();
    for (const op of result.completed) {
      opQueueStore.removeOperation(op.id);
    }

    // 失敗した操作を更新
    for (const { operation, error } of result.failed) {
      opQueueStore.updateOperation(operation.id, {
        status: 'failed',
        errorMessage: error,
      });
    }

    // コンフリクトを処理
    const conflictStore = useConflictStore.getState();
    for (const conflict of result.conflicts) {
      opQueueStore.updateOperation(conflict.operation.id, {
        status: 'conflict',
      });

      // TODO: localOptionNameを取得するにはStatusOptionsが必要
      // 現状は空文字で代用
      conflictStore.addConflict(
        createConflictDetail(
          {
            operation: conflict.operation,
            currentOptionId: conflict.currentOptionId,
            currentOptionName: conflict.currentOptionName,
          },
          '' // localOptionNameは別途取得が必要
        )
      );
    }

    // 同期時刻を更新
    opQueueStore.setLastSyncedAt(Date.now());

    return result;
  }

  async getSyncState(): Promise<SyncState> {
    return syncApi.getSyncState();
  }

  async resolveConflictWithCurrent(operationId: string): Promise<void> {
    await syncApi.resolveConflictWithCurrent(operationId);

    // キャッシュから削除
    useOpQueueStore.getState().removeOperation(operationId);
    useConflictStore.getState().resolveConflict(operationId);
  }

  async resolveConflictWithOperation(operationId: string, currentOptionId: string): Promise<void> {
    await syncApi.resolveConflictWithOperation(operationId, currentOptionId);

    // 操作を pending に戻す
    useOpQueueStore.getState().updateOperation(operationId, {
      status: 'pending',
    });
    useConflictStore.getState().resolveConflict(operationId);
  }

  async cancelOperation(operationId: string): Promise<void> {
    await syncApi.cancelOperation(operationId);

    // キャッシュから削除
    useOpQueueStore.getState().removeOperation(operationId);
  }
}

/**
 * SyncAdapter のシングルトンインスタンス
 */
let syncAdapterInstance: SyncAdapter | null = null;

/**
 * SyncAdapter を取得
 */
export function getSyncAdapter(): SyncAdapter {
  if (!syncAdapterInstance) {
    syncAdapterInstance = new SyncAdapter();
  }
  return syncAdapterInstance;
}

/**
 * SyncAdapter のインスタンスをリセット（テスト用）
 */
export function resetSyncAdapter(): void {
  syncAdapterInstance = null;
}
