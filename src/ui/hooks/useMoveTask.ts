/**
 * useMoveTask Hook
 *
 * タスク移動の処理
 */

import { useCallback } from 'react';
import { useBoardStore } from '../../infra/state/boardStore';
import { getSyncAdapter } from '../../infra/sync/syncAdapter';
import type { Task } from '../../ui_domain/model/Task';
import type { StatusField } from '../../ui_domain/model/Project';
import { createMoveTaskOperation } from '../../app/usecases/moveTask';

export interface UseMoveTaskOptions {
  projectId: string;
  statusField: StatusField | null;
}

export interface UseMoveTaskResult {
  moveTask: (
    taskId: string,
    fromColumnId: string,
    toColumnId: string,
    toIndex: number
  ) => Promise<void>;
  changeTaskStatus: (task: Task, newStatusOptionId: string) => Promise<void>;
  isMoving: boolean;
  error: string | null;
}

export function useMoveTask({
  projectId,
  statusField,
}: UseMoveTaskOptions): UseMoveTaskResult {
  const { moveTask: moveTaskInBoard, tasks } = useBoardStore((state) => ({
    moveTask: state.moveTask,
    tasks: state.boards[projectId]?.tasks ?? {},
  }));

  const moveTask = useCallback(
    async (
      taskId: string,
      fromColumnId: string,
      toColumnId: string,
      toIndex: number
    ) => {
      if (!statusField) {
        console.error('StatusField is not available');
        return;
      }

      const task = tasks[taskId];
      if (!task) {
        console.error(`Task not found: ${taskId}`);
        return;
      }

      // 楽観的更新（UI上で即座に移動）
      moveTaskInBoard(projectId, taskId, fromColumnId, toColumnId, toIndex);

      try {
        // 操作を作成してキューに追加
        const operationInput = createMoveTaskOperation(
          task,
          statusField,
          toColumnId
        );
        const syncAdapter = getSyncAdapter();
        await syncAdapter.appendOperation(operationInput);
      } catch (error) {
        // エラー時はロールバック
        console.error('Failed to append operation:', error);
        moveTaskInBoard(projectId, taskId, toColumnId, fromColumnId, 0);
      }
    },
    [projectId, statusField, tasks, moveTaskInBoard]
  );

  const changeTaskStatus = useCallback(
    async (task: Task, newStatusOptionId: string) => {
      if (!statusField) {
        console.error('StatusField is not available');
        return;
      }

      const currentStatusOptionId = task.statusOptionId ?? '__no_status__';

      // 楽観的更新
      moveTaskInBoard(
        projectId,
        task.id,
        currentStatusOptionId,
        newStatusOptionId,
        0
      );

      try {
        // 操作を作成してキューに追加
        const operationInput = createMoveTaskOperation(
          task,
          statusField,
          newStatusOptionId
        );
        const syncAdapter = getSyncAdapter();
        await syncAdapter.appendOperation(operationInput);
      } catch (error) {
        // エラー時はロールバック
        console.error('Failed to append operation:', error);
        moveTaskInBoard(
          projectId,
          task.id,
          newStatusOptionId,
          currentStatusOptionId,
          0
        );
      }
    },
    [projectId, statusField, moveTaskInBoard]
  );

  return {
    moveTask,
    changeTaskStatus,
    isMoving: false, // TODO: 実際の状態管理
    error: null,
  };
}
