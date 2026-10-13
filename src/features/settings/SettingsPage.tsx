import { useState } from 'react';
import { Button } from '../../components/ui/Button';
import { useQuery } from '@tanstack/react-query';
import { getStorageStatus, nativeAvailable } from '../../lib/ipc';
import { MeasurementSettings } from './MeasurementSettings';
import { BrowserReset } from '../import/ImportPage';
import { DataSettings } from './DataSettings';
import { UsabilitySettings } from './UsabilitySettings';
import { UpdateNotice } from './UpdateNotice';

const tasks = [
  'Measurements',
  'Shopping order',
  'Recipe websites',
  'Backups & restore',
  'Application updates',
] as const;
export function SettingsPage() {
  const [task, setTask] = useState<(typeof tasks)[number]>('Measurements');
  const status = useQuery({
    queryKey: ['storage-status'],
    queryFn: getStorageStatus,
    enabled: nativeAvailable(),
  });
  return (
    <>
      <section className="mb-5 pp-muted" aria-label="Local data">
        <h2 className="sr-only">Local data</h2>
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
      {nativeAvailable() && (
        <>
          <nav aria-label="Settings tasks" className="pp-tab-nav">
            {tasks.map((name) => (
              <Button
                key={name}
                variant="quiet"
                aria-current={task === name ? 'page' : undefined}
                onClick={() => setTask(name)}
              >
                {name}
              </Button>
            ))}
          </nav>
          <div hidden={task !== 'Measurements'}>
            <MeasurementSettings />
          </div>
          <div hidden={task !== 'Shopping order' && task !== 'Recipe websites'}>
            <UsabilitySettings
              section={task === 'Recipe websites' ? 'import' : 'shopping'}
            />
          </div>
          <div hidden={task !== 'Recipe websites'}>
            <h2 className="text-xl font-semibold mb-3">
              Recipe website sessions
            </h2>
            <div className="pp-panel">
              <BrowserReset />
            </div>
          </div>
          <div hidden={task !== 'Backups & restore'}>
            <DataSettings />
          </div>
          <div hidden={task !== 'Application updates'}>
            <UpdateNotice manual />
          </div>
        </>
      )}
      <p className="pp-muted mt-5">
        Core planning and shopping work offline. Recipe websites require an
        internet connection.
      </p>
    </>
  );
}
