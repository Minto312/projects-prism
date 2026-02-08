/**
 * Header Component
 */

import { useSessionStore } from '../../../infra/state/sessionStore';
import { useOpQueueStore, opQueueSelectors } from '../../../infra/sync/opQueueCache';
import { Button } from '../common/Button';

export interface HeaderProps {
  onRefresh?: () => void;
  onSync?: () => void;
  isRefreshing?: boolean;
  isSyncing?: boolean;
}

export function Header({
  onRefresh,
  onSync,
  isRefreshing = false,
  isSyncing = false,
}: HeaderProps) {
  const { currentView, selectedProject, toggleSidebar } = useSessionStore();
  const pendingCount = useOpQueueStore(opQueueSelectors.pendingCount);
  const hasConflicts = useOpQueueStore(opQueueSelectors.hasConflicts);

  const getTitle = () => {
    switch (currentView) {
      case 'myTasks':
        return 'マイタスク';
      case 'project':
        return selectedProject
          ? `${selectedProject.ownerLogin}/${selectedProject.title}`
          : 'プロジェクト';
      case 'settings':
        return '設定';
      default:
        return '';
    }
  };

  return (
    <header className="flex h-14 items-center justify-between border-b border-gray-200 bg-white px-4">
      {/* Left section */}
      <div className="flex items-center">
        {/* Sidebar toggle */}
        <button
          type="button"
          onClick={toggleSidebar}
          className="mr-4 rounded-md p-2 text-gray-500 hover:bg-gray-100 hover:text-gray-700"
          aria-label="サイドバーを切り替え"
        >
          <svg
            className="h-5 w-5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M4 6h16M4 12h16M4 18h16"
            />
          </svg>
        </button>

        {/* Title */}
        <h1 className="text-lg font-semibold text-gray-900">{getTitle()}</h1>
      </div>

      {/* Right section */}
      <div className="flex items-center gap-3">
        {/* Sync status */}
        {(pendingCount > 0 || hasConflicts) && (
          <div className="flex items-center">
            {hasConflicts ? (
              <span className="flex items-center text-sm text-orange-600">
                <svg
                  className="mr-1 h-4 w-4"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                  />
                </svg>
                コンフリクト
              </span>
            ) : pendingCount > 0 ? (
              <span className="flex items-center text-sm text-gray-500">
                <svg
                  className="mr-1 h-4 w-4"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
                  />
                </svg>
                {pendingCount}件の未同期
              </span>
            ) : null}
          </div>
        )}

        {/* Sync button */}
        {onSync && pendingCount > 0 && (
          <Button
            variant="secondary"
            size="sm"
            onClick={onSync}
            isLoading={isSyncing}
            disabled={isSyncing}
          >
            同期
          </Button>
        )}

        {/* Refresh button */}
        {onRefresh && (
          <Button
            variant="ghost"
            size="sm"
            onClick={onRefresh}
            isLoading={isRefreshing}
            disabled={isRefreshing}
            aria-label="更新"
          >
            <svg
              className="h-5 w-5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
              />
            </svg>
          </Button>
        )}
      </div>
    </header>
  );
}
