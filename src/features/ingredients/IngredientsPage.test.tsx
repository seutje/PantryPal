import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { AppProviders } from '../../app/providers';
import { getCatalog, nativeAvailable } from '../../lib/ipc';
import { IngredientsPage } from './IngredientsPage';

vi.mock('../../lib/ipc', () => ({
  getCatalog: vi.fn(),
  nativeAvailable: vi.fn(),
  getMeasurementPreferences: vi.fn(async () => ({
    mode: 'mixed',
    cupConvention: null,
    fractions: true,
    decimalPlaces: 3,
    aggregateCompatible: true,
  })),
}));
beforeEach(() => {
  vi.mocked(nativeAvailable).mockReturnValue(true);
});
describe('saved ingredient catalog', () => {
  it('renders stored names, categories, sections and explicit unit labels safely', async () => {
    vi.mocked(getCatalog).mockResolvedValue({
      ingredients: [
        {
          id: 'test',
          name: '<script>Herb</script>',
          categoryId: 'cat',
          sectionId: 'section',
          preferredUnitId: 'cup-us',
          aliases: [],
          seasonMonths: [],
          notes: '',
          retired: false,
          createdAt: '',
          updatedAt: '',
        },
      ],
      categories: [
        { id: 'cat', name: 'Vegetable', sortOrder: 0, retired: false },
      ],
      sections: [
        { id: 'section', name: 'Produce', sortOrder: 0, retired: false },
      ],
      units: [
        {
          id: 'cup-us',
          label: 'US cup',
          symbol: 'US cup',
          dimension: 'volume',
          system: 'us',
          factorNumerator: 2365882365,
          factorDenominator: 10000000,
          offsetNumerator: 0,
          offsetDenominator: 1,
        },
      ],
    });
    render(
      <AppProviders>
        <IngredientsPage />
      </AppProviders>,
    );
    expect(
      await screen.findByText('<script>Herb</script>'),
    ).toBeInTheDocument();
    expect(screen.getByRole('table')).toBeInTheDocument();
    expect(screen.getByRole('cell', { name: 'Produce' })).toBeInTheDocument();
    expect(screen.getByRole('cell', { name: 'US cup' })).toBeInTheDocument();
    expect(document.querySelector('script')).toBeNull();
  });
  it('shows a recoverable storage error without internal details', async () => {
    vi.mocked(getCatalog).mockRejectedValue(new Error('private database path'));
    render(
      <AppProviders>
        <IngredientsPage />
      </AppProviders>,
    );
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Your data has not been reset',
    );
    expect(screen.queryByText(/private database path/)).not.toBeInTheDocument();
  });
  it('does not pretend the browser preview has local data', () => {
    vi.mocked(nativeAvailable).mockReturnValue(false);
    render(
      <AppProviders>
        <IngredientsPage />
      </AppProviders>,
    );
    expect(
      screen.getByText(/Open PantryPal on your desktop/),
    ).toBeInTheDocument();
    expect(screen.queryByRole('table')).not.toBeInTheDocument();
  });
});
