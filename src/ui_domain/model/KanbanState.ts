/**
 * カンバンボードのUIドメインモデル
 */

import type { Task } from './Task';
import type { StatusOption, Project } from './Project';
import type { StatusField } from './Project';

/**
 * カンバンカラム
 */
export interface KanbanColumn {
  /** Status option ID */
  id: string;
  /** カラム名（Status名） */
  name: string;
  /** 色 */
  color: string | null;
  /** カラム内のタスクID一覧（表示順） */
  taskIds: string[];
}

/**
 * カンバンボードの状態
 */
export interface KanbanState {
  /** 対象プロジェクト */
  project: Project;
  /** Statusフィールド */
  statusField: StatusField | null;
  /** カラム一覧（Statusの表示順） */
  columns: KanbanColumn[];
  /** タスクのマップ（ID -> Task） */
  tasks: Record<string, Task>;
  /** ドラッグ中のタスクID */
  draggingTaskId: string | null;
  /** ローディング状態 */
  isLoading: boolean;
  /** エラー */
  error: string | null;
}

/**
 * StatusOptionsからカラム一覧を生成
 */
export function createColumnsFromStatusOptions(
  statusOptions: StatusOption[],
  tasks: Task[]
): KanbanColumn[] {
  // StatusOptionをposition順でソート
  const sortedOptions = [...statusOptions].sort(
    (a, b) => a.position - b.position
  );

  // 各Statusごとにタスクをグループ化
  const tasksByStatus = new Map<string, string[]>();

  for (const task of tasks) {
    const statusId = task.statusOptionId ?? '__no_status__';
    const existing = tasksByStatus.get(statusId) ?? [];
    existing.push(task.id);
    tasksByStatus.set(statusId, existing);
  }

  // カラムを生成
  const columns: KanbanColumn[] = sortedOptions.map((option) => ({
    id: option.id,
    name: option.name,
    color: option.color,
    taskIds: tasksByStatus.get(option.id) ?? [],
  }));

  // Statusが設定されていないタスク用のカラムを先頭に追加（存在する場合）
  const noStatusTasks = tasksByStatus.get('__no_status__');
  if (noStatusTasks && noStatusTasks.length > 0) {
    columns.unshift({
      id: '__no_status__',
      name: 'No Status',
      color: null,
      taskIds: noStatusTasks,
    });
  }

  return columns;
}

/**
 * 空のKanbanStateを生成
 */
export function createEmptyKanbanState(project: Project): KanbanState {
  return {
    project,
    statusField: null,
    columns: [],
    tasks: {},
    draggingTaskId: null,
    isLoading: false,
    error: null,
  };
}

/**
 * タスクを別カラムに移動した状態を生成
 */
export function moveTaskToColumn(
  state: KanbanState,
  taskId: string,
  fromColumnId: string,
  toColumnId: string,
  toIndex: number
): KanbanState {
  const newColumns = state.columns.map((column) => {
    if (column.id === fromColumnId) {
      // 移動元カラムからタスクを削除
      return {
        ...column,
        taskIds: column.taskIds.filter((id) => id !== taskId),
      };
    }
    if (column.id === toColumnId) {
      // 移動先カラムにタスクを挿入
      const newTaskIds = [...column.taskIds];
      newTaskIds.splice(toIndex, 0, taskId);
      return {
        ...column,
        taskIds: newTaskIds,
      };
    }
    return column;
  });

  // タスクのstatusOptionIdも更新
  const task = state.tasks[taskId];
  const newTasks = task
    ? {
        ...state.tasks,
        [taskId]: {
          ...task,
          statusOptionId: toColumnId === '__no_status__' ? null : toColumnId,
        },
      }
    : state.tasks;

  return {
    ...state,
    columns: newColumns,
    tasks: newTasks,
  };
}
