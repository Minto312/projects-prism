/**
 * Sidebar Component
 */

import type { Project } from '../../../ui_domain/model/Project';
import { getProjectLabel } from '../../../ui_domain/model/Project';
import { useSessionStore } from '../../../infra/state/sessionStore';

export interface SidebarProps {
  projects: Project[];
  isLoading?: boolean;
}

export function Sidebar({ projects, isLoading = false }: SidebarProps) {
  const {
    currentView,
    selectedProject,
    isSidebarOpen,
    navigateToMyTasks,
    navigateToProject,
    navigateToSettings,
  } = useSessionStore();

  if (!isSidebarOpen) {
    return null;
  }

  return (
    <aside className="flex h-full w-64 flex-col border-r border-gray-200 bg-gray-50">
      {/* Logo / App Name */}
      <div className="flex h-14 items-center border-b border-gray-200 px-4">
        <h1 className="text-lg font-bold text-gray-900">Project Prism</h1>
      </div>

      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto p-4">
        {/* My Tasks */}
        <div className="mb-6">
          <button
            type="button"
            onClick={navigateToMyTasks}
            className={`flex w-full items-center rounded-md px-3 py-2 text-sm font-medium transition-colors ${
              currentView === 'myTasks'
                ? 'bg-blue-100 text-blue-700'
                : 'text-gray-700 hover:bg-gray-100'
            }`}
          >
            <svg
              className="mr-3 h-5 w-5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4"
              />
            </svg>
            マイタスク
          </button>
        </div>

        {/* Projects */}
        <div>
          <h2 className="mb-2 px-3 text-xs font-semibold uppercase tracking-wider text-gray-500">
            プロジェクト
          </h2>
          {isLoading ? (
            <div className="px-3 py-2 text-sm text-gray-500">読み込み中...</div>
          ) : projects.length === 0 ? (
            <div className="px-3 py-2 text-sm text-gray-500">
              プロジェクトがありません
            </div>
          ) : (
            <ul className="space-y-1">
              {projects.map((project) => (
                <li key={project.id}>
                  <button
                    type="button"
                    onClick={() => navigateToProject(project)}
                    className={`flex w-full items-center rounded-md px-3 py-2 text-sm transition-colors ${
                      currentView === 'project' &&
                      selectedProject?.id === project.id
                        ? 'bg-blue-100 text-blue-700'
                        : 'text-gray-700 hover:bg-gray-100'
                    }`}
                  >
                    <svg
                      className="mr-3 h-5 w-5 text-gray-400"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
                      />
                    </svg>
                    <span className="truncate">{getProjectLabel(project)}</span>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      </nav>

      {/* Settings */}
      <div className="border-t border-gray-200 p-4">
        <button
          type="button"
          onClick={navigateToSettings}
          className={`flex w-full items-center rounded-md px-3 py-2 text-sm font-medium transition-colors ${
            currentView === 'settings'
              ? 'bg-blue-100 text-blue-700'
              : 'text-gray-700 hover:bg-gray-100'
          }`}
        >
          <svg
            className="mr-3 h-5 w-5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
            />
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
            />
          </svg>
          設定
        </button>
      </div>
    </aside>
  );
}
