/**
 * Query Keys
 *
 * TanStack Query のキー定義
 */

/**
 * Query Keys ファクトリ
 */
export const queryKeys = {
  /**
   * Bootstrap 関連
   */
  bootstrap: {
    all: ['bootstrap'] as const,
    global: () => [...queryKeys.bootstrap.all, 'global'] as const,
    project: (projectId: string) =>
      [...queryKeys.bootstrap.all, 'project', projectId] as const,
  },

  /**
   * プロジェクト関連
   */
  projects: {
    all: ['projects'] as const,
    list: () => [...queryKeys.projects.all, 'list'] as const,
    detail: (projectId: string) =>
      [...queryKeys.projects.all, 'detail', projectId] as const,
  },

  /**
   * タスク関連
   */
  tasks: {
    all: ['tasks'] as const,
    list: () => [...queryKeys.tasks.all, 'list'] as const,
    myTasks: () => [...queryKeys.tasks.all, 'myTasks'] as const,
    byProject: (projectId: string) =>
      [...queryKeys.tasks.all, 'byProject', projectId] as const,
    detail: (taskId: string) =>
      [...queryKeys.tasks.all, 'detail', taskId] as const,
  },

  /**
   * 同期関連
   */
  sync: {
    all: ['sync'] as const,
    state: () => [...queryKeys.sync.all, 'state'] as const,
  },

  /**
   * 設定関連
   */
  settings: {
    all: ['settings'] as const,
    hasPat: () => [...queryKeys.settings.all, 'hasPat'] as const,
    hiddenProjectIds: () => [...queryKeys.settings.all, 'hiddenProjectIds'] as const,
  },
} as const;

/**
 * 全キャッシュを無効化
 */
export function invalidateAllQueries(queryClient: {
  invalidateQueries: (options: { queryKey: readonly string[] }) => Promise<void>;
}) {
  return Promise.all([
    queryClient.invalidateQueries({ queryKey: queryKeys.bootstrap.all }),
    queryClient.invalidateQueries({ queryKey: queryKeys.projects.all }),
    queryClient.invalidateQueries({ queryKey: queryKeys.tasks.all }),
    queryClient.invalidateQueries({ queryKey: queryKeys.sync.all }),
  ]);
}
