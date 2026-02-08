/**
 * タスク（ProjectV2 Item）のUIドメインモデル
 */

export type ContentType = 'Issue' | 'DraftIssue' | 'PullRequest';

export interface Task {
  /** GitHub ProjectV2Item node ID */
  id: string;
  /** 所属プロジェクトID */
  projectId: string;
  /** コンテンツタイプ */
  contentType: ContentType;
  /** Issue/PR node ID（DraftIssueはnull） */
  contentId: string | null;
  /** タスクタイトル */
  title: string;
  /** タスク本文 */
  body: string | null;
  /** 現在のStatus option ID */
  statusOptionId: string | null;
  /** 担当者のログイン名（MVP: 単一担当のみ） */
  assigneeLogin: string | null;
  /** 期限（YYYY-MM-DD形式） */
  dueDate: string | null;
  /** GitHub上のURL */
  url: string | null;
  /** ProjectV2Item.updatedAt（epoch ms） */
  updatedAt: number | null;
  /** 最後に同期した日時（epoch ms） */
  syncedAt: number | null;
}

/**
 * タスク作成用の入力型
 */
export type TaskInput = Omit<Task, 'id' | 'syncedAt'>;

/**
 * タスクの期限状態を判定
 */
export function getDueDateStatus(
  dueDate: string | null,
  now: Date = new Date()
): 'overdue' | 'today' | 'upcoming' | 'none' {
  if (!dueDate) return 'none';

  const due = new Date(dueDate);
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const dueDay = new Date(due.getFullYear(), due.getMonth(), due.getDate());

  const diffDays = Math.floor(
    (dueDay.getTime() - today.getTime()) / (1000 * 60 * 60 * 24)
  );

  if (diffDays < 0) return 'overdue';
  if (diffDays === 0) return 'today';
  return 'upcoming';
}
