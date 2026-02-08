/**
 * Task Card Component
 */

import type { Task } from '../../../ui_domain/model/Task';
import { getDueDateStatus } from '../../../ui_domain/model/Task';

export interface TaskCardProps {
  task: Task;
  onClick?: () => void;
  isDragging?: boolean;
  draggableProps?: Record<string, unknown>;
  dragHandleProps?: Record<string, unknown>;
}

const dueDateStyles: Record<string, string> = {
  overdue: 'text-red-600 bg-red-50',
  today: 'text-orange-600 bg-orange-50',
  upcoming: 'text-gray-600 bg-gray-100',
  none: 'text-gray-400',
};

const contentTypeIcons: Record<string, JSX.Element> = {
  Issue: (
    <svg className="h-4 w-4 text-green-600" viewBox="0 0 16 16" fill="currentColor">
      <path d="M8 9.5a1.5 1.5 0 100-3 1.5 1.5 0 000 3z" />
      <path
        fillRule="evenodd"
        d="M8 0a8 8 0 100 16A8 8 0 008 0zM1.5 8a6.5 6.5 0 1113 0 6.5 6.5 0 01-13 0z"
      />
    </svg>
  ),
  PullRequest: (
    <svg className="h-4 w-4 text-purple-600" viewBox="0 0 16 16" fill="currentColor">
      <path
        fillRule="evenodd"
        d="M7.177 3.073L9.573.677A.25.25 0 0110 .854v4.792a.25.25 0 01-.427.177L7.177 3.427a.25.25 0 010-.354zM3.75 2.5a.75.75 0 100 1.5.75.75 0 000-1.5zm-2.25.75a2.25 2.25 0 113 2.122v5.256a2.251 2.251 0 11-1.5 0V5.372A2.25 2.25 0 011.5 3.25zM11 2.5h-1V4h1a1 1 0 011 1v5.628a2.251 2.251 0 101.5 0V5A2.5 2.5 0 0011 2.5zm1 10.25a.75.75 0 111.5 0 .75.75 0 01-1.5 0zM3.75 12a.75.75 0 100 1.5.75.75 0 000-1.5z"
      />
    </svg>
  ),
  DraftIssue: (
    <svg className="h-4 w-4 text-gray-400" viewBox="0 0 16 16" fill="currentColor">
      <path d="M8 9.5a1.5 1.5 0 100-3 1.5 1.5 0 000 3z" />
      <path
        fillRule="evenodd"
        d="M8 0a8 8 0 100 16A8 8 0 008 0zM1.5 8a6.5 6.5 0 1113 0 6.5 6.5 0 01-13 0z"
        opacity={0.5}
      />
    </svg>
  ),
};

export function TaskCard({
  task,
  onClick,
  isDragging = false,
  draggableProps,
  dragHandleProps,
}: TaskCardProps) {
  const dueDateStatus = getDueDateStatus(task.dueDate);

  const formatDueDate = (dateStr: string | null): string => {
    if (!dateStr) return '';
    const date = new Date(dateStr);
    return `${date.getMonth() + 1}/${date.getDate()}`;
  };

  return (
    <div
      className={`rounded-lg border bg-white p-3 shadow-sm transition-shadow hover:shadow-md ${
        isDragging ? 'opacity-50 shadow-lg' : ''
      }`}
      onClick={onClick}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          onClick?.();
        }
      }}
      {...draggableProps}
      {...dragHandleProps}
    >
      {/* Header: Content type icon + Title */}
      <div className="mb-2 flex items-start gap-2">
        <span className="mt-0.5 flex-shrink-0">
          {contentTypeIcons[task.contentType]}
        </span>
        <h3 className="text-sm font-medium text-gray-900 line-clamp-2">
          {task.title}
        </h3>
      </div>

      {/* Footer: Due date + Assignee */}
      <div className="flex items-center justify-between text-xs">
        {/* Due date */}
        {task.dueDate ? (
          <span
            className={`rounded px-1.5 py-0.5 font-medium ${dueDateStyles[dueDateStatus]}`}
          >
            {dueDateStatus === 'overdue' && '期限切れ: '}
            {dueDateStatus === 'today' && '今日: '}
            {formatDueDate(task.dueDate)}
          </span>
        ) : (
          <span />
        )}

        {/* Assignee */}
        {task.assigneeLogin && (
          <span className="flex items-center text-gray-500">
            <svg
              className="mr-1 h-3 w-3"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
              />
            </svg>
            {task.assigneeLogin}
          </span>
        )}
      </div>
    </div>
  );
}
