/**
 * Bootstrap Query
 *
 * Bootstrap データ取得の Query Hook
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { queryKeys } from './keys';
import { bootstrapApi } from '../tauri/client';
import type { BootstrapResponse, ProjectBootstrapResponse } from '../../application/dtos/BootstrapDto';

/**
 * グローバル Bootstrap を取得
 */
export function useBootstrapQuery() {
  return useQuery<BootstrapResponse, Error>({
    queryKey: queryKeys.bootstrap.global(),
    queryFn: () => bootstrapApi.getBootstrap(),
  });
}

/**
 * プロジェクト Bootstrap を取得
 */
export function useProjectBootstrapQuery(projectId: string | null) {
  return useQuery<ProjectBootstrapResponse, Error>({
    queryKey: queryKeys.bootstrap.project(projectId ?? ''),
    queryFn: () => bootstrapApi.getProjectBootstrap(projectId!),
    enabled: !!projectId,
  });
}

/**
 * グローバル Bootstrap をリフレッシュ
 */
export function useRefreshBootstrap() {
  const queryClient = useQueryClient();

  return useMutation<BootstrapResponse, Error>({
    mutationFn: () => bootstrapApi.refreshBootstrap(),
    onSuccess: (data) => {
      // キャッシュを更新
      queryClient.setQueryData(queryKeys.bootstrap.global(), data);
    },
  });
}

/**
 * プロジェクト Bootstrap をリフレッシュ
 */
export function useRefreshProjectBootstrap() {
  const queryClient = useQueryClient();

  return useMutation<ProjectBootstrapResponse, Error, string>({
    mutationFn: (projectId: string) =>
      bootstrapApi.refreshProjectBootstrap(projectId),
    onSuccess: (data, projectId) => {
      // キャッシュを更新
      queryClient.setQueryData(queryKeys.bootstrap.project(projectId), data);
    },
  });
}

/**
 * PAT が設定されているかチェック
 */
export function useHasPatQuery() {
  return useQuery<boolean, Error>({
    queryKey: queryKeys.settings.hasPat(),
    queryFn: async () => {
      const { settingsApi } = await import('../tauri/client');
      return settingsApi.hasPat();
    },
  });
}

/**
 * 非表示プロジェクトIDリストを取得
 */
export function useHiddenProjectIdsQuery() {
  return useQuery<string[], Error>({
    queryKey: queryKeys.settings.hiddenProjectIds(),
    queryFn: async () => {
      const { settingsApi } = await import('../tauri/client');
      return settingsApi.getHiddenProjectIds();
    },
  });
}

/**
 * 非表示プロジェクトIDリストを更新
 */
export function useSetHiddenProjectIds() {
  const queryClient = useQueryClient();

  return useMutation<void, Error, string[]>({
    mutationFn: async (ids: string[]) => {
      const { settingsApi } = await import('../tauri/client');
      return settingsApi.setHiddenProjectIds(ids);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.settings.hiddenProjectIds() });
    },
  });
}
