/**
 * Settings Page
 *
 * 設定画面 - PAT設定など
 */

import { useState, useCallback, useEffect } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { getVersion } from '@tauri-apps/api/app';
import { settingsApi, debugApi } from '../../infra/tauri/client';
import { useHasPatQuery, useBootstrapQuery, useHiddenProjectIdsQuery, useSetHiddenProjectIds } from '../../infra/query/bootstrapQuery';
import { queryKeys } from '../../infra/query/keys';
import { useAppUpdater } from '../hooks/useAppUpdater';
import { Button } from '../components/common/Button';
import { ConfirmModal } from '../components/common/Modal';
import { PageSpinner } from '../components/common/Spinner';

export function SettingsPage() {
  const queryClient = useQueryClient();
  const { data: hasPat, isLoading: isCheckingPat } = useHasPatQuery();
  const { data: bootstrapData } = useBootstrapQuery();
  const { data: hiddenProjectIds = [] } = useHiddenProjectIdsQuery();
  const setHiddenProjectIds = useSetHiddenProjectIds();

  const [patInput, setPatInput] = useState('');
  const [showClearConfirm, setShowClearConfirm] = useState(false);
  const [appVersion, setAppVersion] = useState<string | null>(null);
  const [debugLogs, setDebugLogs] = useState<string>('');
  const [isLoadingLogs, setIsLoadingLogs] = useState(false);

  const {
    status: updateStatus,
    updateVersion,
    updateBody,
    downloadProgress,
    error: updateError,
    channel,
    setChannel,
    checkForUpdate,
    downloadAndInstall,
    restartApp,
  } = useAppUpdater();

  // アプリバージョンを取得
  useEffect(() => {
    getVersion().then(setAppVersion).catch(() => {});
  }, []);

  // Nightlyチャンネル時にデバッグログを自動読み込み
  const loadDebugLogs = useCallback(async () => {
    setIsLoadingLogs(true);
    try {
      const logs = await debugApi.getDebugLogs();
      setDebugLogs(logs);
    } catch {
      setDebugLogs('ログの読み込みに失敗しました');
    } finally {
      setIsLoadingLogs(false);
    }
  }, []);

  useEffect(() => {
    if (channel === 'nightly') {
      loadDebugLogs();
    }
  }, [channel, loadDebugLogs]);

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

  const handleToggleProject = useCallback(
    (projectId: string) => {
      const isHidden = hiddenProjectIds.includes(projectId);
      const newIds = isHidden
        ? hiddenProjectIds.filter((id) => id !== projectId)
        : [...hiddenProjectIds, projectId];
      setHiddenProjectIds.mutate(newIds);
    },
    [hiddenProjectIds, setHiddenProjectIds]
  );

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
          必要なスコープ:{' '}
          <code className="rounded bg-gray-100 px-1">repo</code>{' '}
          <code className="rounded bg-gray-100 px-1">read:org</code>{' '}
          <code className="rounded bg-gray-100 px-1">project</code>
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
              スコープで{' '}
              <code className="rounded bg-gray-100 px-1">repo</code>、
              <code className="rounded bg-gray-100 px-1">read:org</code>、
              <code className="rounded bg-gray-100 px-1">project</code>{' '}
              を選択
            </li>
            <li>トークンを生成してコピー</li>
          </ol>
        </div>
      </section>

      {/* プロジェクト表示 */}
      {bootstrapData?.projects && bootstrapData.projects.length > 0 && (
        <section className="mt-6 rounded-lg border border-gray-200 bg-white p-6">
          <h2 className="mb-4 text-lg font-semibold text-gray-900">
            プロジェクト表示
          </h2>
          <p className="mb-4 text-sm text-gray-600">
            サイドバーに表示するプロジェクトを選択してください。
          </p>
          <div className="space-y-3">
            {bootstrapData.projects.map((project) => {
              const isVisible = !hiddenProjectIds.includes(project.id);
              return (
                <label
                  key={project.id}
                  className="flex items-center justify-between rounded-md border border-gray-100 px-4 py-3 hover:bg-gray-50 cursor-pointer"
                >
                  <div className="flex flex-col">
                    <span className="text-sm font-medium text-gray-900">
                      {project.title}
                    </span>
                    <span className="text-xs text-gray-500">
                      {project.ownerLogin}
                    </span>
                  </div>
                  <button
                    type="button"
                    role="switch"
                    aria-checked={isVisible}
                    onClick={() => handleToggleProject(project.id)}
                    className={`relative inline-flex h-6 w-11 shrink-0 rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 ${
                      isVisible ? 'bg-blue-600' : 'bg-gray-200'
                    }`}
                  >
                    <span
                      className={`pointer-events-none inline-block h-5 w-5 rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${
                        isVisible ? 'translate-x-5' : 'translate-x-0'
                      }`}
                    />
                  </button>
                </label>
              );
            })}
          </div>
        </section>
      )}

      {/* アプリケーション更新 */}
      <section className="mt-6 rounded-lg border border-gray-200 bg-white p-6">
        <h2 className="mb-4 text-lg font-semibold text-gray-900">
          アプリケーション更新
        </h2>

        <div className="mb-4 text-sm text-gray-600">
          現在のバージョン:{' '}
          <span className="font-mono font-medium text-gray-900">
            v{appVersion ?? '...'}
          </span>
        </div>

        {/* チャネル切り替え */}
        <div className="mb-4">
          <label className="mb-2 block text-sm font-medium text-gray-700">
            更新チャネル
          </label>
          <div className="flex gap-2">
            <button
              type="button"
              onClick={() => setChannel('stable')}
              disabled={updateStatus === 'downloading'}
              className={`rounded-md px-4 py-2 text-sm font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed ${
                channel === 'stable'
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
              }`}
            >
              Stable
            </button>
            <button
              type="button"
              onClick={() => setChannel('nightly')}
              disabled={updateStatus === 'downloading'}
              className={`rounded-md px-4 py-2 text-sm font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed ${
                channel === 'nightly'
                  ? 'bg-orange-500 text-white'
                  : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
              }`}
            >
              Nightly
            </button>
          </div>
          {channel === 'nightly' && (
            <p className="mt-2 text-xs text-orange-600">
              Nightly ビルドは開発中の最新機能を含みますが、不安定な場合があります。
            </p>
          )}
        </div>

        {/* ステータス表示 */}
        {updateStatus === 'up-to-date' && (
          <div className="mb-4 flex items-center gap-2 rounded-md bg-green-50 p-3">
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
              最新バージョンです
            </span>
          </div>
        )}

        {updateStatus === 'available' && updateVersion && (
          <div className="mb-4 rounded-md bg-blue-50 p-3">
            <div className="flex items-center gap-2">
              <svg
                className="h-5 w-5 text-blue-600"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
              <span className="text-sm font-medium text-blue-800">
                新しいバージョンが利用可能です: v{updateVersion}
              </span>
            </div>
            {updateBody && (
              <p className="mt-2 text-sm text-blue-700 whitespace-pre-wrap">
                {updateBody}
              </p>
            )}
          </div>
        )}

        {updateStatus === 'downloading' && (
          <div className="mb-4">
            <div className="mb-1 flex justify-between text-sm text-gray-600">
              <span>ダウンロード中...</span>
              {downloadProgress != null && <span>{downloadProgress}%</span>}
            </div>
            <div className="h-2 w-full overflow-hidden rounded-full bg-gray-200">
              {downloadProgress != null ? (
                <div
                  className="h-full rounded-full bg-blue-600 transition-all duration-300"
                  style={{ width: `${downloadProgress}%` }}
                />
              ) : (
                <div className="h-full w-full animate-pulse rounded-full bg-blue-400" />
              )}
            </div>
          </div>
        )}

        {updateStatus === 'done' && (
          <div className="mb-4 flex items-center gap-2 rounded-md bg-green-50 p-3">
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
              更新のインストールが完了しました。再起動してください。
            </span>
          </div>
        )}

        {updateStatus === 'error' && updateError && (
          <div className="mb-4 rounded-md bg-red-50 p-3">
            <p className="text-sm text-red-600">{updateError}</p>
          </div>
        )}

        {/* アクションボタン */}
        <div className="flex gap-3">
          {(updateStatus === 'idle' || updateStatus === 'checking' || updateStatus === 'up-to-date' || updateStatus === 'error') && (
            <Button
              variant="secondary"
              onClick={checkForUpdate}
              isLoading={updateStatus === 'checking'}
            >
              更新を確認
            </Button>
          )}

          {updateStatus === 'available' && (
            <Button
              variant="primary"
              onClick={downloadAndInstall}
            >
              更新をインストール
            </Button>
          )}

          {updateStatus === 'done' && (
            <Button
              variant="primary"
              onClick={restartApp}
            >
              再起動
            </Button>
          )}
        </div>
      </section>

      {/* デバッグログ（Nightlyのみ表示） */}
      {channel === 'nightly' && (
        <section className="mt-6 rounded-lg border border-gray-200 bg-white p-6">
          <div className="mb-4 flex items-center justify-between">
            <h2 className="text-lg font-semibold text-gray-900">デバッグログ</h2>
            <Button
              variant="secondary"
              onClick={loadDebugLogs}
              isLoading={isLoadingLogs}
            >
              更新
            </Button>
          </div>
          <pre className="max-h-96 overflow-auto rounded-md bg-gray-900 p-4 text-xs leading-relaxed text-gray-100">
            {debugLogs || 'ログはありません'}
          </pre>
        </section>
      )}

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
