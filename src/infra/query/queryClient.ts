/**
 * Query Client
 *
 * TanStack Query の設定
 */

import { QueryClient, QueryCache, MutationCache } from '@tanstack/react-query';
import { toDomainError } from '../../ui_domain/errors/parseBackendError';
import { useNotificationStore } from '../state/notificationStore';

/**
 * グローバルエラーハンドラ
 * CONFLICT_DETECTED はコンフリクト専用UIがあるためスキップ
 */
function handleGlobalError(error: unknown): void {
  const domainError = toDomainError(error);
  if (domainError.code === 'CONFLICT_DETECTED') return;
  useNotificationStore.getState().notify(domainError);
}

/**
 * Query Client のデフォルト設定
 */
export const queryClient = new QueryClient({
  queryCache: new QueryCache({
    onError: (error) => handleGlobalError(error),
  }),
  mutationCache: new MutationCache({
    onError: (error) => handleGlobalError(error),
  }),
  defaultOptions: {
    queries: {
      // データが古いとみなされるまでの時間（5分）
      staleTime: 5 * 60 * 1000,
      // キャッシュから削除されるまでの時間（30分）
      gcTime: 30 * 60 * 1000,
      // 自動リフェッチを無効化（オンデマンド更新のため）
      refetchOnWindowFocus: false,
      refetchOnReconnect: false,
      // リトライは1回まで
      retry: 1,
      // リトライ間隔
      retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 10000),
    },
    mutations: {
      // ミューテーションはリトライしない
      retry: false,
    },
  },
});

/**
 * Query Client を取得
 */
export function getQueryClient(): QueryClient {
  return queryClient;
}
