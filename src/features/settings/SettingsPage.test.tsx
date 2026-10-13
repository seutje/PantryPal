import userEvent from '@testing-library/user-event';
import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { AppProviders } from '../../app/providers';
import { getStorageStatus, saveMeasurementPreferences } from '../../lib/ipc';
import { SettingsPage } from './SettingsPage';

vi.mock('../../lib/ipc', () => ({
  getStorageStatus: vi.fn(),
  saveMeasurementPreferences: vi.fn(async (input) => input),
  nativeAvailable: () => true,
  getBackupInfo: vi.fn(async () => ({ directory: 'test-backups' })),
  getUsabilityPreferences: vi.fn(async () => ({
    clipboardUrlPrefill: false,
    sectionOrder: [],
  })),
  getMeasurementPreferences: vi.fn(async () => ({
    mode: 'mixed',
    cupConvention: null,
    fractions: true,
    decimalPlaces: 3,
    aggregateCompatible: true,
  })),
  getCatalog: vi.fn(async () => ({
    ingredients: [],
    categories: [],
    sections: [],
    units: [],
  })),
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
  it('retains drafts between settings tasks and clears saved feedback after a change', async () => {
    const user = userEvent.setup();
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
    await user.selectOptions(
      await screen.findByLabelText('Measurement mode'),
      'metric',
    );
    await user.click(screen.getByRole('button', { name: 'Recipe websites' }));
    await user.click(
      screen.getByLabelText(
        'Prefill recipe URL from clipboard when I choose Import recipe',
      ),
    );
    await user.click(screen.getByRole('button', { name: 'Shopping order' }));
    await user.click(screen.getByRole('button', { name: 'Recipe websites' }));
    expect(
      screen.getByLabelText(
        'Prefill recipe URL from clipboard when I choose Import recipe',
      ),
    ).toBeChecked();
    await user.click(screen.getByRole('button', { name: 'Measurements' }));
    expect(screen.getByLabelText('Measurement mode')).toHaveValue('metric');
    await user.click(
      screen.getByRole('button', { name: 'Save measurement preferences' }),
    );
    expect(
      await screen.findByText('Measurement preferences saved.'),
    ).toBeVisible();
    expect(saveMeasurementPreferences).toHaveBeenCalledWith(
      expect.objectContaining({ mode: 'metric' }),
    );
    await user.selectOptions(screen.getByLabelText('Measurement mode'), 'us');
    expect(
      screen.queryByText('Measurement preferences saved.'),
    ).not.toBeInTheDocument();
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
