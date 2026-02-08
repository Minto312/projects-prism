/**
 * UIドメインのエラー定義
 */

/**
 * エラーコード
 */
export type DomainErrorCode =
  | 'TASK_NOT_FOUND'
  | 'PROJECT_NOT_FOUND'
  | 'COLUMN_NOT_FOUND'
  | 'INVALID_OPERATION'
  | 'CONFLICT_DETECTED'
  | 'SYNC_FAILED'
  | 'NETWORK_ERROR'
  | 'RATE_LIMITED'
  | 'UNAUTHORIZED'
  | 'UNKNOWN';

/**
 * ドメインエラー
 */
export class DomainError extends Error {
  readonly code: DomainErrorCode;
  readonly cause?: Error;

  constructor(code: DomainErrorCode, message: string, cause?: Error) {
    super(message);
    this.name = 'DomainError';
    this.code = code;
    this.cause = cause;
  }

  /**
   * エラーがリトライ可能かどうか
   */
  isRetryable(): boolean {
    return this.code === 'NETWORK_ERROR' || this.code === 'RATE_LIMITED';
  }

  /**
   * ユーザーに表示するメッセージを取得
   */
  getUserMessage(): string {
    switch (this.code) {
      case 'TASK_NOT_FOUND':
        return 'タスクが見つかりませんでした';
      case 'PROJECT_NOT_FOUND':
        return 'プロジェクトが見つかりませんでした';
      case 'COLUMN_NOT_FOUND':
        return 'カラムが見つかりませんでした';
      case 'INVALID_OPERATION':
        return '無効な操作です';
      case 'CONFLICT_DETECTED':
        return '競合が検出されました。手動で解決してください';
      case 'SYNC_FAILED':
        return '同期に失敗しました';
      case 'NETWORK_ERROR':
        return 'ネットワークエラーが発生しました';
      case 'RATE_LIMITED':
        return 'APIレート制限に達しました。しばらくお待ちください';
      case 'UNAUTHORIZED':
        return '認証エラー。PATを確認してください';
      case 'UNKNOWN':
      default:
        return '予期しないエラーが発生しました';
    }
  }
}

/**
 * エラーファクトリ関数
 */
export function createTaskNotFoundError(taskId: string): DomainError {
  return new DomainError(
    'TASK_NOT_FOUND',
    `Task not found: ${taskId}`
  );
}

export function createProjectNotFoundError(projectId: string): DomainError {
  return new DomainError(
    'PROJECT_NOT_FOUND',
    `Project not found: ${projectId}`
  );
}

export function createColumnNotFoundError(columnId: string): DomainError {
  return new DomainError(
    'COLUMN_NOT_FOUND',
    `Column not found: ${columnId}`
  );
}

export function createConflictError(message: string): DomainError {
  return new DomainError('CONFLICT_DETECTED', message);
}

export function createNetworkError(cause?: Error): DomainError {
  return new DomainError(
    'NETWORK_ERROR',
    'Network error occurred',
    cause
  );
}

export function createRateLimitError(): DomainError {
  return new DomainError(
    'RATE_LIMITED',
    'GitHub API rate limit exceeded'
  );
}

export function createUnauthorizedError(): DomainError {
  return new DomainError(
    'UNAUTHORIZED',
    'Authentication failed'
  );
}
