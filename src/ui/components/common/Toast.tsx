/**
 * Toast Component
 *
 * リトライ可能なエラーを一時的に表示する通知
 */

import { useEffect } from 'react';
import { createPortal } from 'react-dom';
import {
  useNotificationStore,
  notificationSelectors,
  type ToastItem,
} from '../../../infra/state/notificationStore';

/** Toast の自動 dismiss 時間（ミリ秒） */
const TOAST_DURATION = 5000;

/**
 * 個々の Toast アイテム
 */
function ToastItemView({ toast }: { toast: ToastItem }) {
  const dismissToast = useNotificationStore((state) => state.dismissToast);

  useEffect(() => {
    const timer = setTimeout(() => {
      dismissToast(toast.id);
    }, TOAST_DURATION);
    return () => clearTimeout(timer);
  }, [toast.id, dismissToast]);

  return (
    <div
      role="alert"
      aria-live="polite"
      className="animate-slide-in flex items-start gap-3 rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 shadow-lg"
    >
      <div className="flex-1">
        <p className="text-sm font-medium text-amber-800">{toast.message}</p>
      </div>
      <button
        type="button"
        className="rounded-md p-0.5 text-amber-400 hover:text-amber-600"
        onClick={() => dismissToast(toast.id)}
        aria-label="閉じる"
      >
        <svg
          className="h-4 w-4"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M6 18L18 6M6 6l12 12"
          />
        </svg>
      </button>
    </div>
  );
}

/**
 * Toast コンテナ — 画面右上に固定表示
 */
export function ToastContainer() {
  const toasts = useNotificationStore(notificationSelectors.toasts);

  if (toasts.length === 0) return null;

  return createPortal(
    <div className="fixed right-4 top-4 z-[60] flex w-80 flex-col gap-2">
      {toasts.map((toast) => (
        <ToastItemView key={toast.id} toast={toast} />
      ))}
    </div>,
    document.body
  );
}
