/**
 * Bootstrap Port
 *
 * 初期データ取得の抽象インターフェース
 */

import type { BootstrapResponse, ProjectBootstrapResponse } from '../dtos/BootstrapDto';

/**
 * Bootstrap Port インターフェース
 */
export interface BootstrapPort {
  /**
   * 全体のBootstrapデータを取得
   *
   * - 全プロジェクトの一覧
   * - 自分がアサインされているタスク
   * - 未同期の操作
   * - コンフリクト情報
   */
  getBootstrap(): Promise<BootstrapResponse>;

  /**
   * 特定プロジェクトの詳細Bootstrapデータを取得
   *
   * - プロジェクトの全タスク
   * - Statusフィールドと選択肢
   * - 未同期の操作（このプロジェクトのみ）
   */
  getProjectBootstrap(projectId: string): Promise<ProjectBootstrapResponse>;

  /**
   * キャッシュをリフレッシュ（GitHubから再取得）
   */
  refreshBootstrap(): Promise<BootstrapResponse>;

  /**
   * 特定プロジェクトのキャッシュをリフレッシュ
   */
  refreshProjectBootstrap(projectId: string): Promise<ProjectBootstrapResponse>;
}
