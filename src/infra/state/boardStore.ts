/**
 * Board Store
 *
 * カンバンボード状態を管理
 */

import { create } from 'zustand';
import type { Task } from '../../ui_domain/model/Task';
import type { Project, StatusField, StatusOption } from '../../ui_domain/model/Project';
import type { KanbanState } from '../../ui_domain/model/KanbanState';
import {
  createColumnsFromStatusOptions,
  moveTaskToColumn,
} from '../../ui_domain/model/KanbanState';

/**
 * Board State（複数プロジェクトのボード状態を管理）
 */
export interface BoardStoreState {
  /** プロジェクトIDごとのKanbanState */
  boards: Record<string, KanbanState>;
  /** 現在アクティブなプロジェクトID */
  activeProjectId: string | null;
}

/**
 * Board Actions
 */
export interface BoardActions {
  /** ボードを初期化 */
  initializeBoard: (
    project: Project,
    statusField: StatusField | null,
    statusOptions: StatusOption[],
    tasks: Task[]
  ) => void;
  /** アクティブなプロジェクトを設定 */
  setActiveProject: (projectId: string | null) => void;
  /** タスクを移動（楽観的更新） */
  moveTask: (
    projectId: string,
    taskId: string,
    fromColumnId: string,
    toColumnId: string,
    toIndex: number
  ) => void;
  /** ドラッグ中のタスクを設定 */
  setDragging: (projectId: string, taskId: string | null) => void;
  /** タスクを更新 */
  updateTask: (projectId: string, task: Task) => void;
  /** ボードをクリア */
  clearBoard: (projectId: string) => void;
  /** 全ボードをクリア */
  clearAllBoards: () => void;
  /** ローディング状態を設定 */
  setLoading: (projectId: string, isLoading: boolean) => void;
  /** エラーを設定 */
  setError: (projectId: string, error: string | null) => void;
}

/**
 * 初期状態
 */
const initialState: BoardStoreState = {
  boards: {},
  activeProjectId: null,
};

/**
 * 空のKanbanStateを作成
 */
export function createEmptyBoard(project: Project): KanbanState {
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
 * Board Store
 */
export const useBoardStore = create<BoardStoreState & BoardActions>((set) => ({
  ...initialState,

  initializeBoard: (project, statusField, statusOptions, tasks) =>
    set((state) => {
      // タスクをMapに変換
      const tasksMap: Record<string, Task> = {};
      for (const task of tasks) {
        tasksMap[task.id] = task;
      }

      // カラムを生成
      const columns = createColumnsFromStatusOptions(statusOptions, tasks);

      const newBoard: KanbanState = {
        project,
        statusField,
        columns,
        tasks: tasksMap,
        draggingTaskId: null,
        isLoading: false,
        error: null,
      };

      return {
        boards: {
          ...state.boards,
          [project.id]: newBoard,
        },
      };
    }),

  setActiveProject: (projectId) =>
    set({
      activeProjectId: projectId,
    }),

  moveTask: (projectId, taskId, fromColumnId, toColumnId, toIndex) =>
    set((state) => {
      const board = state.boards[projectId];
      if (!board) return state;

      const newBoard = moveTaskToColumn(
        board,
        taskId,
        fromColumnId,
        toColumnId,
        toIndex
      );

      return {
        boards: {
          ...state.boards,
          [projectId]: newBoard,
        },
      };
    }),

  setDragging: (projectId, taskId) =>
    set((state) => {
      const board = state.boards[projectId];
      if (!board) return state;

      return {
        boards: {
          ...state.boards,
          [projectId]: {
            ...board,
            draggingTaskId: taskId,
          },
        },
      };
    }),

  updateTask: (projectId, task) =>
    set((state) => {
      const board = state.boards[projectId];
      if (!board) return state;

      return {
        boards: {
          ...state.boards,
          [projectId]: {
            ...board,
            tasks: {
              ...board.tasks,
              [task.id]: task,
            },
          },
        },
      };
    }),

  clearBoard: (projectId) =>
    set((state) => {
      const { [projectId]: _, ...rest } = state.boards;
      return {
        boards: rest,
        activeProjectId:
          state.activeProjectId === projectId ? null : state.activeProjectId,
      };
    }),

  clearAllBoards: () => set(initialState),

  setLoading: (projectId, isLoading) =>
    set((state) => {
      const board = state.boards[projectId];
      if (!board) return state;

      return {
        boards: {
          ...state.boards,
          [projectId]: {
            ...board,
            isLoading,
          },
        },
      };
    }),

  setError: (projectId, error) =>
    set((state) => {
      const board = state.boards[projectId];
      if (!board) return state;

      return {
        boards: {
          ...state.boards,
          [projectId]: {
            ...board,
            error,
          },
        },
      };
    }),
}));

/**
 * セレクター
 */
export const boardSelectors = {
  board: (projectId: string) => (state: BoardStoreState) =>
    state.boards[projectId],
  activeBoard: (state: BoardStoreState) =>
    state.activeProjectId ? state.boards[state.activeProjectId] : null,
  columns: (projectId: string) => (state: BoardStoreState) =>
    state.boards[projectId]?.columns ?? [],
  tasks: (projectId: string) => (state: BoardStoreState) =>
    state.boards[projectId]?.tasks ?? {},
  isLoading: (projectId: string) => (state: BoardStoreState) =>
    state.boards[projectId]?.isLoading ?? false,
  error: (projectId: string) => (state: BoardStoreState) =>
    state.boards[projectId]?.error ?? null,
};
