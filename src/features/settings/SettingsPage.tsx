import { EmptyState } from '../../components/EmptyState';
import { useQuery } from '@tanstack/react-query';
import { getStorageStatus, nativeAvailable } from '../../lib/ipc';
import { MeasurementSettings } from './MeasurementSettings';
import { BrowserReset } from '../import/ImportPage';

export function SettingsPage() {
  const status = useQuery({
    queryKey: ['storage-status'],
    queryFn: getStorageStatus,
    enabled: nativeAvailable(),
  });
  return (
    <>
      <section
        className="mb-6 rounded-xl border border-stone-200 bg-white p-6"
        aria-label="Local data"
      >
        <h2 className="mb-2 text-xl font-semibold">Local data</h2>
        {!nativeAvailable() ? (
          <p>Open the desktop app to check your local data.</p>
        ) : status.isError ? (
          <p role="alert">
            Local data could not be opened. Your data has not been reset. Close
            PantryPal and retry.
          </p>
        ) : status.data ? (
          <p role="status">
            Saved on this computer · {status.data.ingredientCount} active
            ingredients · {status.data.mealCount} active meals
          </p>
        ) : (
          <p role="status">Checking local data…</p>
        )}
      </section>
      {nativeAvailable() && <MeasurementSettings />}
      {nativeAvailable() && <BrowserReset />}
      <EmptyState title="Made for your household">
        Backups will be available in a later update.
      </EmptyState>
    </>
  );
}
