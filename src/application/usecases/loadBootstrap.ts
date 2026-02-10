/**
 * loadBootstrap Usecase
 *
 * 初期データ取得のユースケース（純粋関数）
 */

import type { BootstrapPort } from '../ports/BootstrapPort';
import type { BootstrapResponse, ProjectBootstrapResponse } from '../dtos/BootstrapDto';
import type { Task } from '../../ui_domain/model/Task';
import type { Project, StatusOption } from '../../ui_domain/model/Project';

/**
 * マイタスクのグルーピング方法
 */
export type MyTasksGrouping = 'dueDate' | 'project' | 'status' | 'priority';

/**
 * グループ化されたタスク
 */
export interface GroupedTasks {
  groupKey: string;
  groupLabel: string;
  tasks: Task[];
}

/**
 * Bootstrapを実行
 */
export async function loadBootstrap(
  port: BootstrapPort
): Promise<BootstrapResponse> {
  return port.getBootstrap();
}

/**
 * プロジェクト詳細のBootstrapを実行
 */
export async function loadProjectBootstrap(
  port: BootstrapPort,
  projectId: string
): Promise<ProjectBootstrapResponse> {
  return port.getProjectBootstrap(projectId);
}

/**
 * Bootstrapをリフレッシュ（GitHubから再取得）
 */
export async function refreshBootstrap(
  port: BootstrapPort
): Promise<BootstrapResponse> {
  return port.refreshBootstrap();
}

/**
 * プロジェクト詳細をリフレッシュ
 */
export async function refreshProjectBootstrap(
  port: BootstrapPort,
  projectId: string
): Promise<ProjectBootstrapResponse> {
  return port.refreshProjectBootstrap(projectId);
}

/**
 * マイタスクを期限順でグループ化
 */
export function groupTasksByDueDate(tasks: Task[]): GroupedTasks[] {
  const today = new Date();
  today.setHours(0, 0, 0, 0);

  const overdue: Task[] = [];
  const todayTasks: Task[] = [];
  const thisWeek: Task[] = [];
  const later: Task[] = [];
  const noDueDate: Task[] = [];

  const oneWeekLater = new Date(today);
  oneWeekLater.setDate(oneWeekLater.getDate() + 7);

  for (const task of tasks) {
    if (!task.dueDate) {
      noDueDate.push(task);
      continue;
    }

    const dueDate = new Date(task.dueDate);
    dueDate.setHours(0, 0, 0, 0);

    if (dueDate < today) {
      overdue.push(task);
    } else if (dueDate.getTime() === today.getTime()) {
      todayTasks.push(task);
    } else if (dueDate < oneWeekLater) {
      thisWeek.push(task);
    } else {
      later.push(task);
    }
  }

  // 各グループ内を期限順でソート
  const sortByDueDate = (a: Task, b: Task) => {
    if (!a.dueDate) return 1;
    if (!b.dueDate) return -1;
    return a.dueDate.localeCompare(b.dueDate);
  };

  overdue.sort(sortByDueDate);
  todayTasks.sort(sortByDueDate);
  thisWeek.sort(sortByDueDate);
  later.sort(sortByDueDate);

  const groups: GroupedTasks[] = [];

  if (overdue.length > 0) {
    groups.push({ groupKey: 'overdue', groupLabel: '期限切れ', tasks: overdue });
  }
  if (todayTasks.length > 0) {
    groups.push({ groupKey: 'today', groupLabel: '今日', tasks: todayTasks });
  }
  if (thisWeek.length > 0) {
    groups.push({ groupKey: 'thisWeek', groupLabel: '今週', tasks: thisWeek });
  }
  if (later.length > 0) {
    groups.push({ groupKey: 'later', groupLabel: '今後', tasks: later });
  }
  if (noDueDate.length > 0) {
    groups.push({ groupKey: 'noDueDate', groupLabel: '期限なし', tasks: noDueDate });
  }

  return groups;
}

/**
 * マイタスクをプロジェクト別でグループ化
 */
export function groupTasksByProject(
  tasks: Task[],
  projects: Project[]
): GroupedTasks[] {
  const projectMap = new Map(projects.map((p) => [p.id, p]));
  const tasksByProject = new Map<string, Task[]>();

  for (const task of tasks) {
    const existing = tasksByProject.get(task.projectId) ?? [];
    existing.push(task);
    tasksByProject.set(task.projectId, existing);
  }

  const groups: GroupedTasks[] = [];

  for (const [projectId, projectTasks] of tasksByProject) {
    const project = projectMap.get(projectId);
    const label = project
      ? `${project.ownerLogin}/${project.title}`
      : 'Unknown Project';

    groups.push({
      groupKey: projectId,
      groupLabel: label,
      tasks: projectTasks,
    });
  }

  return groups;
}

/**
 * マイタスクをステータス別でグループ化
 */
export function groupTasksByStatus(
  tasks: Task[],
  statusOptions: StatusOption[]
): GroupedTasks[] {
  const tasksByStatus = new Map<string, Task[]>();

  for (const task of tasks) {
    const statusKey = task.statusOptionId ?? '__no_status__';
    const existing = tasksByStatus.get(statusKey) ?? [];
    existing.push(task);
    tasksByStatus.set(statusKey, existing);
  }

  const groups: GroupedTasks[] = [];

  // Statusの順序でグループを生成
  const sortedOptions = [...statusOptions].sort((a, b) => a.position - b.position);

  for (const option of sortedOptions) {
    const statusTasks = tasksByStatus.get(option.id);
    if (statusTasks && statusTasks.length > 0) {
      groups.push({
        groupKey: option.id,
        groupLabel: option.name,
        tasks: statusTasks,
      });
    }
  }

  // Statusなしのタスク
  const noStatusTasks = tasksByStatus.get('__no_status__');
  if (noStatusTasks && noStatusTasks.length > 0) {
    groups.push({
      groupKey: '__no_status__',
      groupLabel: 'No Status',
      tasks: noStatusTasks,
    });
  }

  return groups;
}

/**
 * 自分がアサインされているタスクをフィルタ
 */
export function filterMyTasks(tasks: Task[], currentUserLogin: string): Task[] {
  return tasks.filter((task) => task.assigneeLogin === currentUserLogin);
}
