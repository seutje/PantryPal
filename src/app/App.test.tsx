import { render, screen } from '@testing-library/react';
import { StrictMode } from 'react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { AppProviders } from './providers';
import { getStorageStatus } from '../lib/ipc';

vi.mock('../lib/ipc', () => ({
  nativeAvailable: () => true,
  getBackupInfo: vi.fn(async () => ({ directory: 'test-backups' })),
  getUsabilityPreferences: vi.fn(async () => ({
    clipboardUrlPrefill: false,
    sectionOrder: [],
  })),
  readRecipeClipboardUrl: vi.fn(async () => null),
  changeShopping: vi.fn(),
  copyShopping: vi.fn(),
  getShopping: vi.fn(async () => ({
    weekStart: '2026-12-28',
    revision: 0,
    generated: false,
    stale: false,
    items: [],
    sections: [],
    units: [],
    text: '',
  })),
  getWeek: vi.fn(async () => ({
    weekStart: '2026-12-28',
    weekEnd: '2027-01-03',
    days: [
      '2026-12-28',
      '2026-12-29',
      '2026-12-30',
      '2026-12-31',
      '2027-01-01',
      '2027-01-02',
      '2027-01-03',
    ],
    previousWeek: '2026-12-21',
    nextWeek: '2027-01-04',
    revision: 0,
    entries: [],
    library: {
      meals: [],
      fields: [],
      catalog: { ingredients: [], categories: [], sections: [], units: [] },
    },
  })),
  searchMeals: vi.fn(async () => ({
    meals: [],
    fields: [],
    catalog: { ingredients: [], categories: [], sections: [], units: [] },
  })),
  getMeasurementPreferences: vi.fn(async () => ({
    mode: 'mixed',
    cupConvention: null,
    fractions: true,
    decimalPlaces: 3,
    aggregateCompatible: true,
  })),
  getAppInfo: vi.fn(async () => ({ name: 'PantryPal', version: '0.1.0' })),
  getCatalog: vi.fn(async () => ({
    ingredients: [],
    categories: [],
    sections: [],
    units: [],
  })),
  getStorageStatus: vi.fn(async () => ({
    schemaVersion: 4,
    ingredientCount: 16,
    mealCount: 0,
  })),
}));

describe('navigation shell', () => {
  it('keeps a failed database visible while the navigation shell remains usable', async () => {
    vi.mocked(getStorageStatus).mockRejectedValueOnce(
      new Error('private storage details'),
    );
    render(
      <AppProviders>
        <App />
      </AppProviders>,
    );
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Your data has not been reset',
    );
    expect(
      screen.queryByText(/private storage details/),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole('navigation', { name: 'Main navigation' }),
    ).toBeInTheDocument();
  });
  it('renders the initial section and connects through the typed native API', async () => {
    render(
      <StrictMode>
        <AppProviders>
          <App />
        </AppProviders>
      </StrictMode>,
    );
    expect(
      screen.getByRole('heading', { name: 'Week', level: 1 }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('heading', { name: 'Week', level: 1 }),
    ).not.toHaveFocus();
    expect(
      await screen.findByText('Desktop · v0.1.0 · Local data ready'),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('navigation', { name: 'Main navigation' }),
    ).toBeInTheDocument();
  });

  it('lets the keyboard reach every section and focuses the new heading', async () => {
    const user = userEvent.setup();
    render(
      <AppProviders>
        <App />
      </AppProviders>,
    );
    await user.tab();
    expect(screen.getByRole('link', { name: 'Skip to content' })).toHaveFocus();
    await user.tab();
    expect(screen.getByRole('button', { name: 'Week' })).toHaveFocus();
    for (const name of ['Meals', 'Shopping', 'Ingredients', 'Settings']) {
      screen.getByRole('button', { name }).focus();
      await user.keyboard('{Enter}');
      expect(screen.getByRole('heading', { name, level: 1 })).toHaveFocus();
      expect(screen.getByRole('button', { name })).toHaveAttribute(
        'aria-current',
        'page',
      );
      if (name === 'Meals')
        expect(
          screen.getByRole('button', { name: 'Import recipe' }),
        ).toBeEnabled();
    }
  });
  it('keeps recipe edits when leaving is cancelled and confirms deliberate discard', async () => {
    const user = userEvent.setup();
    const confirm = vi
      .spyOn(window, 'confirm')
      .mockReturnValueOnce(false)
      .mockReturnValueOnce(true);
    render(
      <AppProviders>
        <App />
      </AppProviders>,
    );
    await user.click(screen.getByRole('button', { name: 'Meals' }));
    await screen.findByLabelText('Search meals');
    await user.click(screen.getByRole('button', { name: 'New meal' }));
    await user.type(screen.getByLabelText('Title'), 'Unsaved soup');
    await user.click(screen.getByRole('button', { name: 'Shopping' }));
    expect(screen.getByLabelText('Title')).toHaveValue('Unsaved soup');
    expect(
      screen.getByRole('heading', { name: 'Meals', level: 1 }),
    ).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Shopping' }));
    expect(
      screen.getByRole('heading', { name: 'Shopping', level: 1 }),
    ).toHaveFocus();
    expect(confirm).toHaveBeenCalledTimes(2);
  });
  it('moves navigation focus with arrow keys, Home and End without activating a page', async () => {
    const user = userEvent.setup();
    render(
      <AppProviders>
        <App />
      </AppProviders>,
    );
    screen.getByRole('button', { name: 'Week' }).focus();
    await user.keyboard('{ArrowDown}');
    expect(screen.getByRole('button', { name: 'Meals' })).toHaveFocus();
    expect(
      screen.getByRole('heading', { name: 'Week', level: 1 }),
    ).toBeInTheDocument();
    await user.keyboard('{End}');
    expect(screen.getByRole('button', { name: 'Settings' })).toHaveFocus();
    await user.keyboard('{Home}');
    expect(screen.getByRole('button', { name: 'Week' })).toHaveFocus();
    await user.keyboard('{ArrowUp}');
    expect(screen.getByRole('button', { name: 'Settings' })).toHaveFocus();
  });
});
