/**
 * Project Page
 *
 * プロジェクトビュー - GitHub Projectsのカンバン表示
 */

import { useSessionStore } from '../../infra/state/sessionStore';
import { useBoardStore, boardSelectors } from '../../infra/state/boardStore';
import { useLoadProjectBootstrap } from '../hooks/useLoadBootstrap';
import { useMoveTask } from '../hooks/useMoveTask';
import { KanbanBoard } from '../components/kanban/KanbanBoard';
import { PageSpinner } from '../components/common/Spinner';
import type { Task } from '../../ui_domain/model/Task';

export function ProjectPage() {
  const selectedProject = useSessionStore((state) => state.selectedProject);
  const projectId = selectedProject?.id ?? null;

  const { data, isLoading, isError, errorMessage } = useLoadProjectBootstrap(projectId);
  const board = useBoardStore(
    projectId ? boardSelectors.board(projectId) : () => null
  );

  const { moveTask, changeTaskStatus } = useMoveTask({
    projectId: projectId ?? '',
    statusField: data?.statusField ?? null,
  });

  // プロジェクトが選択されていない場合
  if (!selectedProject) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-center">
          <svg
            className="mx-auto h-12 w-12 text-gray-400"
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
          <h3 className="mt-2 text-sm font-medium text-gray-900">
            プロジェクトを選択してください
          </h3>
          <p className="mt-1 text-sm text-gray-500">
            左のサイドバーからプロジェクトを選択してください
          </p>
        </div>
      </div>
    );
  }

  if (isLoading) {
    return <PageSpinner />;
  }

  if (isError) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-center">
          <p className="text-red-600">
            {errorMessage ?? 'プロジェクトの取得に失敗しました'}
          </p>
        </div>
      </div>
    );
  }

  if (!board) {
    return <PageSpinner />;
  }

  const handleMoveTask = async (
    taskId: string,
    fromColumnId: string,
    toColumnId: string,
    toIndex: number
  ) => {
    await moveTask(taskId, fromColumnId, toColumnId, toIndex);
  };

  const handleStatusChange = async (task: Task, newStatusOptionId: string) => {
    await changeTaskStatus(task, newStatusOptionId);
  };

  const handleOpenInGitHub = (task: Task) => {
    if (task.url) {
      window.open(task.url, '_blank');
    }
  };

  return (
    <div className="h-full">
      <KanbanBoard
        state={board}
        statusOptions={data?.statusOptions ?? []}
        onMoveTask={handleMoveTask}
        onStatusChange={handleStatusChange}
        onOpenInGitHub={handleOpenInGitHub}
      />
    </div>
  );
}
