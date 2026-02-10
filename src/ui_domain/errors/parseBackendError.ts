/**
 * Backend Error Parser
 *
 * 任意のエラーを DomainError に変換する
 */

import { DomainError, type DomainErrorCode } from './DomainError';

/**
 * Rust側の #[error("...")] プレフィックスとDomainErrorCodeのマッピング
 */
const ERROR_PREFIX_MAP: ReadonlyArray<[string, DomainErrorCode]> = [
  ['Authentication error', 'UNAUTHORIZED'],
  ['Rate limited', 'RATE_LIMITED'],
  ['Network error', 'NETWORK_ERROR'],
  ['Conflict detected', 'CONFLICT_DETECTED'],
  ['Sync failed', 'SYNC_FAILED'],
  ['Task not found', 'TASK_NOT_FOUND'],
  ['Project not found', 'PROJECT_NOT_FOUND'],
  ['Column not found', 'COLUMN_NOT_FOUND'],
  ['Invalid operation', 'INVALID_OPERATION'],
];

/**
 * エラーメッセージからDomainErrorCodeを推定する
 */
function inferErrorCode(message: string): DomainErrorCode {
  for (const [prefix, code] of ERROR_PREFIX_MAP) {
    if (message.startsWith(prefix)) {
      return code;
    }
  }
  return 'UNKNOWN';
}

/**
 * 任意のエラーを DomainError に変換する
 */
export function toDomainError(error: unknown): DomainError {
  if (error instanceof DomainError) {
    return error;
  }

  if (error instanceof Error) {
    const code = inferErrorCode(error.message);
    return new DomainError(code, error.message, error);
  }

  if (typeof error === 'string') {
    const code = inferErrorCode(error);
    return new DomainError(code, error);
  }

  return new DomainError('UNKNOWN', '予期しないエラーが発生しました');
}
