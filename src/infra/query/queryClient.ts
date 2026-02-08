/**
 * Query Client
 *
 * TanStack Query の設定
 */

import { QueryClient } from '@tanstack/react-query';

/**
 * Query Client のデフォルト設定
 */
export const queryClient = new QueryClient({
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
