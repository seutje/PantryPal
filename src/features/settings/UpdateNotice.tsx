import { useEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { Button } from '../../components/ui/Button';
import {
  checkUpdate,
  startupUpdate,
  installUpdate,
  type Release,
  type UpdateProgress,
} from '../../lib/updates';

export function UpdateNotice({ manual = false }: { manual?: boolean }) {
  const [release, setRelease] = useState<Release | null>(null);
  const [checking, setChecking] = useState(false);
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');
  const [installing, setInstalling] = useState(false);
  const [progress, setProgress] = useState<UpdateProgress | null>(null);
  useEffect(() => {
    if (manual) return;
    let active = true;
    void startupUpdate()
      .then((result) => {
        if (active) setRelease(result.update);
      })
      .catch(() => undefined);
    return () => {
      active = false;
    };
  }, [manual]);
  async function check() {
    setChecking(true);
    setError('');
    setMessage('');
    try {
      const result = await checkUpdate();
      setRelease(result.update);
      setMessage(
        !result.enabled
          ? 'Update checks are available in installed release builds.'
          : result.update
            ? ''
            : 'PantryPal is up to date.',
      );
    } catch {
      setError(
        'Could not check for updates. You can keep using PantryPal offline and try again later.',
      );
    } finally {
      setChecking(false);
    }
  }
  async function install() {
    if (!release || installing) return;
    setInstalling(true);
    setError('');
    setProgress(null);
    try {
      await installUpdate(release.version, setProgress);
    } catch {
      setInstalling(false);
      setError(
        'Could not verify or start the update. Your household data has not been replaced. Try again or choose Not now.',
      );
    }
  }
  return (
    <>
      {manual && (
        <section
          className="mb-6 space-y-3 rounded-xl border border-stone-200 bg-white p-6"
          aria-label="Application updates"
        >
          <h2 className="text-xl font-semibold">Application updates</h2>
          <p>
            PantryPal checks once when a release build starts. Updates install
            only after you choose to download them. Core use works offline.
          </p>
          <Button
            disabled={checking || installing}
            onClick={() => void check()}
          >
            {checking ? 'Checking…' : 'Check for updates'}
          </Button>
          {message && <p role="status">{message}</p>}
          {error && !release && <p role="alert">{error}</p>}
        </section>
      )}
      {release && (
        <UpdateDialog
          release={release}
          busy={installing}
          error={error}
          progress={progress}
          onInstall={() => void install()}
          onClose={() => {
            if (!installing) {
              setRelease(null);
              setError('');
            }
          }}
        />
      )}
    </>
  );
}
function UpdateDialog({
  release,
  busy,
  error,
  progress,
  onInstall,
  onClose,
}: {
  release: Release;
  busy: boolean;
  error: string;
  progress: UpdateProgress | null;
  onInstall: () => void;
  onClose: () => void;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [opener] = useState(() =>
    document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null,
  );
  useEffect(() => {
    const element = dialog.current!;
    element.showModal();
    return () => {
      element.close();
      opener?.focus();
    };
  }, [opener]);
  const percentage = progress?.total
    ? Math.min(100, Math.round((100 * progress.downloaded) / progress.total))
    : undefined;
  return createPortal(
    <dialog
      ref={dialog}
      aria-label={`PantryPal ${release.version} update`}
      aria-modal="true"
      className="m-auto max-h-[90vh] w-[min(40rem,95vw)] space-y-4 overflow-y-auto rounded-xl bg-white p-6 shadow-xl backdrop:bg-black/40"
      onCancel={(e) => {
        e.preventDefault();
        if (!busy) onClose();
      }}
    >
      <h2 className="text-xl font-semibold">
        PantryPal {release.version} is ready
      </h2>
      <p>
        Save any recipe or ingredient edits before installing. Download and
        install checks the signature and saves a household recovery backup.
        PantryPal closes when the Windows installer starts.
      </p>
      {release.notes && (
        <div>
          <h3 className="font-semibold">What’s new</h3>
          <p className="whitespace-pre-wrap">{release.notes}</p>
        </div>
      )}
      {busy && (
        <div role="status">
          <p>
            {progress?.phase === 'backing_up'
              ? 'Saving recovery backup…'
              : progress?.phase === 'installing'
                ? 'Starting installer…'
                : 'Downloading and verifying…'}
          </p>
          <progress aria-label="Update download" max={100} value={percentage} />
          {percentage !== undefined && <span>{percentage}%</span>}
        </div>
      )}
      {progress?.recoveryPath && (
        <p className="break-all">Recovery backup: {progress.recoveryPath}</p>
      )}
      {error && <p role="alert">{error}</p>}
      <div className="flex gap-3">
        <Button autoFocus disabled={busy} onClick={onClose}>
          Not now
        </Button>
        <Button variant="primary" disabled={busy} onClick={onInstall}>
          {busy ? 'Preparing update…' : 'Download and install'}
        </Button>
      </div>
    </dialog>,
    document.body,
  );
}
