import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { AppProviders } from '../../app/providers';
import { convertMeasurement } from '../../lib/ipc';
import { catalogFixture, preferences } from '../../test/catalog-fixture';
import { ConversionControl } from './ConversionControl';
vi.mock('../../lib/ipc', () => ({
  getMeasurementPreferences: vi.fn(async () => ({
    ...preferences,
    cupConvention: 'metric',
  })),
  convertMeasurement: vi.fn(),
}));
describe('inline conversion', () => {
  it('requires an explicit regional selection and shows approximation and exact rounding', async () => {
    const user = userEvent.setup();
    vi.mocked(convertMeasurement)
      .mockResolvedValueOnce({
        status: 'cannot_convert',
        reason: 'Ambiguous unit',
        candidates: ['cup-metric', 'cup-us', 'cup-imperial'],
        original: {
          amount: '1',
          fromUnit: 'cup',
          toUnit: 'ml',
          ingredientId: null,
          purpose: 'quantity',
        },
      })
      .mockResolvedValueOnce({
        status: 'converted',
        original: {
          amount: '1',
          fromUnit: 'cup-metric',
          toUnit: 'ml',
          ingredientId: null,
          purpose: 'quantity',
        },
        fromUnitId: 'cup-metric',
        toUnitId: 'ml',
        numerator: '1000',
        denominator: '3',
        exact: '1000/3',
        display: '333.333',
        rounded: true,
        approximate: true,
        ruleIds: ['rule'],
      });
    render(
      <AppProviders>
        <ConversionControl units={catalogFixture.units} />
      </AppProviders>,
    );
    await user.selectOptions(screen.getByLabelText('From unit'), 'cup');
    await user.selectOptions(screen.getByLabelText('To unit'), 'ml');
    await user.click(screen.getByRole('button', { name: 'Convert' }));
    expect(await screen.findByRole('status')).toHaveTextContent(
      'Cannot convert',
    );
    expect(screen.getByText(/review preference is metric/)).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Use Metric cup' }));
    expect(screen.queryByRole('status')).not.toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Convert' }));
    expect(await screen.findByRole('status')).toHaveTextContent(
      'Approximate: 333.333 mL (rounded; exact 1000/3)',
    );
    expect(convertMeasurement).toHaveBeenLastCalledWith({
      amount: '1',
      fromUnit: 'cup-metric',
      toUnit: 'ml',
      ingredientId: null,
      purpose: 'quantity',
    });
  });
  it('sends signed temperatures separately and clears results when inputs change', async () => {
    const user = userEvent.setup();
    vi.mocked(convertMeasurement).mockResolvedValue({
      status: 'converted',
      original: {
        amount: '-40',
        fromUnit: 'celsius',
        toUnit: 'fahrenheit',
        ingredientId: null,
        purpose: 'temperature',
      },
      fromUnitId: 'celsius',
      toUnitId: 'fahrenheit',
      numerator: '-40',
      denominator: '1',
      exact: '-40',
      display: '-40',
      rounded: false,
      approximate: false,
      ruleIds: [],
    });
    render(
      <AppProviders>
        <ConversionControl units={catalogFixture.units} temperature />
      </AppProviders>,
    );
    expect(
      screen.queryByRole('option', { name: 'Gram' }),
    ).not.toBeInTheDocument();
    await user.clear(screen.getByLabelText('Amount to convert'));
    await user.type(screen.getByLabelText('Amount to convert'), '-40');
    await user.click(screen.getByRole('button', { name: 'Convert' }));
    expect(await screen.findByRole('status')).toHaveTextContent('-40 °F');
    expect(convertMeasurement).toHaveBeenLastCalledWith({
      amount: '-40',
      fromUnit: 'celsius',
      toUnit: 'fahrenheit',
      ingredientId: null,
      purpose: 'temperature',
    });
    await user.type(screen.getByLabelText('Amount to convert'), '0');
    expect(screen.queryByRole('status')).not.toBeInTheDocument();
  });
});
