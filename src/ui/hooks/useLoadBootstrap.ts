/**
 * useLoadBootstrap Hook
 *
 * Bootstrap データの取得と管理
 */

import { useEffect, useCallback, useState } from 'react';
import { useBootstrapQuery, useProjectBootstrapQuery, useRefreshBootstrap, useRefreshProjectBootstrap } from '../../infra/query/bootstrapQuery';
import { useSessionStore } from '../../infra/state/sessionStore';
import { useBoardStore } from '../../infra/state/boardStore';
import { useOpQueueStore } from '../../infra/sync/opQueueCache';
import { useConflictStore, createConflictDetail } from '../../infra/sync/conflictStore';
import type { BootstrapResponse, ProjectBootstrapResponse } from '../../application/dtos/BootstrapDto';

/**
 * グローバルBootstrapの取得と初期化
 */
export function useLoadBootstrap() {
  const query = useBootstrapQuery();
  const refreshMutation = useRefreshBootstrap();
  const setCurrentUserLogin = useSessionStore((state) => state.setCurrentUserLogin);
  const setInitialized = useSessionStore((state) => state.setInitialized);
  const setOperations = useOpQueueStore((state) => state.setOperations);
  const setConflicts = useConflictStore((state) => state.setConflicts);

  const initializeFromBootstrap = useCallback((data: BootstrapResponse) => {
    // ユーザー情報を設定
    setCurrentUserLogin(data.currentUserLogin);

    // Pending operationsを設定
    setOperations(data.pendingOperations);

    // Conflictsを設定
    const conflictDetails = data.conflicts.map((conflict) =>
      createConflictDetail(conflict, '')
    );
    setConflicts(conflictDetails);

    // 初期化完了
    setInitialized(true);
  }, [setCurrentUserLogin, setOperations, setConflicts, setInitialized]);

  // Bootstrap成功時に初期化
  useEffect(() => {
    if (query.data) {
      initializeFromBootstrap(query.data);
    }
  }, [query.data, initializeFromBootstrap]);

  const refresh = async () => {
    const data = await refreshMutation.mutateAsync();
    initializeFromBootstrap(data);
    return data;
  };

  return {
    data: query.data,
    isLoading: query.isLoading,
    isError: query.isError,
    error: query.error,
    isRefreshing: refreshMutation.isPending,
    refresh,
  };
}

/**
 * プロジェクト詳細Bootstrapの取得と初期化
 */
export function useLoadProjectBootstrap(projectId: string | null) {
  const query = useProjectBootstrapQuery(projectId);
  const refreshMutation = useRefreshProjectBootstrap();
  const initializeBoard = useBoardStore((state) => state.initializeBoard);
  const setActiveProject = useBoardStore((state) => state.setActiveProject);
  const [initError, setInitError] = useState<string | null>(null);

  const initializeFromProjectBootstrap = useCallback((data: ProjectBootstrapResponse) => {
    initializeBoard(
      data.project,
      data.statusField,
      data.statusOptions,
      data.tasks
    );
    setActiveProject(data.project.id);
  }, [initializeBoard, setActiveProject]);

  // Bootstrap成功時にボードを初期化
  useEffect(() => {
    if (query.data && projectId) {
      try {
        setInitError(null);
        initializeFromProjectBootstrap(query.data);
      } catch (e) {
        console.error('Failed to initialize board:', e);
        setInitError(e instanceof Error ? e.message : String(e));
      }
    }
  }, [query.data, projectId, initializeFromProjectBootstrap]);

  // projectId変更時にinitErrorをリセット
  useEffect(() => {
    setInitError(null);
  }, [projectId]);

  const refresh = async () => {
    if (!projectId) return null;
    const data = await refreshMutation.mutateAsync(projectId);
    try {
      setInitError(null);
      initializeFromProjectBootstrap(data);
    } catch (e) {
      console.error('Failed to initialize board on refresh:', e);
      setInitError(e instanceof Error ? e.message : String(e));
    }
    return data;
  };

  // Tauriはエラーを文字列で返すため、Error.messageではなくそのまま取得
  const queryError = query.error;
  const errorMessage = initError
    ?? (queryError instanceof Error ? queryError.message : typeof queryError === 'string' ? queryError : null);

  return {
    data: query.data,
    isLoading: query.isLoading,
    isError: query.isError || initError !== null,
    errorMessage,
    isRefreshing: refreshMutation.isPending,
    refresh,
  };
}

/**
 * マイタスク用のBootstrapデータ取得
 */
export function useMyTasks() {
  const { data, isLoading, isError, error } = useLoadBootstrap();
  const currentUserLogin = useSessionStore((state) => state.currentUserLogin);

  // 自分にアサインされているタスクをフィルタ
  const myTasks = data?.tasks.filter(
    (task) => task.assigneeLogin === currentUserLogin
  ) ?? [];

  return {
    tasks: myTasks,
    projects: data?.projects ?? [],
    statusOptions: data?.statusOptions ?? [],
    isLoading,
    isError,
    error,
  };
}
