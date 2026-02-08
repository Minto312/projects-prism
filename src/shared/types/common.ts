/**
 * 共通型定義
 */

/**
 * Result型（成功/失敗を表す）
 */
export type Result<T, E = Error> =
  | { ok: true; value: T }
  | { ok: false; error: E };

export function ok<T>(value: T): Result<T, never> {
  return { ok: true, value };
}

export function err<E>(error: E): Result<never, E> {
  return { ok: false, error };
}

/**
 * Optional型のユーティリティ
 */
export type Nullable<T> = T | null;
export type Optional<T> = T | undefined;

/**
 * オブジェクトのキーを型安全に取得
 */
export type Keys<T> = keyof T;

/**
 * オブジェクトの値の型を取得
 */
export type Values<T> = T[keyof T];

/**
 * DeepPartial型
 */
export type DeepPartial<T> = T extends object
  ? {
      [P in keyof T]?: DeepPartial<T[P]>;
    }
  : T;
