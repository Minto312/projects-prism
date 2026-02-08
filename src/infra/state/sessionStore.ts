/**
 * Session Store
 *
 * セッション状態（選択中プロジェクト、UIタブ等）を管理
 */

import { create } from 'zustand';
import type { Project } from '../../ui_domain/model/Project';

/**
 * 現在のビュー
 */
export type CurrentView = 'myTasks' | 'project' | 'settings';

/**
 * Session State
 */
export interface SessionState {
  /** 現在のビュー */
  currentView: CurrentView;
  /** 選択中のプロジェクト */
  selectedProject: Project | null;
  /** 認証済みユーザーのログイン名 */
  currentUserLogin: string | null;
  /** サイドバーの開閉状態 */
  isSidebarOpen: boolean;
  /** 初期化完了フラグ */
  isInitialized: boolean;
}

/**
 * Session Actions
 */
export interface SessionActions {
  /** マイタスクビューに切り替え */
  navigateToMyTasks: () => void;
  /** プロジェクトビューに切り替え */
  navigateToProject: (project: Project) => void;
  /** 設定画面に切り替え */
  navigateToSettings: () => void;
  /** プロジェクトを選択 */
  selectProject: (project: Project | null) => void;
  /** ユーザーログイン名を設定 */
  setCurrentUserLogin: (login: string | null) => void;
  /** サイドバーの開閉を切り替え */
  toggleSidebar: () => void;
  /** サイドバーの開閉状態を設定 */
  setSidebarOpen: (isOpen: boolean) => void;
  /** 初期化完了を設定 */
  setInitialized: (isInitialized: boolean) => void;
  /** セッションをリセット */
  reset: () => void;
}

/**
 * 初期状態
 */
const initialState: SessionState = {
  currentView: 'myTasks',
  selectedProject: null,
  currentUserLogin: null,
  isSidebarOpen: true,
  isInitialized: false,
};

/**
 * Session Store
 */
export const useSessionStore = create<SessionState & SessionActions>((set) => ({
  ...initialState,

  navigateToMyTasks: () =>
    set({
      currentView: 'myTasks',
      selectedProject: null,
    }),

  navigateToProject: (project) =>
    set({
      currentView: 'project',
      selectedProject: project,
    }),

  navigateToSettings: () =>
    set({
      currentView: 'settings',
    }),

  selectProject: (project) =>
    set({
      selectedProject: project,
    }),

  setCurrentUserLogin: (login) =>
    set({
      currentUserLogin: login,
    }),

  toggleSidebar: () =>
    set((state) => ({
      isSidebarOpen: !state.isSidebarOpen,
    })),

  setSidebarOpen: (isOpen) =>
    set({
      isSidebarOpen: isOpen,
    }),

  setInitialized: (isInitialized) =>
    set({
      isInitialized,
    }),

  reset: () => set(initialState),
}));

/**
 * セレクター
 */
export const sessionSelectors = {
  currentView: (state: SessionState) => state.currentView,
  selectedProject: (state: SessionState) => state.selectedProject,
  currentUserLogin: (state: SessionState) => state.currentUserLogin,
  isSidebarOpen: (state: SessionState) => state.isSidebarOpen,
  isInitialized: (state: SessionState) => state.isInitialized,
};
