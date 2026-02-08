/**
 * カンバンボード状態のReducer
 */

import type { Task } from '../model/Task';
import type { Project, StatusField, StatusOption } from '../model/Project';
import {
  type KanbanState,
  createColumnsFromStatusOptions,
  moveTaskToColumn,
} from '../model/KanbanState';

/**
 * アクションタイプ
 */
export type KanbanAction =
  | { type: 'LOAD_START' }
  | {
      type: 'LOAD_SUCCESS';
      payload: {
        project: Project;
        statusField: StatusField | null;
        statusOptions: StatusOption[];
        tasks: Task[];
      };
    }
  | { type: 'LOAD_ERROR'; payload: { error: string } }
  | {
      type: 'MOVE_TASK';
      payload: {
        taskId: string;
        fromColumnId: string;
        toColumnId: string;
        toIndex: number;
      };
    }
  | { type: 'SET_DRAGGING'; payload: { taskId: string | null } }
  | {
      type: 'UPDATE_TASK';
      payload: { task: Task };
    }
  | { type: 'CLEAR_ERROR' };

/**
 * Kanban Reducer
 */
export function kanbanReducer(
  state: KanbanState,
  action: KanbanAction
): KanbanState {
  switch (action.type) {
    case 'LOAD_START':
      return {
        ...state,
        isLoading: true,
        error: null,
      };

    case 'LOAD_SUCCESS': {
      const { project, statusField, statusOptions, tasks } = action.payload;

      // タスクをMapに変換
      const tasksMap: Record<string, Task> = {};
      for (const task of tasks) {
        tasksMap[task.id] = task;
      }

      // カラムを生成
      const columns = createColumnsFromStatusOptions(statusOptions, tasks);

      return {
        ...state,
        project,
        statusField,
        columns,
        tasks: tasksMap,
        isLoading: false,
        error: null,
      };
    }

    case 'LOAD_ERROR':
      return {
        ...state,
        isLoading: false,
        error: action.payload.error,
      };

    case 'MOVE_TASK': {
      const { taskId, fromColumnId, toColumnId, toIndex } = action.payload;
      return moveTaskToColumn(state, taskId, fromColumnId, toColumnId, toIndex);
    }

    case 'SET_DRAGGING':
      return {
        ...state,
        draggingTaskId: action.payload.taskId,
      };

    case 'UPDATE_TASK': {
      const { task } = action.payload;
      return {
        ...state,
        tasks: {
          ...state.tasks,
          [task.id]: task,
        },
      };
    }

    case 'CLEAR_ERROR':
      return {
        ...state,
        error: null,
      };

    default:
      return state;
  }
}

/**
 * アクションクリエイター
 */
export const kanbanActions = {
  loadStart: (): KanbanAction => ({ type: 'LOAD_START' }),

  loadSuccess: (
    project: Project,
    statusField: StatusField | null,
    statusOptions: StatusOption[],
    tasks: Task[]
  ): KanbanAction => ({
    type: 'LOAD_SUCCESS',
    payload: { project, statusField, statusOptions, tasks },
  }),

  loadError: (error: string): KanbanAction => ({
    type: 'LOAD_ERROR',
    payload: { error },
  }),

  moveTask: (
    taskId: string,
    fromColumnId: string,
    toColumnId: string,
    toIndex: number
  ): KanbanAction => ({
    type: 'MOVE_TASK',
    payload: { taskId, fromColumnId, toColumnId, toIndex },
  }),

  setDragging: (taskId: string | null): KanbanAction => ({
    type: 'SET_DRAGGING',
    payload: { taskId },
  }),

  updateTask: (task: Task): KanbanAction => ({
    type: 'UPDATE_TASK',
    payload: { task },
  }),

  clearError: (): KanbanAction => ({ type: 'CLEAR_ERROR' }),
};
