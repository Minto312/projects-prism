/**
 * Error Dialog Component
 *
 * リトライ不可のエラーをモーダルダイアログで表示する
 */

import { Modal } from './Modal';
import { Button } from './Button';
import {
  useNotificationStore,
  notificationSelectors,
} from '../../../infra/state/notificationStore';

/**
 * Error Dialog — 既存の Modal を再利用
 */
export function ErrorDialog() {
  const dialogError = useNotificationStore(notificationSelectors.dialogError);
  const isDialogOpen = useNotificationStore(notificationSelectors.isDialogOpen);
  const closeDialog = useNotificationStore((state) => state.closeDialog);

  return (
    <Modal
      isOpen={isDialogOpen}
      onClose={closeDialog}
      title="エラー"
      closeOnOverlayClick={false}
      size="sm"
      footer={
        <Button variant="danger" onClick={closeDialog}>
          閉じる
        </Button>
      }
    >
      <div className="space-y-3">
        <p className="text-sm text-gray-700">
          {dialogError?.getUserMessage() ?? '予期しないエラーが発生しました'}
        </p>
        {dialogError && (
          <p className="text-xs text-gray-400">
            エラーコード: {dialogError.code}
          </p>
        )}
      </div>
    </Modal>
  );
}
