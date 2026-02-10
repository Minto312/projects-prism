/**
 * Backend Error Parser
 *
 * 任意のエラーを DomainError に変換する
 */

import { DomainError, type DomainErrorCode } from './DomainError';

/**
 * Rust側の #[error("...")] プレフィックスとDomainErrorCodeのマッピング
 *
 * Core層 DomainError の定義:
 *   Authentication error: {0}  → UNAUTHORIZED
 *   GitHub API error: {0}      → SYNC_FAILED
 *   Rate limited until {..}    → RATE_LIMITED
 *   Persistence error: {0}     → SYNC_FAILED
 *   Conflict detected for ..   → CONFLICT_DETECTED
 *   Not found: {0}             → TASK_NOT_FOUND
 *   Invalid input: {0}         → INVALID_OPERATION
 *   Network error: {0}         → NETWORK_ERROR
 */
const ERROR_PREFIX_MAP: ReadonlyArray<[string, DomainErrorCode]> = [
  ['Authentication error', 'UNAUTHORIZED'],
  ['Rate limited', 'RATE_LIMITED'],
  ['Network error', 'NETWORK_ERROR'],
  ['Conflict detected', 'CONFLICT_DETECTED'],
  ['GitHub API error', 'SYNC_FAILED'],
  ['Persistence error', 'SYNC_FAILED'],
  ['Not found', 'TASK_NOT_FOUND'],
  ['Invalid input', 'INVALID_OPERATION'],
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
