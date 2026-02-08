/**
 * Main Layout Component
 */

import type { ReactNode } from 'react';
import type { Project } from '../../../ui_domain/model/Project';
import { Sidebar } from './Sidebar';
import { Header } from './Header';

export interface MainLayoutProps {
  children: ReactNode;
  projects: Project[];
  isProjectsLoading?: boolean;
  onRefresh?: () => void;
  onSync?: () => void;
  isRefreshing?: boolean;
  isSyncing?: boolean;
}

export function MainLayout({
  children,
  projects,
  isProjectsLoading = false,
  onRefresh,
  onSync,
  isRefreshing = false,
  isSyncing = false,
}: MainLayoutProps) {
  return (
    <div className="flex h-screen bg-white">
      {/* Sidebar */}
      <Sidebar projects={projects} isLoading={isProjectsLoading} />

      {/* Main content */}
      <div className="flex flex-1 flex-col overflow-hidden">
        {/* Header */}
        <Header
          onRefresh={onRefresh}
          onSync={onSync}
          isRefreshing={isRefreshing}
          isSyncing={isSyncing}
        />

        {/* Content area */}
        <main className="flex-1 overflow-auto bg-gray-50 p-6">{children}</main>
      </div>
    </div>
  );
}
