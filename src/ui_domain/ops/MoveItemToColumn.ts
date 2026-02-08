/**
 * MoveItemToColumn操作
 *
 * タスクを別のStatusカラムに移動する操作を表す
 */

export type OperationStatus =
  | 'pending'
  | 'syncing'
  | 'completed'
  | 'conflict'
  | 'failed';

/**
 * MoveItemToColumn操作のペイロード
 */
export interface MoveItemToColumnPayload {
  /** 対象タスクID */
  itemId: string;
  /** プロジェクトID */
  projectId: string;
  /** StatusフィールドID */
  statusFieldId: string;
  /** 移動先Status optionId */
  toOptionId: string;
}

/**
 * MoveItemToColumn操作の前提条件
 */
export interface MoveItemToColumnPrecondition {
  /** 作成時点のProjectV2Item.updatedAt（epoch ms） */
  baseItemUpdatedAt: number;
  /** 作成時点のStatus optionId（衝突判定に使用） */
  expectedFromOptionId: string;
}

/**
 * MoveItemToColumn操作
 */
export interface MoveItemToColumnOperation {
  /** 操作ID（UUID） */
  id: string;
  /** 操作タイプ */
  opType: 'MoveItemToColumn';
  /** ペイロード */
  payload: MoveItemToColumnPayload;
  /** 前提条件 */
  precondition: MoveItemToColumnPrecondition;
  /** 作成日時（epoch ms） */
  createdAt: number;
  /** ステータス */
  status: OperationStatus;
  /** エラーメッセージ */
  errorMessage: string | null;
  /** 解決日時（epoch ms） */
  resolvedAt: number | null;
}

/**
 * 汎用Operation型（将来的に他の操作タイプも追加可能）
 */
export type Operation = MoveItemToColumnOperation;

/**
 * MoveItemToColumn操作を作成
 */
export function createMoveItemToColumnOperation(
  id: string,
  payload: MoveItemToColumnPayload,
  precondition: MoveItemToColumnPrecondition
): MoveItemToColumnOperation {
  return {
    id,
    opType: 'MoveItemToColumn',
    payload,
    precondition,
    createdAt: Date.now(),
    status: 'pending',
    errorMessage: null,
    resolvedAt: null,
  };
}

/**
 * 操作がpending状態かどうか
 */
export function isPendingOperation(operation: Operation): boolean {
  return operation.status === 'pending';
}

/**
 * 操作がconflict状態かどうか
 */
export function isConflictOperation(operation: Operation): boolean {
  return operation.status === 'conflict';
}

/**
 * 操作が完了済みかどうか
 */
export function isCompletedOperation(operation: Operation): boolean {
  return operation.status === 'completed';
}
