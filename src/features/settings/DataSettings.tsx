import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  getBackupInfo,
  exportBackup,
  previewRestore,
  restoreBackup,
} from '../../lib/ipc';
import type { RestoreMode, RestorePreview } from '../../lib/models';
import { Button } from '../../components/ui/Button';
import { FormField } from '../../components/ui/FormField';

export function DataSettings() {
  const client = useQueryClient();
  const info = useQuery({ queryKey: ['backup-info'], queryFn: getBackupInfo });
  const [text, setText] = useState('');
  const [mode, setMode] = useState<RestoreMode>('merge');
  const [review, setReview] = useState<RestorePreview | null>(null);
  const [confirmed, setConfirmed] = useState(false);
  const [fileError, setFileError] = useState('');
  const [reading, setReading] = useState(false);
  const exporting = useMutation({ mutationFn: () => exportBackup() });
  const preview = useMutation({
    mutationFn: () => previewRestore(text, mode),
    onSuccess: setReview,
  });
  const restore = useMutation({
    mutationFn: () =>
      restoreBackup(text, mode, review!.expectedState, confirmed),
    onSuccess: () => {
      setReview(null);
      setText('');
      setConfirmed(false);
      void client.invalidateQueries();
    },
  });
  const busy =
    reading || exporting.isPending || preview.isPending || restore.isPending;
  return (
    <section
      className="mb-6 space-y-3 rounded-xl border border-stone-200 bg-white p-6"
      aria-label="Backups and restore"
    >
      <h2 className="text-xl font-semibold">Backups and restore</h2>
      <p>
        Export includes recipes, original quantities, ingredients, custom
        fields, plans, cooking history, shopping edits and settings. Website
        cookies and login sessions are excluded.
      </p>
      {info.data && (
        <details>
          <summary className="pp-muted">Backup location</summary>
          <p className="break-all mt-2">Backup folder: {info.data.directory}</p>
        </details>
      )}
      {info.error && <p role="alert">{info.error.message}</p>}
      <p className="text-sm text-stone-600">
        Backups are kept until you remove them. Copy exported JSON files to
        another location for protection if this computer fails.
      </p>
      <Button
        variant="primary"
        disabled={busy}
        onClick={() => exporting.mutate()}
      >
        Export JSON backup
      </Button>
      {exporting.data && (
        <p role="status" className="break-all">
          Backup saved: {exporting.data.path}
        </p>
      )}
      <fieldset disabled={busy} className="space-y-3">
        <legend className="font-semibold">Restore from a JSON backup</legend>
        <FormField label="Backup file">
          <input
            type="file"
            accept=".json,application/json"
            onChange={async (e) => {
              setReview(null);
              setConfirmed(false);
              preview.reset();
              restore.reset();
              setFileError('');
              setText('');
              const file = e.target.files?.[0];
              if (!file) return;
              if (file.size > 50 * 1024 * 1024) {
                setFileError('Choose a backup smaller than 50 MiB.');
                return;
              }
              setReading(true);
              try {
                setText(await file.text());
              } catch {
                setFileError(
                  'Could not read this file. Your data has not been changed.',
                );
              } finally {
                setReading(false);
              }
            }}
          />
        </FormField>
        <FormField label="Import behavior">
          <select
            value={mode}
            onChange={(e) => {
              setMode(e.target.value as RestoreMode);
              setReview(null);
              setConfirmed(false);
              preview.reset();
              restore.reset();
            }}
          >
            <option value="merge">Merge new records; reject conflicts</option>
            <option value="replace">Replace all household data</option>
          </select>
        </FormField>
        <p>
          {mode === 'merge'
            ? 'Merge adds new IDs and keeps identical records. Different data for an existing ID or conflicting names stops the whole import; nothing is overwritten.'
            : 'Replace restores the entire backup, removing current household records that are absent from it. Website sessions remain separate.'}
        </p>
        <Button
          disabled={!text || busy}
          onClick={() => {
            setReview(null);
            setConfirmed(false);
            preview.mutate();
          }}
        >
          Review backup
        </Button>
        {review && (
          <div
            role="group"
            aria-label="Confirm backup import"
            className="pp-panel space-y-3"
          >
            <p>
              Result: {review.meals} recipes · {review.ingredients} ingredients
              · {review.rows} stored records, including retired data.
            </p>
            <p>
              A recovery backup is saved before importing. If backup creation
              fails, your data will not be changed.
            </p>
            <FormField
              label={
                mode === 'replace'
                  ? 'I confirm replacing all household data with this backup.'
                  : 'I confirm merging the reviewed backup.'
              }
            >
              <input
                type="checkbox"
                checked={confirmed}
                onChange={(e) => setConfirmed(e.target.checked)}
              />
            </FormField>
            <Button
              variant={mode === 'replace' ? 'danger' : 'primary'}
              disabled={!confirmed || busy}
              onClick={() => restore.mutate()}
            >
              Confirm {mode === 'replace' ? 'replacement' : 'merge'}
            </Button>
            <Button
              onClick={() => {
                setReview(null);
                setConfirmed(false);
              }}
            >
              Cancel import
            </Button>
          </div>
        )}
      </fieldset>
      {reading && <p role="status">Reading backup…</p>}
      {restore.data && (
        <p role="status" className="break-all">
          Import completed. Recovery backup: {restore.data.path}
        </p>
      )}
      {(exporting.error || preview.error || restore.error || fileError) && (
        <p role="alert">
          {fileError ||
            exporting.error?.message ||
            preview.error?.message ||
            restore.error?.message}{' '}
          Review again before retrying an import.
        </p>
      )}
    </section>
  );
}
