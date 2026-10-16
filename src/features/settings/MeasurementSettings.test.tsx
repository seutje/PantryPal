import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { AppProviders } from '../../app/providers';
import { saveMeasurementPreferences } from '../../lib/ipc';
import { preferences, catalogFixture } from '../../test/catalog-fixture';
import { MeasurementSettings } from './MeasurementSettings';
vi.mock('../../lib/ipc', () => ({
  getMeasurementPreferences: vi.fn(async () => preferences),
  getCatalog: vi.fn(async () => catalogFixture),
  saveMeasurementPreferences: vi.fn(),
}));
describe('measurement preferences', () => {
  it('persists explicit display/review choices through the typed API', async () => {
    const user = userEvent.setup();
    vi.mocked(saveMeasurementPreferences).mockImplementation(
      async (input) => input,
    );
    render(
      <AppProviders>
        <MeasurementSettings />
      </AppProviders>,
    );
    await user.selectOptions(
      await screen.findByLabelText('Measurement mode'),
      'metric',
    );
    await user.click(screen.getByLabelText('Display exact fractions'));
    await user.clear(screen.getByLabelText('Decimal places'));
    await user.type(screen.getByLabelText('Decimal places'), '2');
    await user.click(
      screen.getByRole('button', { name: 'Save measurement preferences' }),
    );
    expect(
      await screen.findByText('Measurement preferences saved.'),
    ).toBeInTheDocument();
    expect(saveMeasurementPreferences).toHaveBeenCalledWith({
      ...preferences,
      mode: 'metric',
      fractions: false,
      decimalPlaces: 2,
    });
  });
});
