/**
 * プロジェクト（GitHub Projects V2）のUIドメインモデル
 */

export type OwnerType = 'organization' | 'user';

export interface Project {
  /** GitHub ProjectV2 node ID */
  id: string;
  /** オーナータイプ */
  ownerType: OwnerType;
  /** オーナーのログイン名 */
  ownerLogin: string;
  /** プロジェクトタイトル */
  title: string;
  /** GitHub上のURL */
  url: string;
  /** キャッシュ無効化判断用の更新日時（epoch ms） */
  updatedAt: number | null;
  /** 最後に同期した日時（epoch ms） */
  syncedAt: number | null;
}

/**
 * Statusフィールド定義
 */
export interface StatusField {
  /** GitHub SingleSelectField node ID */
  id: string;
  /** 所属プロジェクトID */
  projectId: string;
  /** フィールド名 */
  name: string;
}

/**
 * Status選択肢
 */
export interface StatusOption {
  /** GitHub SingleSelectOption node ID */
  id: string;
  /** 所属StatusフィールドID */
  statusFieldId: string;
  /** 選択肢名 */
  name: string;
  /** 色（GitHub上の色名） */
  color: string | null;
  /** 表示順序 */
  position: number;
}

/**
 * プロジェクトの完全な状態（Statusフィールド込み）
 */
export interface ProjectWithStatus {
  project: Project;
  statusField: StatusField | null;
  statusOptions: StatusOption[];
}

/**
 * プロジェクト表示用のラベルを生成
 */
export function getProjectLabel(project: Project): string {
  return `${project.ownerLogin}/${project.title}`;
}
