/**
 * Kanban Column Component
 */

import type { ReactNode } from 'react';
import type { KanbanColumn as KanbanColumnType } from '../../../ui_domain/model/KanbanState';
import type { Task } from '../../../ui_domain/model/Task';

export interface KanbanColumnProps {
  column: KanbanColumnType;
  tasks: Task[];
  renderTask: (task: Task, index: number) => ReactNode;
  onDragOver?: (e: React.DragEvent<HTMLDivElement>) => void;
  onDrop?: (e: React.DragEvent<HTMLDivElement>) => void;
  isDragOver?: boolean;
}

const columnColors: Record<string, string> = {
  // GitHub Projects のデフォルトカラー
  GRAY: 'bg-gray-100 border-gray-300',
  RED: 'bg-red-100 border-red-300',
  ORANGE: 'bg-orange-100 border-orange-300',
  YELLOW: 'bg-yellow-100 border-yellow-300',
  GREEN: 'bg-green-100 border-green-300',
  BLUE: 'bg-blue-100 border-blue-300',
  PURPLE: 'bg-purple-100 border-purple-300',
  PINK: 'bg-pink-100 border-pink-300',
};

export function KanbanColumn({
  column,
  tasks,
  renderTask,
  onDragOver,
  onDrop,
  isDragOver = false,
}: KanbanColumnProps) {
  const _colorClass =
    column.color && columnColors[column.color.toUpperCase()]
      ? columnColors[column.color.toUpperCase()]
      : 'bg-gray-100 border-gray-300';
  void _colorClass; // Future use: column background styling

  return (
    <div
      className={`flex h-full w-72 flex-shrink-0 flex-col rounded-lg border-t-4 bg-gray-50 ${
        isDragOver ? 'ring-2 ring-blue-400' : ''
      }`}
      style={{
        borderTopColor: column.color
          ? `var(--color-${column.color.toLowerCase()}, #9ca3af)`
          : '#9ca3af',
      }}
      onDragOver={onDragOver}
      onDrop={onDrop}
    >
      {/* Column header */}
      <div className="flex items-center justify-between p-3">
        <div className="flex items-center gap-2">
          <h3 className="font-medium text-gray-900">{column.name}</h3>
          <span className="rounded-full bg-gray-200 px-2 py-0.5 text-xs font-medium text-gray-700">
            {column.taskIds.length}
          </span>
        </div>
      </div>

      {/* Tasks list */}
      <div className="flex-1 overflow-y-auto p-2">
        <div className="space-y-2">
          {tasks.length === 0 ? (
            <div className="py-8 text-center text-sm text-gray-400">
              タスクがありません
            </div>
          ) : (
            tasks.map((task, index) => (
              <div key={task.id}>{renderTask(task, index)}</div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
