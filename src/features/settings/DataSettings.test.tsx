import { render, screen, within, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, expect, it, vi } from 'vitest';
import { AppProviders } from '../../app/providers';
import * as api from '../../lib/ipc';
import { DataSettings } from './DataSettings';
import { UsabilitySettings } from './UsabilitySettings';
vi.mock('../../lib/ipc', () => ({
  getBackupInfo: vi.fn(),
  exportBackup: vi.fn(),
  previewRestore: vi.fn(),
  restoreBackup: vi.fn(),
  getUsabilityPreferences: vi.fn(),
  saveUsabilityPreferences: vi.fn(),
  getCatalog: vi.fn(),
}));
beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(api.getBackupInfo).mockResolvedValue({ directory: 'test-backups' });
  vi.mocked(api.exportBackup).mockResolvedValue({
    path: 'test-backups/export.json',
  });
  vi.mocked(api.previewRestore).mockResolvedValue({
    expectedState: 'revision',
    meals: 3,
    ingredients: 16,
    rows: 120,
  });
  vi.mocked(api.restoreBackup).mockResolvedValue({
    path: 'test-backups/recovery.json',
  });
  vi.mocked(api.getUsabilityPreferences).mockResolvedValue({
    clipboardUrlPrefill: false,
    sectionOrder: ['produce', 'pantry'],
  });
  vi.mocked(api.saveUsabilityPreferences).mockResolvedValue();
  vi.mocked(api.getCatalog).mockResolvedValue({
    ingredients: [],
    categories: [],
    units: [],
    sections: [
      { id: 'produce', name: 'Produce', sortOrder: 0, retired: false },
      { id: 'pantry', name: 'Pantry', sortOrder: 1, retired: false },
    ],
  });
});
function show(node: React.ReactNode) {
  render(<AppProviders>{node}</AppProviders>);
}
function file(text: string) {
  const f = new File([text], 'backup.json', { type: 'application/json' });
  Object.defineProperty(f, 'text', { value: async () => text });
  return f;
}
it('exports to the backend folder and restores only after review and explicit replacement confirmation', async () => {
  const user = userEvent.setup();
  show(<DataSettings />);
  expect(await screen.findByText('Backup folder: test-backups')).toBeVisible();
  await user.click(screen.getByRole('button', { name: 'Export JSON backup' }));
  expect(
    await screen.findByText('Backup saved: test-backups/export.json'),
  ).toBeVisible();
  await user.upload(screen.getByLabelText('Backup file'), file('fixture JSON'));
  await user.selectOptions(screen.getByLabelText('Import behavior'), 'replace');
  expect(api.restoreBackup).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Review backup' }));
  const review = await screen.findByRole('group', {
    name: 'Confirm backup import',
  });
  expect(within(review).getByText(/3 recipes · 16 ingredients/)).toBeVisible();
  const button = within(review).getByRole('button', {
    name: 'Confirm replacement',
  });
  expect(button).toBeDisabled();
  await user.click(within(review).getByRole('checkbox'));
  await user.click(button);
  expect(api.previewRestore).toHaveBeenCalledWith('fixture JSON', 'replace');
  expect(api.restoreBackup).toHaveBeenCalledWith(
    'fixture JSON',
    'replace',
    'revision',
    true,
  );
  expect(
    await screen.findByText(
      'Import completed. Recovery backup: test-backups/recovery.json',
    ),
  ).toBeVisible();
});
it('keeps failed import input and invalidates review when changing mode or cancelling', async () => {
  const user = userEvent.setup();
  show(<DataSettings />);
  await user.upload(screen.getByLabelText('Backup file'), file('fixture'));
  await user.click(screen.getByRole('button', { name: 'Review backup' }));
  await screen.findByRole('group', { name: 'Confirm backup import' });
  await user.selectOptions(screen.getByLabelText('Import behavior'), 'replace');
  expect(
    screen.queryByRole('group', { name: 'Confirm backup import' }),
  ).not.toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: 'Review backup' }));
  await screen.findByRole('group', { name: 'Confirm backup import' });
  await user.click(screen.getByRole('button', { name: 'Cancel import' }));
  expect(api.restoreBackup).not.toHaveBeenCalled();
  vi.mocked(api.previewRestore).mockRejectedValueOnce(
    new Error('Unsupported backup version.'),
  );
  await user.click(screen.getByRole('button', { name: 'Review backup' }));
  expect(await screen.findByRole('alert')).toHaveTextContent(
    'Unsupported backup version',
  );
  expect(screen.getByRole('button', { name: 'Review backup' })).toBeEnabled();
});
it('orders aisles with the keyboard and saves opt-in URL prefill without dropping choices on failure', async () => {
  const user = userEvent.setup();
  show(<UsabilitySettings />);
  const checkbox = await screen.findByRole('checkbox', {
    name: /Prefill recipe URL/,
  });
  expect(checkbox).not.toBeChecked();
  await user.click(checkbox);
  const button = screen.getByRole('button', { name: 'Move Pantry earlier' });
  button.focus();
  await user.keyboard('{Enter}');
  expect(screen.getAllByRole('listitem')[0]).toHaveTextContent('Pantry');
  vi.mocked(api.saveUsabilityPreferences).mockRejectedValueOnce(
    new Error('Save failed.'),
  );
  await user.click(
    screen.getByRole('button', {
      name: 'Save shopping and import preferences',
    }),
  );
  expect(await screen.findByRole('alert')).toHaveTextContent(
    'Your choices are kept',
  );
  expect(api.saveUsabilityPreferences).toHaveBeenCalledWith({
    clipboardUrlPrefill: true,
    sectionOrder: ['pantry', 'produce'],
  });
  expect(checkbox).toBeChecked();
  await user.click(
    screen.getByRole('button', {
      name: 'Save shopping and import preferences',
    }),
  );
  await waitFor(() =>
    expect(api.saveUsabilityPreferences).toHaveBeenCalledTimes(2),
  );
});
