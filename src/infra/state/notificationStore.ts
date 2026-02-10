/**
 * Notification Store
 *
 * エラー通知状態の管理
 * isRetryable() で Toast / Error Dialog に自動振り分け
 */

import { create } from 'zustand';
import type { DomainError } from '../../ui_domain/errors/DomainError';

/**
 * Toast アイテム
 */
export interface ToastItem {
  /** 一意なID */
  id: string;
  /** エラーメッセージ */
  message: string;
  /** エラーコード（デバッグ用） */
  code: string;
  /** 作成時刻 */
  createdAt: number;
}

/** Toast の最大表示件数 */
const MAX_TOASTS = 5;

let toastIdCounter = 0;

/**
 * Notification State
 */
export interface NotificationState {
  /** 表示中の Toast 一覧 */
  toasts: ToastItem[];
  /** Error Dialog に表示するエラー */
  dialogError: DomainError | null;
  /** Error Dialog の表示状態 */
  isDialogOpen: boolean;
}

/**
 * Notification Actions
 */
export interface NotificationActions {
  /** エラーを通知（isRetryable で自動振り分け） */
  notify: (error: DomainError) => void;
  /** Toast を削除 */
  dismissToast: (id: string) => void;
  /** Error Dialog を閉じる */
  closeDialog: () => void;
}

/**
 * 初期状態
 */
const initialState: NotificationState = {
  toasts: [],
  dialogError: null,
  isDialogOpen: false,
};

/**
 * Notification Store
 */
export const useNotificationStore = create<
  NotificationState & NotificationActions
>((set) => ({
  ...initialState,

  notify: (error) => {
    if (error.isRetryable()) {
      // リトライ可能 → Toast
      const toast: ToastItem = {
        id: `toast-${++toastIdCounter}`,
        message: error.getUserMessage(),
        code: error.code,
        createdAt: Date.now(),
      };
      set((state) => ({
        toasts: [...state.toasts, toast].slice(-MAX_TOASTS),
      }));
    } else {
      // リトライ不可 → Error Dialog
      set({
        dialogError: error,
        isDialogOpen: true,
      });
    }
  },

  dismissToast: (id) =>
    set((state) => ({
      toasts: state.toasts.filter((t) => t.id !== id),
    })),

  closeDialog: () =>
    set({
      dialogError: null,
      isDialogOpen: false,
    }),
}));

/**
 * セレクター
 */
export const notificationSelectors = {
  toasts: (state: NotificationState) => state.toasts,
  dialogError: (state: NotificationState) => state.dialogError,
  isDialogOpen: (state: NotificationState) => state.isDialogOpen,
};
