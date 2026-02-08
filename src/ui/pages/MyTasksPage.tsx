/**
 * My Tasks Page
 *
 * マイタスクビュー - 全プロジェクト横断で自分がアサインされているタスクを表示
 */

import { useMemo } from 'react';
import { useMyTasks } from '../hooks/useLoadBootstrap';
import { groupTasksByDueDate, type GroupedTasks } from '../../app/usecases/loadBootstrap';
import { TaskCard } from '../components/task/TaskCard';
import { PageSpinner } from '../components/common/Spinner';

export function MyTasksPage() {
  const { tasks, isLoading, isError, error } = useMyTasks();

  // 期限順でグループ化
  const groupedTasks = useMemo<GroupedTasks[]>(() => {
    return groupTasksByDueDate(tasks);
  }, [tasks]);

  if (isLoading) {
    return <PageSpinner />;
  }

  if (isError) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-center">
          <p className="text-red-600">
            {error?.message ?? 'データの取得に失敗しました'}
          </p>
        </div>
      </div>
    );
  }

  if (tasks.length === 0) {
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
              d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4"
            />
          </svg>
          <h3 className="mt-2 text-sm font-medium text-gray-900">
            タスクがありません
          </h3>
          <p className="mt-1 text-sm text-gray-500">
            あなたにアサインされているタスクはありません
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-8">
      {groupedTasks.map((group) => (
        <section key={group.groupKey}>
          {/* Group header */}
          <div className="mb-4 flex items-center gap-2">
            <h2 className="text-lg font-semibold text-gray-900">
              {group.groupLabel}
            </h2>
            <span className="rounded-full bg-gray-200 px-2 py-0.5 text-sm font-medium text-gray-700">
              {group.tasks.length}
            </span>
          </div>

          {/* Task list */}
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            {group.tasks.map((task) => (
              <TaskCard
                key={task.id}
                task={task}
                onClick={() => {
                  // タスク詳細の表示は将来のバージョンで実装予定
                }}
              />
            ))}
          </div>
        </section>
      ))}
    </div>
  );
}
