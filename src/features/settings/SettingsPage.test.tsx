import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { AppProviders } from '../../app/providers';
import { getStorageStatus } from '../../lib/ipc';
import { SettingsPage } from './SettingsPage';

vi.mock('../../lib/ipc', () => ({
  getStorageStatus: vi.fn(),
  nativeAvailable: () => true,
}));
describe('local storage status', () => {
  it('shows repository counts', async () => {
    vi.mocked(getStorageStatus).mockResolvedValue({
      schemaVersion: 4,
      ingredientCount: 16,
      mealCount: 2,
    });
    render(
      <AppProviders>
        <SettingsPage />
      </AppProviders>,
    );
    expect(
      await screen.findByText(/16 active ingredients · 2 active meals/),
    ).toBeInTheDocument();
  });
  it('keeps a failed database visible without offering a data reset', async () => {
    vi.mocked(getStorageStatus).mockRejectedValue(
      new Error('migration_failed'),
    );
    render(
      <AppProviders>
        <SettingsPage />
      </AppProviders>,
    );
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Your data has not been reset',
    );
  });
});
