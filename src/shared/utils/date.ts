/**
 * 日付ユーティリティ
 */

/**
 * 日付を YYYY-MM-DD 形式の文字列に変換
 */
export function formatDateISO(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

/**
 * 日付を日本語形式の文字列に変換
 */
export function formatDateJa(date: Date): string {
  return date.toLocaleDateString('ja-JP', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  });
}

/**
 * 日付と時刻を日本語形式の文字列に変換
 */
export function formatDateTimeJa(date: Date): string {
  return date.toLocaleString('ja-JP', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

/**
 * epoch ms を Date に変換
 */
export function fromEpochMs(epochMs: number): Date {
  return new Date(epochMs);
}

/**
 * Date を epoch ms に変換
 */
export function toEpochMs(date: Date): number {
  return date.getTime();
}

/**
 * 現在時刻の epoch ms を取得
 */
export function nowEpochMs(): number {
  return Date.now();
}

/**
 * 相対時間を取得（〜前、〜後）
 */
export function getRelativeTime(date: Date, now: Date = new Date()): string {
  const diffMs = now.getTime() - date.getTime();
  const diffSec = Math.floor(diffMs / 1000);
  const diffMin = Math.floor(diffSec / 60);
  const diffHour = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHour / 24);

  if (diffSec < 60) {
    return 'たった今';
  }
  if (diffMin < 60) {
    return `${diffMin}分前`;
  }
  if (diffHour < 24) {
    return `${diffHour}時間前`;
  }
  if (diffDay < 7) {
    return `${diffDay}日前`;
  }
  return formatDateJa(date);
}
