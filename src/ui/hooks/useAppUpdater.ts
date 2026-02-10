/**
 * App Updater Hook
 *
 * アプリケーションの更新チェック・ダウンロード・インストールを管理する
 * Rust側のカスタムコマンド経由で更新チャネル（Stable/Nightly）に対応
 */

import { useState, useCallback, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { relaunch } from '@tauri-apps/plugin-process';
import { updaterApi } from '../../infra/tauri/client';

export type UpdateChannel = 'stable' | 'nightly';

export type UpdateStatus =
  | 'idle'
  | 'checking'
  | 'available'
  | 'up-to-date'
  | 'downloading'
  | 'installing'
  | 'done'
  | 'error';

export interface AppUpdaterState {
  status: UpdateStatus;
  hasUpdate: boolean;
  updateVersion: string | null;
  updateBody: string | null;
  downloadProgress: number | null;
  error: string | null;
  channel: UpdateChannel;
}

interface UpdateProgressPayload {
  chunkLength: number;
  contentLength: number | null;
}

export function useAppUpdater() {
  const [state, setState] = useState<AppUpdaterState>({
    status: 'idle',
    hasUpdate: false,
    updateVersion: null,
    updateBody: null,
    downloadProgress: null,
    error: null,
    channel: 'stable',
  });

  // 起動時にチャネル設定をロード
  useEffect(() => {
    updaterApi.getUpdateChannel().then((ch) => {
      setState((prev) => ({ ...prev, channel: ch as UpdateChannel }));
    }).catch(() => {});
  }, []);

  // ダウンロード進捗トラッキング用のref
  const progressRef = useRef({ totalSize: 0, downloaded: 0 });

  // Rust側からの進捗イベントを受信
  useEffect(() => {
    const unlisten = listen<UpdateProgressPayload>('update-progress', (event) => {
      const { chunkLength, contentLength } = event.payload;
      const p = progressRef.current;
      if (contentLength != null && p.totalSize === 0) {
        p.totalSize = contentLength;
      }
      p.downloaded += chunkLength;
      setState((prev) => ({
        ...prev,
        downloadProgress: p.totalSize > 0
          ? Math.round((p.downloaded / p.totalSize) * 100)
          : null,
      }));
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const setChannel = useCallback(async (newChannel: UpdateChannel) => {
    try {
      await updaterApi.setUpdateChannel(newChannel);
      setState((prev) => ({
        ...prev,
        channel: newChannel,
        // チャネル変更時に前回の更新情報をリセット
        status: 'idle',
        hasUpdate: false,
        updateVersion: null,
        updateBody: null,
        downloadProgress: null,
        error: null,
      }));
    } catch (err) {
      setState((prev) => ({
        ...prev,
        status: 'error',
        error: err instanceof Error ? err.message : 'チャネルの変更に失敗しました',
      }));
    }
  }, []);

  const checkForUpdate = useCallback(async () => {
    setState((prev) => ({ ...prev, status: 'checking', error: null }));
    try {
      const update = await updaterApi.checkForUpdate();
      if (update) {
        setState((prev) => ({
          ...prev,
          status: 'available',
          hasUpdate: true,
          updateVersion: update.version,
          updateBody: update.body,
        }));
      } else {
        setState((prev) => ({
          ...prev,
          status: 'up-to-date',
          hasUpdate: false,
          updateVersion: null,
          updateBody: null,
        }));
      }
    } catch (err) {
      setState((prev) => ({
        ...prev,
        status: 'error',
        error: err instanceof Error ? err.message : '更新の確認に失敗しました',
      }));
    }
  }, []);

  const downloadAndInstall = useCallback(async () => {
    // リトライ時にも正しく動作するようリセット
    progressRef.current = { totalSize: 0, downloaded: 0 };
    setState((prev) => ({ ...prev, status: 'downloading', downloadProgress: 0 }));
    try {
      await updaterApi.downloadAndInstallUpdate();
      setState((prev) => ({ ...prev, status: 'done', downloadProgress: 100 }));
    } catch (err) {
      setState((prev) => ({
        ...prev,
        status: 'error',
        error: err instanceof Error ? err.message : '更新のインストールに失敗しました',
      }));
    }
  }, []);

  const restartApp = useCallback(async () => {
    try {
      await relaunch();
    } catch (err) {
      setState((prev) => ({
        ...prev,
        status: 'error',
        error: err instanceof Error ? err.message : '再起動に失敗しました',
      }));
    }
  }, []);

  return {
    ...state,
    setChannel,
    checkForUpdate,
    downloadAndInstall,
    restartApp,
  };
}
