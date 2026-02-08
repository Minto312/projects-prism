/**
 * moveTask Usecase
 *
 * タスクのステータス変更ユースケース（純粋関数）
 */

import type { SyncPort } from '../ports/SyncPort';
import type { Task } from '../../ui_domain/model/Task';
import type { StatusField } from '../../ui_domain/model/Project';
import type { Operation } from '../../ui_domain/ops/MoveItemToColumn';
import type { AppendOperationInput } from '../dtos/SyncDto';
import { DomainError } from '../../ui_domain/errors/DomainError';

/**
 * タスク移動の入力
 */
export interface MoveTaskInput {
  /** 移動するタスク */
  task: Task;
  /** Statusフィールド */
  statusField: StatusField;
  /** 移動先のStatus optionId */
  toOptionId: string;
}

/**
 * タスク移動の結果
 */
export interface MoveTaskResult {
  /** 作成された操作 */
  operation: Operation;
  /** 楽観的に更新されたタスク */
  optimisticTask: Task;
}

/**
 * タスクを別のStatusカラムに移動
 *
 * 1. 操作をキューに追加（Rust側の永続キュー）
 * 2. 楽観的に更新されたタスクを返す
 *
 * 実際のGitHub同期は別途 syncOperations で実行
 */
export async function moveTask(
  port: SyncPort,
  input: MoveTaskInput
): Promise<MoveTaskResult> {
  const { task, statusField, toOptionId } = input;

  // 現在のStatus optionIdを取得（衝突判定用）
  const expectedFromOptionId = task.statusOptionId;
  if (expectedFromOptionId === null) {
    throw new DomainError(
      'INVALID_OPERATION',
      'Cannot move task without current status'
    );
  }

  // 同じStatusへの移動は無視
  if (expectedFromOptionId === toOptionId) {
    throw new DomainError(
      'INVALID_OPERATION',
      'Task is already in the target status'
    );
  }

  // 操作入力を作成
  const operationInput: AppendOperationInput = {
    opType: 'MoveItemToColumn',
    itemId: task.id,
    projectId: task.projectId,
    statusFieldId: statusField.id,
    toOptionId,
    baseItemUpdatedAt: task.updatedAt ?? Date.now(),
    expectedFromOptionId,
  };

  // 操作をキューに追加
  const operation = await port.appendOperation(operationInput);

  // 楽観的に更新されたタスクを作成
  const optimisticTask: Task = {
    ...task,
    statusOptionId: toOptionId,
  };

  return {
    operation,
    optimisticTask,
  };
}

/**
 * タスク移動操作を作成（操作追加なし、純粋関数）
 */
export function createMoveTaskOperation(
  task: Task,
  statusField: StatusField,
  toOptionId: string
): AppendOperationInput {
  const expectedFromOptionId = task.statusOptionId;
  if (expectedFromOptionId === null) {
    throw new DomainError(
      'INVALID_OPERATION',
      'Cannot move task without current status'
    );
  }

  return {
    opType: 'MoveItemToColumn',
    itemId: task.id,
    projectId: task.projectId,
    statusFieldId: statusField.id,
    toOptionId,
    baseItemUpdatedAt: task.updatedAt ?? Date.now(),
    expectedFromOptionId,
  };
}
