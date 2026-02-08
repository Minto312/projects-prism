/**
 * Task Detail Component
 */

import type { Task } from '../../../ui_domain/model/Task';
import type { StatusOption } from '../../../ui_domain/model/Project';
import { getDueDateStatus } from '../../../ui_domain/model/Task';
import { Modal } from '../common/Modal';
import { Button } from '../common/Button';

export interface TaskDetailProps {
  task: Task | null;
  statusOptions: StatusOption[];
  isOpen: boolean;
  onClose: () => void;
  onStatusChange?: (task: Task, newStatusOptionId: string) => void;
  onOpenInGitHub?: (task: Task) => void;
}

const dueDateLabels: Record<string, string> = {
  overdue: '期限切れ',
  today: '今日が期限',
  upcoming: '期限',
  none: '',
};

const dueDateColors: Record<string, string> = {
  overdue: 'text-red-600',
  today: 'text-orange-600',
  upcoming: 'text-gray-600',
  none: 'text-gray-400',
};

export function TaskDetail({
  task,
  statusOptions,
  isOpen,
  onClose,
  onStatusChange,
  onOpenInGitHub,
}: TaskDetailProps) {
  if (!task) return null;

  const dueDateStatus = getDueDateStatus(task.dueDate);
  const currentStatus = statusOptions.find(
    (opt) => opt.id === task.statusOptionId
  );

  const formatDate = (dateStr: string | null): string => {
    if (!dateStr) return '未設定';
    const date = new Date(dateStr);
    return date.toLocaleDateString('ja-JP', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    });
  };

  const handleOpenInGitHub = () => {
    if (task.url) {
      onOpenInGitHub?.(task);
    }
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={task.title}
      size="lg"
      footer={
        <>
          {task.url && (
            <Button variant="secondary" onClick={handleOpenInGitHub}>
              GitHubで開く
            </Button>
          )}
          <Button variant="primary" onClick={onClose}>
            閉じる
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        {/* Status */}
        <div>
          <label className="block text-sm font-medium text-gray-700">
            ステータス
          </label>
          <select
            value={task.statusOptionId ?? ''}
            onChange={(e) => {
              if (onStatusChange && e.target.value) {
                onStatusChange(task, e.target.value);
              }
            }}
            className="mt-1 block w-full rounded-md border-gray-300 py-2 pl-3 pr-10 text-base focus:border-blue-500 focus:outline-none focus:ring-blue-500"
          >
            <option value="">No Status</option>
            {statusOptions.map((option) => (
              <option key={option.id} value={option.id}>
                {option.name}
              </option>
            ))}
          </select>
          {currentStatus && (
            <p className="mt-1 text-sm text-gray-500">
              現在: {currentStatus.name}
            </p>
          )}
        </div>

        {/* Due date */}
        <div>
          <label className="block text-sm font-medium text-gray-700">
            期限
          </label>
          <p className={`mt-1 ${dueDateColors[dueDateStatus]}`}>
            {dueDateLabels[dueDateStatus] && `${dueDateLabels[dueDateStatus]}: `}
            {formatDate(task.dueDate)}
          </p>
        </div>

        {/* Assignee */}
        <div>
          <label className="block text-sm font-medium text-gray-700">
            担当者
          </label>
          <p className="mt-1 text-gray-900">
            {task.assigneeLogin ?? '未割り当て'}
          </p>
        </div>

        {/* Body */}
        {task.body && (
          <div>
            <label className="block text-sm font-medium text-gray-700">
              説明
            </label>
            <div className="mt-1 rounded-md border border-gray-200 bg-gray-50 p-3">
              <p className="whitespace-pre-wrap text-sm text-gray-700">
                {task.body}
              </p>
            </div>
          </div>
        )}

        {/* Metadata */}
        <div className="border-t border-gray-200 pt-4">
          <dl className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <dt className="text-gray-500">タイプ</dt>
              <dd className="text-gray-900">{task.contentType}</dd>
            </div>
            {task.updatedAt && (
              <div>
                <dt className="text-gray-500">更新日時</dt>
                <dd className="text-gray-900">
                  {new Date(task.updatedAt).toLocaleString('ja-JP')}
                </dd>
              </div>
            )}
          </dl>
        </div>
      </div>
    </Modal>
  );
}
