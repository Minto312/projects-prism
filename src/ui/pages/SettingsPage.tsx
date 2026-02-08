/**
 * Settings Page
 *
 * 設定画面 - PAT設定など
 */

import { useState, useCallback } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { settingsApi } from '../../infra/tauri/client';
import { useHasPatQuery } from '../../infra/query/bootstrapQuery';
import { queryKeys } from '../../infra/query/keys';
import { Button } from '../components/common/Button';
import { ConfirmModal } from '../components/common/Modal';
import { PageSpinner } from '../components/common/Spinner';

export function SettingsPage() {
  const queryClient = useQueryClient();
  const { data: hasPat, isLoading: isCheckingPat } = useHasPatQuery();

  const [patInput, setPatInput] = useState('');
  const [showClearConfirm, setShowClearConfirm] = useState(false);

  // PAT設定のミューテーション
  const setPatMutation = useMutation({
    mutationFn: (pat: string) => settingsApi.setPat(pat),
    onSuccess: () => {
      setPatInput('');
      queryClient.invalidateQueries({ queryKey: queryKeys.settings.hasPat() });
      queryClient.invalidateQueries({ queryKey: queryKeys.bootstrap.all });
    },
  });

  // PAT削除のミューテーション
  const clearPatMutation = useMutation({
    mutationFn: () => settingsApi.clearPat(),
    onSuccess: () => {
      setShowClearConfirm(false);
      queryClient.invalidateQueries({ queryKey: queryKeys.settings.hasPat() });
      queryClient.invalidateQueries({ queryKey: queryKeys.bootstrap.all });
    },
  });

  const handleSetPat = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();
      if (!patInput.trim()) return;
      await setPatMutation.mutateAsync(patInput.trim());
    },
    [patInput, setPatMutation]
  );

  const handleClearPat = useCallback(async () => {
    await clearPatMutation.mutateAsync();
  }, [clearPatMutation]);

  if (isCheckingPat) {
    return <PageSpinner />;
  }

  return (
    <div className="mx-auto max-w-2xl">
      <h1 className="mb-8 text-2xl font-bold text-gray-900">設定</h1>

      {/* GitHub Personal Access Token */}
      <section className="rounded-lg border border-gray-200 bg-white p-6">
        <h2 className="mb-4 text-lg font-semibold text-gray-900">
          GitHub Personal Access Token
        </h2>

        <p className="mb-4 text-sm text-gray-600">
          GitHub Projects V2にアクセスするために、Personal Access Token
          (Classic)が必要です。
          <br />
          必要なスコープ: <code className="rounded bg-gray-100 px-1">repo</code>
        </p>

        {hasPat ? (
          <div className="space-y-4">
            <div className="flex items-center gap-2 rounded-md bg-green-50 p-3">
              <svg
                className="h-5 w-5 text-green-600"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M5 13l4 4L19 7"
                />
              </svg>
              <span className="text-sm font-medium text-green-800">
                PATが設定されています
              </span>
            </div>

            <div className="flex gap-3">
              <Button
                variant="secondary"
                onClick={() => setShowClearConfirm(true)}
                isLoading={clearPatMutation.isPending}
              >
                PATを削除
              </Button>
            </div>
          </div>
        ) : (
          <form onSubmit={handleSetPat} className="space-y-4">
            <div>
              <label
                htmlFor="pat"
                className="block text-sm font-medium text-gray-700"
              >
                Personal Access Token
              </label>
              <input
                type="password"
                id="pat"
                value={patInput}
                onChange={(e) => setPatInput(e.target.value)}
                placeholder="ghp_xxxxxxxxxxxxxxxxxxxx"
                className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                autoComplete="off"
              />
            </div>

            {setPatMutation.isError && (
              <p className="text-sm text-red-600">
                PATの設定に失敗しました。正しいトークンか確認してください。
              </p>
            )}

            <Button
              type="submit"
              variant="primary"
              isLoading={setPatMutation.isPending}
              disabled={!patInput.trim()}
            >
              保存
            </Button>
          </form>
        )}

        {/* PAT作成のヘルプ */}
        <div className="mt-6 border-t border-gray-200 pt-4">
          <h3 className="text-sm font-medium text-gray-900">PATの作成方法</h3>
          <ol className="mt-2 list-inside list-decimal space-y-1 text-sm text-gray-600">
            <li>
              GitHubの{' '}
              <a
                href="https://github.com/settings/tokens"
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-600 hover:underline"
              >
                Settings → Developer settings → Personal access tokens
              </a>{' '}
              にアクセス
            </li>
            <li>「Generate new token (classic)」をクリック</li>
            <li>
              スコープで <code className="rounded bg-gray-100 px-1">repo</code>{' '}
              を選択
            </li>
            <li>トークンを生成してコピー</li>
          </ol>
        </div>
      </section>

      {/* PAT削除確認ダイアログ */}
      <ConfirmModal
        isOpen={showClearConfirm}
        onClose={() => setShowClearConfirm(false)}
        onConfirm={handleClearPat}
        title="PATを削除"
        message="Personal Access Tokenを削除しますか？削除するとGitHub Projectsにアクセスできなくなります。"
        confirmLabel="削除"
        variant="danger"
        isLoading={clearPatMutation.isPending}
      />
    </div>
  );
}
