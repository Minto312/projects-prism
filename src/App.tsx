/**
 * App Component
 *
 * アプリケーションのルートコンポーネント
 */

import { useSessionStore } from './infra/state/sessionStore';
import { useLoadBootstrap } from './ui/hooks/useLoadBootstrap';
import { useSyncOperations } from './ui/hooks/useSyncOperations';
import { MainLayout } from './ui/components/layout/MainLayout';
import { MyTasksPage } from './ui/pages/MyTasksPage';
import { ProjectPage } from './ui/pages/ProjectPage';
import { SettingsPage } from './ui/pages/SettingsPage';
import { PageSpinner } from './ui/components/common/Spinner';
import { ErrorBoundary } from './ui/components/common/ErrorBoundary';

function AppContent() {
  const currentView = useSessionStore((state) => state.currentView);
  const selectedProject = useSessionStore((state) => state.selectedProject);
  const isInitialized = useSessionStore((state) => state.isInitialized);

  const { data, isLoading, isRefreshing, refresh } = useLoadBootstrap();
  const { syncNow, isSyncing, pendingCount } = useSyncOperations();

  // 初期化中
  if (isLoading && !isInitialized) {
    return (
      <div className="flex h-screen items-center justify-center">
        <div className="text-center">
          <PageSpinner />
          <p className="mt-4 text-gray-600">読み込み中...</p>
        </div>
      </div>
    );
  }

  const handleRefresh = async () => {
    await refresh();
  };

  const handleSync = async () => {
    if (pendingCount > 0) {
      await syncNow();
    }
  };

  const renderPage = () => {
    switch (currentView) {
      case 'myTasks':
        return <MyTasksPage />;
      case 'project':
        return <ProjectPage />;
      case 'settings':
        return <SettingsPage />;
      default:
        return <MyTasksPage />;
    }
  };

  return (
    <MainLayout
      projects={data?.projects ?? []}
      isProjectsLoading={isLoading}
      onRefresh={handleRefresh}
      onSync={handleSync}
      isRefreshing={isRefreshing}
      isSyncing={isSyncing}
    >
      <ErrorBoundary key={`${currentView}-${selectedProject?.id ?? ''}`}>
        {renderPage()}
      </ErrorBoundary>
    </MainLayout>
  );
}

export function App() {
  return <AppContent />;
}

export default App;
