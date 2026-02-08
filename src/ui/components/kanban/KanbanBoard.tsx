/**
 * Kanban Board Component
 */

import { useState, useCallback } from 'react';
import type { KanbanState } from '../../../ui_domain/model/KanbanState';
import type { Task } from '../../../ui_domain/model/Task';
import type { StatusOption } from '../../../ui_domain/model/Project';
import { KanbanColumn } from './KanbanColumn';
import { TaskCard } from '../task/TaskCard';
import { TaskDetail } from '../task/TaskDetail';
import { PageSpinner } from '../common/Spinner';

export interface KanbanBoardProps {
  state: KanbanState;
  statusOptions: StatusOption[];
  onMoveTask: (
    taskId: string,
    fromColumnId: string,
    toColumnId: string,
    toIndex: number
  ) => void;
  onTaskClick?: (task: Task) => void;
  onStatusChange?: (task: Task, newStatusOptionId: string) => void;
  onOpenInGitHub?: (task: Task) => void;
}

export function KanbanBoard({
  state,
  statusOptions,
  onMoveTask,
  onTaskClick,
  onStatusChange,
  onOpenInGitHub,
}: KanbanBoardProps) {
  const [draggedTaskId, setDraggedTaskId] = useState<string | null>(null);
  const [draggedFromColumnId, setDraggedFromColumnId] = useState<string | null>(
    null
  );
  const [dragOverColumnId, setDragOverColumnId] = useState<string | null>(null);
  const [selectedTask, setSelectedTask] = useState<Task | null>(null);
  const [isDetailOpen, setIsDetailOpen] = useState(false);

  const handleDragStart = useCallback(
    (taskId: string, columnId: string) => (e: React.DragEvent) => {
      setDraggedTaskId(taskId);
      setDraggedFromColumnId(columnId);
      e.dataTransfer.effectAllowed = 'move';
      e.dataTransfer.setData('text/plain', taskId);
    },
    []
  );

  const handleDragEnd = useCallback(() => {
    setDraggedTaskId(null);
    setDraggedFromColumnId(null);
    setDragOverColumnId(null);
  }, []);

  const handleDragOver = useCallback(
    (columnId: string) => (e: React.DragEvent) => {
      e.preventDefault();
      e.dataTransfer.dropEffect = 'move';
      setDragOverColumnId(columnId);
    },
    []
  );

  const handleDragLeave = useCallback(() => {
    setDragOverColumnId(null);
  }, []);

  const handleDrop = useCallback(
    (columnId: string) => (e: React.DragEvent) => {
      e.preventDefault();
      setDragOverColumnId(null);

      if (draggedTaskId && draggedFromColumnId && columnId !== draggedFromColumnId) {
        // 同じカラム内での移動は無視
        // 新しいカラムの末尾に追加
        const targetColumn = state.columns.find((c) => c.id === columnId);
        const toIndex = targetColumn ? targetColumn.taskIds.length : 0;

        onMoveTask(draggedTaskId, draggedFromColumnId, columnId, toIndex);
      }

      handleDragEnd();
    },
    [draggedTaskId, draggedFromColumnId, state.columns, onMoveTask, handleDragEnd]
  );

  const handleTaskClick = useCallback(
    (task: Task) => {
      setSelectedTask(task);
      setIsDetailOpen(true);
      onTaskClick?.(task);
    },
    [onTaskClick]
  );

  const handleCloseDetail = useCallback(() => {
    setIsDetailOpen(false);
    setSelectedTask(null);
  }, []);

  const handleStatusChange = useCallback(
    (task: Task, newStatusOptionId: string) => {
      onStatusChange?.(task, newStatusOptionId);
      // 詳細を閉じる（移動後にUIを更新するため）
      handleCloseDetail();
    },
    [onStatusChange, handleCloseDetail]
  );

  if (state.isLoading) {
    return <PageSpinner />;
  }

  if (state.error) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-center">
          <p className="text-red-600">{state.error}</p>
        </div>
      </div>
    );
  }

  if (state.columns.length === 0) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-center">
          <p className="text-gray-500">
            このプロジェクトにはStatusフィールドがありません
          </p>
        </div>
      </div>
    );
  }

  return (
    <>
      <div className="flex h-full gap-4 overflow-x-auto pb-4">
        {state.columns.map((column) => {
          // カラム内のタスクを取得
          const columnTasks = column.taskIds
            .map((taskId) => state.tasks[taskId])
            .filter((task): task is Task => !!task);

          return (
            <KanbanColumn
              key={column.id}
              column={column}
              tasks={columnTasks}
              isDragOver={dragOverColumnId === column.id}
              onDragOver={handleDragOver(column.id)}
              onDrop={handleDrop(column.id)}
              renderTask={(task, _index) => (
                <div
                  draggable
                  onDragStart={handleDragStart(task.id, column.id)}
                  onDragEnd={handleDragEnd}
                  onDragLeave={handleDragLeave}
                >
                  <TaskCard
                    task={task}
                    onClick={() => handleTaskClick(task)}
                    isDragging={draggedTaskId === task.id}
                  />
                </div>
              )}
            />
          );
        })}
      </div>

      {/* Task Detail Modal */}
      <TaskDetail
        task={selectedTask}
        statusOptions={statusOptions}
        isOpen={isDetailOpen}
        onClose={handleCloseDetail}
        onStatusChange={handleStatusChange}
        onOpenInGitHub={onOpenInGitHub}
      />
    </>
  );
}
